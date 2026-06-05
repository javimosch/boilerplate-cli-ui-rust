# boilerplate-cli-ui-rust

Rust CLI with embedded web UI. Single binary, no runtime dependencies.

Part of [SuperCLI](https://github.com/javimosch/supercli) - build CLI/UI plugins fast for 2026.

| Stack | Repo | Binary |
|-------|------|--------|
| Go + inline HTML | [boilerplate-cli-ui-go](https://github.com/javimosch/boilerplate-cli-ui-go) | ~5MB |
| Go + Vue 3 CDN | [boilerplate-cli-ui-go-v2-vue](https://github.com/javimosch/boilerplate-cli-ui-go-v2-vue) | ~5MB |
| Go + React 18 CDN | [boilerplate-cli-ui-go-v2-react](https://github.com/javimosch/boilerplate-cli-ui-go-v2-react) | ~5MB |
| Deno + vanilla JS | [boilerplate-cli-ui-deno](https://github.com/javimosch/boilerplate-cli-ui-deno) | ~76MB |
| Node.js + vanilla JS | [boilerplate-cli-ui-node](https://github.com/javimosch/boilerplate-cli-ui-node) | ~123MB |
| Python + React CDN | [boilerplate-cli-ui-python](https://github.com/javimosch/boilerplate-cli-ui-python) | ~10MB |
| **Rust + vanilla JS** | **boilerplate-cli-ui-rust** | **~1.1MB** |
| .NET 8 + Vue 3 | [boilerplate-cli-ui-dotnet](https://github.com/javimosch/boilerplate-cli-ui-dotnet) | ~89MB |
| C++ + Vue 3 | [boilerplate-cli-ui-cpp](https://github.com/javimosch/boilerplate-cli-ui-cpp) | ~493KB |
| Nim + Vue 3 | [boilerplate-cli-ui-nim](https://github.com/javimosch/boilerplate-cli-ui-nim) | ~364KB |
| Zig + Vue 3 | [boilerplate-cli-ui-zig](https://github.com/javimosch/boilerplate-cli-ui-zig) | ~190KB |
| Dart + Vue 3 | [boilerplate-cli-ui-dart](https://github.com/javimosch/boilerplate-cli-ui-dart) | ~6.4MB |

## Architecture

```
boilerplate-cli-ui-rust/
├── src/
│   └── main.rs         # CLI + HTTP server (axum)
├── ui/                 # Frontend (embedded at compile time)
│   ├── index.html      # Entry point
│   ├── app.js          # Vanilla JS with hashbang routing
│   └── css/
│       └── styles.css  # Custom styles
├── Cargo.toml
├── build.sh
└── README.md
```

## Key Feature: `include_str!`

Frontend files are **embedded into the binary** at compile time:

```rust
const INDEX_HTML: &str = include_str!("../ui/index.html");
const APP_JS: &str = include_str!("../ui/app.js");
const STYLES_CSS: &str = include_str!("../ui/css/styles.css");
```

**Benefits:**
- Single binary output (no runtime file dependencies)
- Separate HTML/CSS/JS files (proper syntax highlighting)
- No build step for frontend (CDN-based Tailwind + Lucide)
- Tiny binary size (~2-5MB stripped)

## Build

```bash
chmod +x build.sh
./build.sh
```

Or manually:

```bash
cargo build --release
```

Output: `target/release/boilerplate-cli-ui-rust`

## Usage

```bash
# Start server (foreground)
./target/release/boilerplate-cli-ui-rust start

# Start on custom port
./target/release/boilerplate-cli-ui-rust start -p 3000

# Show version
./target/release/boilerplate-cli-ui-rust version

# Show help
./target/release/boilerplate-cli-ui-rust help
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | Web UI |
| `GET /api/status` | Server status (JSON) |
| `GET /api/health` | Health check (JSON) |

## Hashbang Routing

Routes use hashbang URLs:
- `http://localhost:8080/#/dashboard` - Dashboard view
- `http://localhost:8080/#/settings` - Settings view
- `http://localhost:8080/` - Defaults to dashboard

## Frontend Stack

- **Vue 3** (CDN) - Reactive UI with hashbang routing
- **Tailwind CSS** (CDN) - Utility-first styling
- **Lucide Icons** (CDN) - Icon library

## Comparison with Go Versions

| Aspect | Go | Rust |
|--------|-----|------|
| Binary size | ~5MB | ~2-5MB |
| Compile time | Fast | Slower |
| Memory safety | GC | Ownership |
| Web framework | net/http | axum |
| Embed | go:embed | include_str! |

## Development

### Edit Frontend

1. Edit files in `ui/`
2. Run `cargo run -- start` (files re-embed each compile)
3. Refresh browser

### Add API Endpoint

```rust
async fn my_handler() -> Json<MyResponse> {
    Json(MyResponse { /* ... */ })
}

// In main(), add to router:
.route("/api/my-endpoint", get(my_handler))
```

## License

MIT
