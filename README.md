# boilerplate-cli-ui-rust

Rust CLI with embedded web UI. Single binary, no runtime dependencies.
Part of [SuperCLI](https://github.com/javimosch/supercli) - build CLI/UI plugins fast for 2026.
<!-- FLEET-TABLE:BEGIN -->

| Stack | Binary | Cold start | Idle RSS | Specs | SDK |
|-------|--------|-----------:|---------:|:-----:|----:|
| [machin + React 18 CDN](https://github.com/javimosch/boilerplate-cli-ui-machin) | 63 KB | 2 ms | 3.1 MB | 28/28 | ~2 MB |
| [machin isomorphic (wasm UI)](https://github.com/javimosch/boilerplate-cli-ui-machin-isomorphic) | 76 KB | 2 ms | 3.1 MB | 28/28 | ~2 MB |
| [C++ + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-cpp) | 692 KB | 3 ms | 7.2 MB | 28/28 | ~2000 MB |
| [Zig + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-zig) | 971 KB | 1 ms | 2.0 MB | 28/28 | ~50 MB |
| **Rust + vanilla JS** | **1003 KB** | **1 ms** | **2.5 MB** | **28/28** | **~800 MB** |
| [Go + Vue 3 CDN](https://github.com/javimosch/boilerplate-cli-ui-go-v2-vue) | 5.5 MB | 2 ms | 5.9 MB | 28/28 | ~150 MB |
| [Go + React 18 CDN](https://github.com/javimosch/boilerplate-cli-ui-go-v2-react) | 5.5 MB | 2 ms | 5.8 MB | 28/28 | ~150 MB |
| [Deno + vanilla JS](https://github.com/javimosch/boilerplate-cli-ui-deno) | 76.1 MB | 24 ms | 43.8 MB | 28/28 | ~100 MB |
| [Node.js + vanilla JS](https://github.com/javimosch/boilerplate-cli-ui-node) | 122.8 MB | 66 ms | 52.2 MB | 28/28 | ~500 MB |

Not measured in this run (toolchain unavailable): [Nim + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-nim), [V + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-v), [Crystal + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-crystal), [Dart + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-dart), [Python + React CDN](https://github.com/javimosch/boilerplate-cli-ui-python), [.NET 8 + Vue 3](https://github.com/javimosch/boilerplate-cli-ui-dotnet).

*Binary size, cold start (median of 11 `version` runs) and idle RSS measured on Linux-x86_64 on 2026-09-09. **Specs** is the [cli-spec-conformance](https://github.com/javimosch/cli-spec-conformance) score across cli-output-spec, cli-guide-spec and cli-daemon-spec, taken by running each binary — not claimed. Every row builds the same reference app and implements the same agent-first contract, which is what makes the sizes comparable: a 63 KB binary that scores 28/28 is doing the work a 122 MB one does. Rows whose toolchain is missing on the measuring host are left out rather than given a stale number; each is gated on the same conformance check in its own CI. Regenerate with [boilerplate-cli-ui-fleet](https://github.com/javimosch/boilerplate-cli-ui-fleet); never edit this table by hand.*

<!-- FLEET-TABLE:END -->
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
## Key Feature: `include_str!`
Frontend files are **embedded into the binary** at compile time:
```rust
const INDEX_HTML: &str = include_str!("../ui/index.html");
const APP_JS: &str = include_str!("../ui/app.js");
const STYLES_CSS: &str = include_str!("../ui/css/styles.css");
**Benefits:**
- Single binary output (no runtime file dependencies)
- Separate HTML/CSS/JS files (proper syntax highlighting)
- No build step for frontend (CDN-based Tailwind + Lucide)
- Tiny binary size (~2-5MB stripped)
## Build
```bash
chmod +x build.sh
./build.sh
Or manually:
cargo build --release
Output: `target/release/boilerplate-cli-ui-rust`
## Usage
# Start server (foreground)
./target/release/boilerplate-cli-ui-rust start
# Start on custom port
./target/release/boilerplate-cli-ui-rust start -p 3000
# Show version
./target/release/boilerplate-cli-ui-rust version
# Show help
./target/release/boilerplate-cli-ui-rust help
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
| Binary size | ~5MB | ~150MB | ~2-5MB |
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
async fn my_handler() -> Json<MyResponse> {
    Json(MyResponse { /* ... */ })
}
// In main(), add to router:
.route("/api/my-endpoint", get(my_handler))
## License
MIT
