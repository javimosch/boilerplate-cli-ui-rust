//! A Rust CLI with an embedded web UI.
//!
//! The command surface follows the agent-first CLI specs
//! (https://cli-specs.intrane.fr):
//!
//! - `cli-output-spec`  data on stdout, context on stderr, exit codes 80-119,
//!   typed errors, `help-json`
//! - `cli-guide-spec`   `guide`, embedded in the binary
//! - `cli-daemon-spec`  `serve --host --port`, `/_health`, `/_shutdown`,
//!   `daemon start|stop|status`

mod daemon;
mod guide;
mod server;

use std::process::exit;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const TOOL: &str = "boilerplate-cli-ui-rust";

// Semantic exit codes (cli-output-spec §2).
pub const EXIT_MISSING_ARG: i32 = 80;
pub const EXIT_UNKNOWN_COMMAND: i32 = 85;
pub const EXIT_PRECONDITION: i32 = 90;
pub const EXIT_EXTERNAL: i32 = 100;
pub const EXIT_INTERNAL: i32 = 110;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        exit(EXIT_MISSING_ARG);
    }

    let rest = &args[2..];

    match args[1].as_str() {
        "serve" => {
            let (host, port) = serve_flags(rest);
            server::serve(host, port);
        }
        "daemon" => {
            if args.len() < 3 {
                die(
                    EXIT_MISSING_ARG,
                    "missing_argument",
                    "daemon needs a subcommand: start, stop or status",
                    &format!("{TOOL} daemon status"),
                );
            }
            let sub = args[2].clone();
            let (host, port) = serve_flags(&args[3..]);
            match sub.as_str() {
                "start" => daemon::start(&host, port),
                "stop" => daemon::stop(port),
                "status" => daemon::status(port),
                _ => die(
                    EXIT_UNKNOWN_COMMAND,
                    "unknown_command",
                    &format!("unknown daemon subcommand \"{sub}\""),
                    &format!("{TOOL} daemon status"),
                ),
            }
        }
        "guide" => {
            if has_flag(rest, "--human") {
                println!("{}", guide::guide_markdown());
            } else {
                println!("{}", guide::guide_json());
            }
        }
        "help-json" => println!("{}", guide::help_json()),
        "version" => {
            if has_flag(rest, "--json") {
                println!("{{\"version\":\"{VERSION}\",\"name\":\"{TOOL}\"}}");
            } else {
                println!("{TOOL} v{VERSION}");
            }
        }
        "help" | "--help" | "-h" => print_help(),

        // Back-compat alias for the pre-spec command name.
        "start" => {
            let (host, port) = serve_flags(rest);
            if has_flag(rest, "-daemon") || has_flag(rest, "--daemon") {
                daemon::start(&host, port);
            } else {
                server::serve(host, port);
            }
        }
        "stop" => {
            let (_, port) = serve_flags(rest);
            daemon::stop(port);
        }
        "status" => {
            let (_, port) = serve_flags(rest);
            daemon::status(port);
        }

        other => die(
            EXIT_UNKNOWN_COMMAND,
            "unknown_command",
            &format!("unknown command \"{other}\""),
            &format!("{TOOL} help-json"),
        ),
    }
}

/// Resolves `--host`/`--port`. The host default MUST be loopback
/// (cli-daemon-spec §1): serving the whole network is a deliberate act, never
/// something that happens because nobody passed a flag.
fn serve_flags(args: &[String]) -> (String, u16) {
    let host = flag_value(args, "--host")
        .or_else(|| flag_value(args, "-host"))
        .or_else(|| std::env::var("HOST").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let port_str = flag_value(args, "--port")
        .or_else(|| flag_value(args, "-port"))
        .or_else(|| flag_value(args, "-p"))
        .or_else(|| std::env::var("PORT").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "8080".to_string());

    let port: u16 = match port_str.parse() {
        Ok(p) => p,
        Err(_) => die(
            EXIT_MISSING_ARG,
            "bad_flag_value",
            &format!("--port must be a number, got \"{port_str}\""),
            &format!("{TOOL} serve --port 8080"),
        ),
    };

    (host, port)
}

pub fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

/// Reads `--name value` or `--name=value`.
pub fn flag_value(args: &[String], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    for (i, a) in args.iter().enumerate() {
        if a == name {
            return args.get(i + 1).cloned();
        }
        if let Some(v) = a.strip_prefix(&prefix) {
            return Some(v.to_string());
        }
    }
    None
}

/// Emits a typed error on stdout and exits with the matching code. The exit
/// status and `.error.code` are the same number by construction (§2, §3).
pub fn die(code: i32, etype: &str, message: &str, suggestion: &str) -> ! {
    let recoverable = (100..=109).contains(&code);
    let body = serde_json::json!({
        "ok": false,
        "error": {
            "code": code,
            "type": etype,
            "message": message,
            "recoverable": recoverable,
            "suggestions": [suggestion],
        }
    });
    println!("{body}");
    exit(code);
}

/// Help is context, not the answer to a query, so it goes to stderr and stdout
/// stays clean for data (cli-output-spec §1).
fn print_help() {
    eprintln!("{TOOL} - Rust CLI with an embedded web UI");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {TOOL} <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  serve [--host H] [--port N]   run the HTTP server in the foreground");
    eprintln!("  daemon start [--port N]       start it in the background");
    eprintln!("  daemon stop [--port N]        stop the background server");
    eprintln!("  daemon status [--port N]      report background server status");
    eprintln!("  guide [--human]               the embedded operator guide");
    eprintln!("  help-json                     machine-readable command catalog");
    eprintln!("  version [--json]              show version information");
    eprintln!("  help                          show this help message");
    eprintln!();
    eprintln!("Endpoints:");
    eprintln!("  GET  /            Web UI");
    eprintln!("  GET  /api/status  Server status (JSON)");
    eprintln!("  GET  /_health     Liveness: {{ok,service,pid}}");
    eprintln!("  POST /_shutdown   Stop the server (token-gated off-loopback)");
    eprintln!();
    eprintln!("Exit codes: 0 ok, 80-89 input, 90-99 state, 100-109 external, 110-119 internal");
}
