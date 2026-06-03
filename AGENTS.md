# AGENTS.md - Agent-First Rust CLI with Embedded UI

This document guides AI agents in understanding, extending, and maintaining this agent-first Rust CLI boilerplate.

## Project Philosophy

This boilerplate implements **agent-first CLI design**:

- **JSON-by-default**: API endpoints return JSON
- **Structured errors**: Error objects with code, type, recoverable field
- **Output separation**: stdout for data, stderr for logs/progress
- **Single binary**: Frontend embedded via `include_str!`, no runtime dependencies
- **Agent-first HTTP**: JSON API at `/api/*`, UI at `/`

## Project Structure

```
boilerplate-cli-ui-rust/
├── src/
│   └── main.rs         # CLI + HTTP server (max 500 LOC)
├── ui/                 # Frontend (embedded at compile time)
│   ├── index.html      # Entry point
│   ├── app.js          # Vanilla JS with hashbang routing
│   └── css/
│       └── styles.css  # Custom styles
├── Cargo.toml
├── build.sh
├── README.md
└── AGENTS.md
```

## Coding Rules

### File Size Limits

- **Max 500 LOC per Rust file** - Split files that exceed this
- **Max 300 LOC per JS/CSS file** - Keep files focused

### Rust File Organization

| File | Responsibility |
|------|----------------|
| `main.rs` | CLI parsing, HTTP handlers, state management |

For larger projects, split into:
- `cli.rs` - CLI argument parsing
- `server.rs` - HTTP handlers
- `state.rs` - Application state

## Key Pattern: `include_str!`

This boilerplate uses Rust's macro to embed frontend files:

```rust
const INDEX_HTML: &str = include_str!("../ui/index.html");
const APP_JS: &str = include_str!("../ui/app.js");
const STYLES_CSS: &str = include_str!("../ui/css/styles.css");
```

**How it works:**
1. Files are read at compile time
2. Embedded as string constants in the binary
3. Served via axum handlers

**Development workflow:**
- Edit files in `ui/`
- Run `cargo run -- start` (files re-embed each compile)

## Adding New Views

### 1. Create View in app.js

```javascript
function renderMyView() {
    return `
        <div>
            <h2 class="text-2xl font-bold text-gray-900">My View</h2>
            <!-- Your content -->
        </div>
    `;
}
```

### 2. Add Route in app.js

```javascript
function renderPage() {
    const container = document.getElementById('page-content');
    switch (currentRoute) {
        case 'dashboard': container.innerHTML = renderDashboard(); break;
        case 'settings': container.innerHTML = renderSettings(); break;
        case 'my-view': container.innerHTML = renderMyView(); break;
        default: container.innerHTML = renderDashboard();
    }
    lucide.createIcons();
}
```

### 3. Add Nav Item

```javascript
const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: 'layout-dashboard' },
    { id: 'settings', label: 'Settings', icon: 'settings' },
    { id: 'my-view', label: 'My View', icon: 'star' },
];
```

## Adding New API Endpoints

### 1. Define Response Type

```rust
#[derive(Serialize)]
struct MyResponse {
    status: String,
    data: String,
}
```

### 2. Add Handler

```rust
async fn my_handler() -> Json<MyResponse> {
    Json(MyResponse {
        status: "success".to_string(),
        data: "hello".to_string(),
    })
}
```

### 3. Register Route

```rust
let app = Router::new()
    .route("/", get(serve_index))
    .route("/api/my-endpoint", get(my_handler))
    .with_state(state);
```

## Agent-First Design Principles

### JSON API Responses

All `/api/*` endpoints must return JSON:

```rust
#[derive(Serialize)]
struct ApiResponse {
    status: String,
    timestamp: String,
}

async fn handler() -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "success".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
```

### Error Handling

```rust
use axum::http::StatusCode;

async fn handler() -> Result<Json<Data>, (StatusCode, String)> {
    let result = perform_operation()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}
```

## Common Pitfalls

### 1. include_str! Paths

Paths are relative to the source file:

```rust
// In src/main.rs
const HTML: &str = include_str!("../ui/index.html");  // Correct
const HTML: &str = include_str!("ui/index.html");     // Wrong
```

### 2. Embedded Files Must Exist

Build fails if embedded file doesn't exist:

```
error: couldn't read at ../ui/index.html
```

### 3. Content-Type Headers

Set proper content types for embedded files:

```rust
async fn serve_js() -> Response {
    (
        [("content-type", "application/javascript")],
        APP_JS,
    ).into_response()
}
```

## Development Workflow

### Local Development

```bash
# Run in development mode
cargo run -- start

# Run on custom port
cargo run -- start -p 3000

# Build release binary
cargo build --release
```

### Testing

```bash
# Test API
curl http://localhost:8080/api/status | jq

# Test UI
open http://localhost:8080/
```

## Performance Optimization

### Binary Size

```toml
[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
strip = true         # Strip debug symbols
```

### Compile Time

For faster dev builds, use `cargo check` instead of `cargo build`.

## References

- [Axum Documentation](https://docs.rs/axum)
- [Tokio Documentation](https://tokio.rs/)
- [Clap Documentation](https://docs.rs/clap)
