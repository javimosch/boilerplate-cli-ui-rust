//! The embedded guide and command catalog (cli-guide-spec, cli-output-spec §4).
//!
//! Compiled into the binary: an agent that lands on a machine with this binary
//! and no network can still learn the tool. Never fetch this at runtime.

use crate::{TOOL, VERSION};

pub fn guide_json() -> String {
    serde_json::json!({
        TOOL: "A Rust CLI with an embedded web UI, compiled to a single binary.",
        "version": VERSION,
        "one_liner": "Starts an axum HTTP server that serves a Vue 3 dashboard at / and a JSON API \
at /api/*, from one binary with the UI compiled in via include_str! — no assets to deploy \
alongside it.",
        "model": {
            "binary": "one executable; the UI ships inside it through include_str!.",
            "server": "axum on tokio, bound to an explicit host:port — never 0.0.0.0 by default.",
            "daemon": "re-execs itself as `serve`, detached, with /_health as the source of truth for liveness.",
            "contract": "agent-first: data on stdout, context on stderr, semantic exit codes, typed errors, an embedded guide.",
        },
        "loop": [
            "./build.sh — cargo build --release with the UI compiled in",
            "./target/release/boilerplate-cli-ui-rust serve — foreground on 127.0.0.1:8080",
            "open http://127.0.0.1:8080/ for the UI, or curl /api/status for JSON",
            "./target/release/boilerplate-cli-ui-rust daemon start — background it instead",
            "./target/release/boilerplate-cli-ui-rust daemon stop — stop it",
        ],
        "concepts": {
            "embedded UI": "include_str! compiles each ui/ file into the binary. Edit the files, rebuild.",
            "loopback default": "serve binds 127.0.0.1 unless --host says otherwise. Binding the whole network is deliberate.",
            "shutdown token": "off-loopback, POST /_shutdown requires X-Shutdown-Token matching $SHUTDOWN_TOKEN, or it answers 403 and keeps running.",
            "exit codes": "0 ok, 80-89 input, 90-99 state, 100-109 external, 110-119 internal. The code equals .error.code in the body.",
        },
        "commands": {
            "server": [
                "boilerplate-cli-ui-rust serve [--host H] [--port N]",
                "boilerplate-cli-ui-rust daemon start [--port N]",
                "boilerplate-cli-ui-rust daemon stop [--port N]",
                "boilerplate-cli-ui-rust daemon status [--port N]",
            ],
            "introspection": [
                "boilerplate-cli-ui-rust guide [--human]",
                "boilerplate-cli-ui-rust help-json",
                "boilerplate-cli-ui-rust version [--json]",
            ],
        },
        "examples": [
            {"goal": "serve the UI on a custom port",
             "do": ["./target/release/boilerplate-cli-ui-rust serve --port 3000"]},
            {"goal": "background it and confirm it is up",
             "do": ["./target/release/boilerplate-cli-ui-rust daemon start --port 3000",
                    "./target/release/boilerplate-cli-ui-rust daemon status --port 3000"]},
            {"goal": "expose it on the LAN with a kill switch that needs a token",
             "do": ["SHUTDOWN_TOKEN=s3cret ./target/release/boilerplate-cli-ui-rust serve --host 0.0.0.0 --port 8080"]},
        ],
        "gotchas": [
            "The UI is compiled in: editing ui/ does nothing until you rebuild.",
            "serve binds 127.0.0.1 by default. If you expected it on the LAN, pass --host 0.0.0.0 — and then set SHUTDOWN_TOKEN, or /_shutdown answers 403 to everyone.",
            "daemon start is idempotent: called twice it reports the running instance instead of racing a second process onto the port.",
            "daemon stop against a stopped daemon exits 0 — a no-op success, not an error.",
            "Startup lines go to stderr. An agent parsing stdout sees only data.",
        ],
        "see_also": ["https://cli-specs.intrane.fr"],
    })
    .to_string()
}

