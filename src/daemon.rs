//! Daemon lifecycle (cli-daemon-spec §4).
//!
//! The tool has an embedded HTTP server, so `/_health` is the source of truth
//! for whether the daemon is up — not the pid file, which goes stale when a
//! process dies without cleaning up. Every subcommand is idempotent.
//!
//! The health probe is a hand-rolled HTTP/1.0 request over `TcpStream` rather
//! than an HTTP client dependency: the probe needs one request/response on
//! loopback, and this boilerplate is a binary-size comparison instrument.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::{die, EXIT_EXTERNAL, EXIT_INTERNAL, EXIT_PRECONDITION, TOOL};

pub const PID_FILE: &str = "/tmp/boilerplate-cli-ui-rust.pid";
pub const LOG_FILE: &str = "/tmp/boilerplate-cli-ui-rust.log";

/// One loopback HTTP request. Returns the response text, or None if nothing is
/// listening.
fn http(port: u16, request: &str) -> Option<String> {
    let addr = format!("127.0.0.1:{port}");
    let sockaddr = addr.parse().ok()?;
    let mut stream = TcpStream::connect_timeout(&sockaddr, Duration::from_millis(500)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    stream.write_all(request.as_bytes()).ok()?;

    let mut buf = String::new();
    stream.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn probe_health(port: u16) -> bool {
    let req = format!("GET /_health HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\n\r\n");
    match http(port, &req) {
        Some(resp) => resp.starts_with("HTTP/1.1 200") || resp.starts_with("HTTP/1.0 200"),
        None => false,
    }
}

/// Polls every 100ms for up to 5s, rather than sleeping a fixed amount and
/// hoping (§4).
fn wait_for(port: u16, want: bool) -> bool {
    for _ in 0..50 {
        if probe_health(port) == want {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn emit(v: serde_json::Value) {
    println!("{v}");
}

/// Idempotent: if the port is already healthy it reports the running instance
/// and succeeds, instead of racing a second process onto it.
pub fn start(host: &str, port: u16) {
    if probe_health(port) {
        emit(serde_json::json!({
            "ok": true, "running": true, "already_running": true, "port": port
        }));
        return;
    }

    let exe = std::env::current_exe().unwrap_or_else(|e| {
        die(
            EXIT_INTERNAL,
            "no_executable_path",
            &format!("cannot resolve own path: {e}"),
            "run the binary by an absolute path",
        )
    });

    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .unwrap_or_else(|e| {
            die(
                EXIT_PRECONDITION,
                "log_unwritable",
                &format!("cannot open {LOG_FILE}: {e}"),
                "check permissions on /tmp",
            )
        });
    let log_err = log.try_clone().unwrap_or_else(|e| {
        die(
            EXIT_INTERNAL,
            "log_unwritable",
            &format!("cannot duplicate the log handle: {e}"),
            "check permissions on /tmp",
        )
    });

    let child = Command::new(exe)
        .arg("serve")
        .arg(format!("--host={host}"))
        .arg(format!("--port={port}"))
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err))
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => die(
            EXIT_INTERNAL,
            "spawn_failed",
            &format!("cannot start the daemon: {e}"),
            &format!("{TOOL} serve --port {port}"),
        ),
    };

    let pid = child.id();
    let _ = std::fs::write(PID_FILE, pid.to_string());

    if !wait_for(port, true) {
        let _ = child.kill();
        let _ = std::fs::remove_file(PID_FILE);
        die(
            EXIT_EXTERNAL,
            "daemon_unhealthy",
            &format!("started pid {pid} but /_health never answered on port {port} (see {LOG_FILE})"),
            &format!("{TOOL} serve --port {port}"),
        );
    }

    emit(serde_json::json!({
        "ok": true, "running": true, "already_running": false,
        "pid": pid, "port": port, "log": LOG_FILE
    }));
}

/// A no-op success when nothing is running: an agent stopping an
/// already-stopped daemon has got what it asked for (§4).
pub fn stop(port: u16) {
    if !probe_health(port) {
        let _ = std::fs::remove_file(PID_FILE);
        emit(serde_json::json!({
            "ok": true, "running": false, "stopped": false, "port": port
        }));
        return;
    }

    let token = std::env::var("SHUTDOWN_TOKEN").unwrap_or_default();
    let auth = if token.is_empty() {
        String::new()
    } else {
        format!("X-Shutdown-Token: {token}\r\n")
    };
    let req = format!(
        "POST /_shutdown HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\n{auth}Content-Length: 0\r\n\r\n"
    );

    match http(port, &req) {
        Some(resp) => {
            if !(resp.starts_with("HTTP/1.1 200") || resp.starts_with("HTTP/1.0 200")) {
                let status = resp.lines().next().unwrap_or("").to_string();
                die(
                    EXIT_EXTERNAL,
                    "shutdown_refused",
                    &format!("POST /_shutdown returned: {status}"),
                    "set SHUTDOWN_TOKEN if the daemon is bound off-loopback",
                );
            }
        }
        None => die(
            EXIT_EXTERNAL,
            "shutdown_failed",
            &format!("POST /_shutdown failed on port {port}"),
            &format!("{TOOL} daemon status --port {port}"),
        ),
    }

    wait_for(port, false);
    let _ = std::fs::remove_file(PID_FILE);
    emit(serde_json::json!({
        "ok": true, "running": false, "stopped": true, "port": port
    }));
}

/// Status only ever reads — it never carries the shutdown token (§4).
pub fn status(port: u16) {
    if !probe_health(port) {
        emit(serde_json::json!({"ok": true, "running": false, "port": port}));
        return;
    }
    let pid: u32 = std::fs::read_to_string(PID_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    emit(serde_json::json!({
        "ok": true, "running": true, "pid": pid, "port": port, "log": LOG_FILE
    }));
}
