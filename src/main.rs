use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

// ─── Embedded UI Files ──────────────────────────────────────────
const INDEX_HTML: &str = include_str!("../ui/index.html");
const APP_JS: &str = include_str!("../ui/js/app.js");
const STYLES_CSS: &str = include_str!("../ui/css/styles.css");
const COMPONENTS_APP_LAYOUT: &str = include_str!("../ui/js/components/AppLayout.js");
const COMPONENTS_SIDEBAR: &str = include_str!("../ui/js/components/Sidebar.js");
const COMPONENTS_STATUS_CARD: &str = include_str!("../ui/js/components/StatusCard.js");
const VIEWS_DASHBOARD: &str = include_str!("../ui/js/views/Dashboard.js");
const VIEWS_SETTINGS: &str = include_str!("../ui/js/views/Settings.js");

// ─── CLI ────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(name = "boilerplate-cli-ui-rust")]
#[command(about = "Rust CLI with embedded web UI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server with web UI
    Start {
        /// Port for HTTP server
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Show version information
    Version,
    /// Show help
    Help,
}

// ─── State ──────────────────────────────────────────────────────
#[derive(Clone)]
struct AppState {
    start_time: Instant,
    port: u16,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    port: u16,
    uptime: String,
    version: String,
    start_time: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// ─── Handlers ───────────────────────────────────────────────────
async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn serve_app_js() -> Response {
    (
        [("content-type", "application/javascript")],
        APP_JS,
    )
        .into_response()
}

async fn serve_styles_css() -> Response {
    (
        [("content-type", "text/css")],
        STYLES_CSS,
    )
        .into_response()
}

async fn serve_component_app_layout() -> Response {
    (
        [("content-type", "application/javascript")],
        COMPONENTS_APP_LAYOUT,
    )
        .into_response()
}

async fn serve_component_sidebar() -> Response {
    (
        [("content-type", "application/javascript")],
        COMPONENTS_SIDEBAR,
    )
        .into_response()
}

async fn serve_component_status_card() -> Response {
    (
        [("content-type", "application/javascript")],
        COMPONENTS_STATUS_CARD,
    )
        .into_response()
}

async fn serve_view_dashboard() -> Response {
    (
        [("content-type", "application/javascript")],
        VIEWS_DASHBOARD,
    )
        .into_response()
}

async fn serve_view_settings() -> Response {
    (
        [("content-type", "application/javascript")],
        VIEWS_SETTINGS,
    )
        .into_response()
}

async fn serve_status(State(state): State<Arc<RwLock<AppState>>>) -> Json<StatusResponse> {
    let state = state.read().await;
    let elapsed = state.start_time.elapsed();
    
    let uptime = if elapsed.as_secs() >= 3600 {
        format!("{}h{}m{}s", 
            elapsed.as_secs() / 3600,
            (elapsed.as_secs() % 3600) / 60,
            elapsed.as_secs() % 60)
    } else if elapsed.as_secs() >= 60 {
        format!("{}m{}s", 
            elapsed.as_secs() / 60,
            elapsed.as_secs() % 60)
    } else {
        format!("{}s", elapsed.as_secs())
    };
    
    Json(StatusResponse {
        status: "running".to_string(),
        port: state.port,
        uptime,
        version: env!("CARGO_PKG_VERSION").to_string(),
        start_time: format!("{:?}", state.start_time.elapsed()),
    })
}

async fn serve_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ─── Main ───────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port } => {
            let state = Arc::new(RwLock::new(AppState {
                start_time: Instant::now(),
                port,
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
                .with_state(state);

            let addr = format!("0.0.0.0:{}", port);
            println!("Server starting on http://localhost:{}", port);
            println!("UI available at http://localhost:{}/", port);
            println!("API available at http://localhost:{}/api/status", port);
            println!("Press Ctrl+C to stop");

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Commands::Version => {
            println!("boilerplate-cli-ui-rust v{}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Help => {
            println!("boilerplate-cli-ui-rust - Rust CLI with embedded web UI");
            println!();
            println!("Usage:");
            println!("  boilerplate-cli-ui-rust <command> [options]");
            println!();
            println!("Commands:");
            println!("  start       Start HTTP server with web UI");
            println!("  version     Show version information");
            println!("  help        Show this help message");
            println!();
            println!("Start Options:");
            println!("  -p, --port <PORT>  Port for HTTP server (default 8080)");
            println!();
            println!("API Endpoints:");
            println!("  GET /            Web UI");
            println!("  GET /api/status  Server status (JSON)");
            println!("  GET /api/health  Health check (JSON)");
        }
    }
}