pub fn guide_markdown() -> String {
    format!(
        r#"# {TOOL}

A Rust CLI with an embedded web UI, compiled to a single binary.

## Model

- One executable; the UI ships inside it through `include_str!`.
- axum on tokio, bound to an explicit host:port — never 0.0.0.0 by default.
- The daemon re-execs itself as `serve`, detached; /_health is liveness.
- Agent-first: data on stdout, context on stderr, semantic exit codes.

## Loop

1. `./build.sh`
2. `./target/release/{TOOL} serve`
3. Open http://127.0.0.1:8080/ or curl /api/status.
4. `./target/release/{TOOL} daemon start` to background it.
5. `./target/release/{TOOL} daemon stop` to stop it.

## Commands

- `serve [--host H] [--port N]`
- `daemon start|stop|status [--port N]`
- `guide [--human]`, `help-json`, `version [--json]`

## Gotchas

- The UI is compiled in: rebuild after editing ui/.
- `serve` binds 127.0.0.1 by default; `--host 0.0.0.0` is deliberate.
- Off-loopback, `POST /_shutdown` needs `X-Shutdown-Token` = `$SHUTDOWN_TOKEN`.
- `daemon start` twice is idempotent; `daemon stop` when stopped exits 0.
"#
    )
}

pub fn llms_txt() -> String {
    format!(
        r#"# {TOOL}

A Rust CLI with an embedded web UI. One binary.

## Drive it

    {TOOL} serve [--host H] [--port N]
    {TOOL} daemon start|stop|status [--port N]

JSON on stdout, context on stderr, exit 0/80-119.

## Learn it

    {TOOL} guide      # embedded, JSON
    {TOOL} help-json  # command catalog

HTTP: GET /  GET /api/status  GET /_health  POST /_shutdown  GET /guide
"#
    )
}

pub fn help_json() -> String {
    serde_json::json!({
        "version": "1.0",
        "tool": TOOL,
        "tool_version": VERSION,
        "commands": [
            {"name": "serve", "summary": "run the HTTP server in the foreground", "flags": [
                {"name": "--host", "summary": "bind address", "default": "127.0.0.1", "env": "HOST"},
                {"name": "--port", "summary": "port", "default": "8080", "env": "PORT"},
            ]},
            {"name": "daemon start", "summary": "start the server in the background (idempotent)"},
            {"name": "daemon stop", "summary": "stop the background server (no-op success if stopped)"},
            {"name": "daemon status", "summary": "report background server status"},
            {"name": "guide", "summary": "the embedded operator guide", "flags": [
                {"name": "--human", "summary": "markdown instead of JSON"},
            ]},
            {"name": "help-json", "summary": "this machine-readable command catalog"},
            {"name": "version", "summary": "print the version", "flags": [
                {"name": "--json", "summary": "JSON output"},
            ]},
        ],
        "endpoints": [
            {"method": "GET", "path": "/", "summary": "the embedded web UI"},
            {"method": "GET", "path": "/api/status", "summary": "app status JSON"},
            {"method": "GET", "path": "/_health", "summary": "liveness: {ok,service,pid}"},
            {"method": "POST", "path": "/_shutdown", "summary": "stop the server; token-gated off-loopback"},
            {"method": "GET", "path": "/guide", "summary": "the guide over HTTP"},
            {"method": "GET", "path": "/llms.txt", "summary": "the short agent-facing README"},
        ],
        "exit_codes": {
            "0": "success",
            "80": "missing argument or bad flag value",
            "85": "unknown command",
            "90": "precondition failed (port unavailable, forbidden)",
            "100": "external failure (the daemon did not answer)",
            "110": "internal error",
        },
        "env": [
            {"name": "PORT", "summary": "default port"},
            {"name": "HOST", "summary": "default bind address"},
            {"name": "SHUTDOWN_TOKEN", "summary": "required by POST /_shutdown when bound off-loopback"},
        ],
        "see_also": [format!("{TOOL} guide"), "https://cli-specs.intrane.fr".to_string()],
    })
    .to_string()
}
