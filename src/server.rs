//! The HTTP server: the embedded UI, the app API, and the daemon lifecycle
//! routes from cli-daemon-spec §2-§3.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::{die, guide, EXIT_PRECONDITION, TOOL, VERSION};

// ─── Embedded UI Files ──────────────────────────────────────────
const INDEX_HTML: &str = include_str!("../ui/index.html");
const APP_JS: &str = include_str!("../ui/js/app.js");
const STYLES_CSS: &str = include_str!("../ui/css/styles.css");
const COMPONENTS_APP_LAYOUT: &str = include_str!("../ui/js/components/AppLayout.js");
const COMPONENTS_SIDEBAR: &str = include_str!("../ui/js/components/Sidebar.js");
const COMPONENTS_STATUS_CARD: &str = include_str!("../ui/js/components/StatusCard.js");
const VIEWS_DASHBOARD: &str = include_str!("../ui/js/views/Dashboard.js");
const VIEWS_SETTINGS: &str = include_str!("../ui/js/views/Settings.js");

#[derive(Clone)]
pub struct AppState {
    start_time: Instant,
    port: u16,
    /// The address we actually bound. `/_shutdown` is token-gated whenever this
    /// is not loopback (§3).
    host: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    port: u16,
    uptime: String,
    version: String,
}

pub fn is_loopback(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

/// Runs the server in the foreground. Binds `host:port` — never `0.0.0.0`
/// implicitly, so a server told to serve localhost is not reachable from the
/// whole network (cli-daemon-spec §1).
pub fn serve(host: String, port: u16) {
    let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
        die(
            crate::EXIT_INTERNAL,
            "runtime_failed",
            &format!("cannot start the async runtime: {e}"),
            "report this as a bug",
        )
    });

    rt.block_on(async move {
        let state = Arc::new(RwLock::new(AppState {
            start_time: Instant::now(),
            port,
            host: host.clone(),
        }));

        let app = Router::new()
            .route("/", get(serve_index))
            .route("/css/styles.css", get(serve_styles_css))
            .route("/js/app.js", get(serve_app_js))
            .route("/js/components/AppLayout.js", get(serve_component_app_layout))
            .route("/js/components/Sidebar.js", get(serve_component_sidebar))
            .route("/js/components/StatusCard.js", get(serve_component_status_card))
            .route("/js/views/Dashboard.js", get(serve_view_dashboard))
            .route("/js/views/Settings.js", get(serve_view_settings))
            .route("/api/status", get(serve_status))
            .route("/api/health", get(serve_health))
            // Daemon lifecycle (§2, §3).
            .route("/_health", get(serve_health))
            .route("/_shutdown", post(serve_shutdown))
            // The guide over HTTP (cli-guide-spec §3).
            .route("/guide", get(serve_guide))
            .route("/llms.txt", get(serve_llms))
            .with_state(state);

        let addr = format!("{host}:{port}");

        // Startup lines are context — stderr, never stdout (§1).
        eprintln!("{TOOL} serving on http://{addr}/");
        eprintln!("  API: http://{addr}/api/status");

        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => die(
                EXIT_PRECONDITION,
                "port_unavailable",
                &format!("cannot bind {addr}: {e}"),
                &format!("{TOOL} serve --port {}", port.wrapping_add(1)),
            ),
        };

        if let Err(e) = axum::serve(listener, app).await {
            die(
                crate::EXIT_INTERNAL,
                "server_failed",
                &format!("server error: {e}"),
                "report this as a bug",
            );
        }
    });
}

// ─── Lifecycle handlers ─────────────────────────────────────────

/// Open and cheap: liveness only, no dependency checks (§2).
async fn serve_health(State(state): State<Arc<RwLock<AppState>>>) -> Json<serde_json::Value> {
    let state = state.read().await;
    Json(serde_json::json!({
        "ok": true,
        "service": TOOL,
        "pid": std::process::id(),
        "port": state.port,
    }))
}

/// Answers before exiting, and is token-gated whenever the server is bound
/// off-loopback — otherwise it is a remote kill switch (§3).
async fn serve_shutdown(
    State(state): State<Arc<RwLock<AppState>>>,
    headers: HeaderMap,
) -> Response {
    let host = state.read().await.host.clone();

    let authorized = if is_loopback(&host) {
        true
    } else {
        match std::env::var("SHUTDOWN_TOKEN") {
            Ok(token) if !token.is_empty() => headers
                .get("x-shutdown-token")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == token)
                .unwrap_or(false),
            _ => false,
        }
    };

    if !authorized {
        // 403, and the process MUST NOT stop.
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "ok": false,
                "error": {
                    "code": EXIT_PRECONDITION,
                    "type": "forbidden",
                    "message": "X-Shutdown-Token required when bound off-loopback",
                    "recoverable": false,
                }
            })),
        )
            .into_response();
    }

    // Exit after the response has had a moment to flush, so the caller learns
    // the request was accepted rather than seeing a dropped connection.
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = std::fs::remove_file(crate::daemon::PID_FILE);
        std::process::exit(0);
    });

    Json(serde_json::json!({"ok": true, "stopping": true})).into_response()
}

async fn serve_guide() -> Response {
    ([("content-type", "application/json")], guide::guide_json()).into_response()
}

async fn serve_llms() -> Response {
    (
        [("content-type", "text/plain; charset=utf-8")],
        guide::llms_txt(),
    )
        .into_response()
}

// ─── App handlers ───────────────────────────────────────────────

async fn serve_status(State(state): State<Arc<RwLock<AppState>>>) -> Json<StatusResponse> {
    let state = state.read().await;
    let elapsed = state.start_time.elapsed().as_secs();

    let uptime = if elapsed >= 3600 {
        format!("{}h{}m{}s", elapsed / 3600, (elapsed % 3600) / 60, elapsed % 60)
    } else if elapsed >= 60 {
        format!("{}m{}s", elapsed / 60, elapsed % 60)
    } else {
        format!("{elapsed}s")
    };

    Json(StatusResponse {
        status: "running".to_string(),
        port: state.port,
        uptime,
        version: VERSION.to_string(),
    })
}

async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn serve_app_js() -> Response {
    ([("content-type", "application/javascript")], APP_JS).into_response()
}

async fn serve_styles_css() -> Response {
    ([("content-type", "text/css")], STYLES_CSS).into_response()
}

async fn serve_component_app_layout() -> Response {
    (
        [("content-type", "application/javascript")],
        COMPONENTS_APP_LAYOUT,
    )
        .into_response()
}

async fn serve_component_sidebar() -> Response {
    ([("content-type", "application/javascript")], COMPONENTS_SIDEBAR).into_response()
}

async fn serve_component_status_card() -> Response {
    (
        [("content-type", "application/javascript")],
        COMPONENTS_STATUS_CARD,
    )
        .into_response()
}

async fn serve_view_dashboard() -> Response {
    ([("content-type", "application/javascript")], VIEWS_DASHBOARD).into_response()
}

async fn serve_view_settings() -> Response {
    ([("content-type", "application/javascript")], VIEWS_SETTINGS).into_response()
}
