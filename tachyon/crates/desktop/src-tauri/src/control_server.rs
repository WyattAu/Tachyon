//! Debug control plane — localhost HTTP server for agent-driven testing.
//!
//! Compiled only into debug builds (`cfg(debug_assertions)`); release builds
//! get a no-op. At runtime it additionally requires `TACHYON_DEBUG >= 1`.
//!
//! Security model:
//! - Binds to 127.0.0.1 only.
//! - Every request must carry `Authorization: Bearer <token>`; the token is
//!   generated at startup and written to `/tmp/tachyon-control.token` (mode
//!   0600). The token is never logged.
//!
//! Endpoints (all JSON):
//! - `GET  /control/state`                  — app/run metadata
//! - `POST /control/navigate {route}`       — SPA navigate via pushState+popstate
//! - `POST /control/eval {js}`              — run JS, result arrives via events
//! - `POST /control/click {selector}`       — programmatic click
//! - `POST /control/type {selector, text}`  — programmatic typing (native setter + events)
//! - `POST /control/key {key}`              — keyboard event on activeElement
//! - `POST /control/screenshot {name?}`     — capture PNG, returns artifact path
//! - `GET  /control/events?since=N`         — page events + eval results (poll)
//! - `GET  /control/report`                 — markdown run report
//!
//! Eval results are asynchronous: `webview.eval()` is fire-and-forget, so the
//! injected wrapper calls back through the `control_event` IPC command into a
//! shared buffer that `GET /control/events` drains.

#![cfg(debug_assertions)]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tauri::Manager;

/// Shared event buffer: `(seq, payload)` received from the page via
/// `control_event`, plus eval results routed back through the same channel.
#[derive(Default)]
pub struct EventBuffer {
    next_seq: AtomicU64,
    events: Mutex<Vec<(u64, Value)>>,
}

impl EventBuffer {
    pub fn push(&self, mut payload: Value) -> u64 {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("seq".into(), json!(seq));
        }
        self.events.lock().unwrap().push((seq, payload));
        // Bound memory: drop oldest beyond 5000 entries.
        let mut events = self.events.lock().unwrap();
        if events.len() > 5000 {
            let drop = events.len() - 5000;
            events.drain(0..drop);
        }
        seq
    }

    pub fn since(&self, since: u64) -> Vec<(u64, Value)> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|(seq, _)| *seq > since)
            .cloned()
            .collect()
    }
}

pub(crate) fn buffer() -> &'static EventBuffer {
    static BUF: OnceLock<EventBuffer> = OnceLock::new();
    BUF.get_or_init(EventBuffer::default)
}

fn control_token() -> &'static String {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        // Check env override first (CI can pin the token), else generate.
        if let Ok(t) = std::env::var("TACHYON_CONTROL_TOKEN") {
            if !t.trim().is_empty() {
                return t;
            }
        }
        uuid::Uuid::new_v4().simple().to_string()
    })
}

/// Start the control server. Returns the bound port, or `None` when disabled
/// (release build, TACHYON_DEBUG unset, or port bind failure — the last is
/// logged but must never crash the app).
pub fn start_control_server(app: tauri::AppHandle) -> Option<u16> {
    let debug_level = std::env::var("TACHYON_DEBUG")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0);
    if debug_level < 1 {
        return None;
    }

    let port: u16 = std::env::var("TACHYON_CONTROL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(17890);

    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("[control] failed to bind {}: {}", addr, e);
            return None;
        }
    };
    let port = listener.local_addr().ok()?.port();

    // Persist token for controllers (agents/CI read this file).
    let token_path = "/tmp/tachyon-control.token";
    let _ = std::fs::write(token_path, format!("{}\n", control_token()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(token_path, std::fs::Permissions::from_mode(0o600));
    }

    tracing::info!(
        "[control] debug control plane listening on http://{} (token in {})",
        addr,
        token_path
    );

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let app = app.clone();
            std::thread::spawn(move || {
                let _ = handle_connection(stream, app);
            });
        }
    });

    Some(port)
}

fn handle_connection(
    stream: std::net::TcpStream,
    app: tauri::AppHandle,
) -> Result<(), std::io::Error> {
    let mut reader = BufReader::new(stream.try_clone()?);

    // Request line
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();
    if method.is_empty() || target.is_empty() {
        return Ok(());
    }

    // Headers
    let mut content_length = 0usize;
    let mut authorization = String::new();
    loop {
        let mut h = String::new();
        reader.read_line(&mut h)?;
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            let v = v.trim().to_string();
            match k.as_str() {
                "content-length" => content_length = v.parse().unwrap_or(0),
                "authorization" => authorization = v,
                _ => {}
            }
        }
    }

    // Body
    let mut body = vec![0u8; content_length.min(10 * 1024 * 1024)];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    // Auth (constant-time-ish: compare strings; localhost-only anyway)
    let expected = format!("Bearer {}", control_token());
    if authorization != expected {
        return respond(stream, 401, &json!({"error": "unauthorized"}));
    }

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };

    let body_json: Value = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body).unwrap_or(Value::Null)
    };

    let response = route_and_dispatch(&method, &path, &query, &body_json, &app);
    let (status, payload) = match response {
        Ok((s, v)) => (s, v),
        Err((s, v)) => (s, v),
    };
    respond(stream, status, &payload)
}

type DispatchResult = Result<(u16, Value), (u16, Value)>;

fn route_and_dispatch(
    method: &str,
    path: &str,
    query: &str,
    body: &Value,
    app: &tauri::AppHandle,
) -> DispatchResult {
    match (method, path) {
        ("GET", "/control/state") => Ok((200, state_response(app))),
        ("POST", "/control/navigate") => navigate(app, body),
        ("POST", "/control/eval") => eval_js(app, body),
        ("POST", "/control/click") => click(app, body),
        ("POST", "/control/type") => type_text(app, body),
        ("POST", "/control/key") => press_key(app, body),
        ("POST", "/control/screenshot") => screenshot(app, body),
        ("GET", "/control/events") => {
            let since = query
                .split('&')
                .find_map(|p| p.strip_prefix("since="))
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            let events = buffer()
                .since(since)
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<_>>();
            Ok((200, json!({ "events": events })))
        }
        ("GET", "/control/report") => Ok((200, json!({ "report": build_report(app) }))),
        _ => Err((404, json!({"error": "not found"}))),
    }
}

fn state_response(app: &tauri::AppHandle) -> Value {
    let route = current_route(app);
    json!({
        "run_id": crate::commands::run_id(),
        "route": route,
        "debug_level": std::env::var("TACHYON_DEBUG").unwrap_or_else(|_| "0".into()),
        "server_url": std::env::var("TACHYON_API_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
        "test_server": std::env::var("TACHYON_TEST_SERVER").unwrap_or_else(|_| "http://127.0.0.1:8080".into()),
        "test_user": std::env::var("TACHYON_TEST_USER").unwrap_or_else(|_| "admin".into()),
        "test_routes_env": std::env::var("TACHYON_TEST_ROUTES").ok(),
        "version": env!("CARGO_PKG_VERSION"),
    })
}

fn get_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, (u16, Value)> {
    app.get_webview_window("main").ok_or((
        503,
        json!({"error": "main webview window not available yet"}),
    ))
}

/// Fire-and-forget JS eval with a result callback into the event buffer.
///
/// The payload posted back is `{ kind: "eval_result", eval_id, ok, result }`.
fn eval_dispatch(app: &tauri::AppHandle, js: &str) -> DispatchResult {
    let window = get_window(app)?;
    let eval_id = uuid::Uuid::new_v4().simple().to_string();
    let wrapped = format!(
        r#"(function() {{
            var EVAL_ID = "{eval_id}";
            function done(ok, result) {{
                try {{
                    window.__TAURI__.core.invoke('control_event', {{ payload: {{
                        kind: 'eval_result', eval_id: EVAL_ID, ok: ok, result: String(result).substring(0, 16000)
                    }} }});
                }} catch (e) {{ console.error('control callback failed: ' + e); }}
            }}
            try {{
                var r = (function() {{ {js} }})();
                if (r && typeof r.then === 'function') {{
                    r.then(function(v) {{ done(true, v); }}, function(e) {{ done(false, e && e.message ? e.message : e); }});
                }} else {{
                    done(true, r);
                }}
            }} catch (e) {{
                done(false, e && e.message ? e.message : String(e));
            }}
        }})();"#,
        eval_id = eval_id,
        js = js
    );
    window
        .eval(&wrapped)
        .map_err(|e| (500, json!({"error": format!("eval failed: {}", e)})))?;
    Ok((200, json!({ "eval_id": eval_id })))
}

fn eval_js(app: &tauri::AppHandle, body: &Value) -> DispatchResult {
    let js = body
        .get("js")
        .and_then(|v| v.as_str())
        .ok_or((400, json!({"error": "missing 'js'"})))?;
    eval_dispatch(app, js)
}

fn navigate(app: &tauri::AppHandle, body: &Value) -> DispatchResult {
    let route = body
        .get("route")
        .and_then(|v| v.as_str())
        .ok_or((400, json!({"error": "missing 'route'"})))?;
    let route_js = route.replace('\\', "\\\\").replace('"', "\\\"");
    let js = format!(
        r#"history.pushState(null, '', "{}"); window.dispatchEvent(new PopStateEvent('popstate', {{ state: null }})); location.href"#,
        route_js
    );
    eval_dispatch(app, &js)
}

fn click(app: &tauri::AppHandle, body: &Value) -> DispatchResult {
    let selector = body
        .get("selector")
        .and_then(|v| v.as_str())
        .ok_or((400, json!({"error": "missing 'selector'"})))?;
    let sel_js = selector.replace('\\', "\\\\").replace('"', "\\\"");
    let js = format!(
        r#"(function() {{
            var el = document.querySelector("{}");
            if (!el) return "NOT_FOUND: {}";
            el.scrollIntoView({{ block: "center" }});
            el.click();
            return "CLICKED: {}";
        }})()"#,
        sel_js, selector, selector
    );
    eval_dispatch(app, &js)
}

fn type_text(app: &tauri::AppHandle, body: &Value) -> DispatchResult {
    let selector = body
        .get("selector")
        .and_then(|v| v.as_str())
        .ok_or((400, json!({"error": "missing 'selector'"})))?;
    let text = body
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or((400, json!({"error": "missing 'text'"})))?;
    let sel_js = selector.replace('\\', "\\\\").replace('"', "\\\"");
    let text_js = text.replace('\\', "\\\\").replace('"', "\\\"");
    let js = format!(
        r#"(function() {{
            var el = document.querySelector("{sel}");
            if (!el) return "NOT_FOUND: {sel}";
            el.focus();
            var set = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
            if (el.tagName === "TEXTAREA") set = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value").set;
            set.call(el, "{text}");
            el.dispatchEvent(new Event("input", {{ bubbles: true }}));
            el.dispatchEvent(new Event("change", {{ bubbles: true }}));
            return "TYPED " + "{text}".length + " chars into {sel}";
        }})()"#,
        sel = sel_js,
        text = text_js
    );
    eval_dispatch(app, &js)
}

fn press_key(app: &tauri::AppHandle, body: &Value) -> DispatchResult {
    let key = body
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or((400, json!({"error": "missing 'key'"})))?;
    let key_js = key.replace('\\', "\\\\").replace('"', "\\\"");
    let js = format!(
        r#"(function() {{
            var target = document.activeElement || document.body;
            var ev = new KeyboardEvent("keydown", {{ key: "{key}", bubbles: true, cancelable: true }});
            target.dispatchEvent(ev);
            target.dispatchEvent(new KeyboardEvent("keyup", {{ key: "{key}", bubbles: true }}));
            return "KEY {key} dispatched to " + (target.tagName || "body");
        }})()"#,
        key = key_js
    );
    eval_dispatch(app, &js)
}

fn screenshot(app: &tauri::AppHandle, _body: &Value) -> DispatchResult {
    let window = get_window(app)?;
    let route = current_route(app);
    // Reuse the existing capture pipeline (ImageMagick primary, WebKit
    // snapshot fallback) via the existing command implementation.
    let result = std::thread::spawn(move || {
        crate::commands::capture_screenshot(window, route)
    })
    .join();
    match result {
        Ok(Ok(path)) => Ok((200, json!({ "path": path }))),
        Ok(Err(e)) => Err((500, json!({ "error": e }))),
        Err(_) => Err((500, json!({ "error": "screenshot thread panicked" }))),
    }
}

fn current_route(app: &tauri::AppHandle) -> String {
    let Some(window) = app.get_webview_window("main") else {
        return String::new();
    };
    window.url().map(|u| u.path().to_string()).unwrap_or_default()
}

/// Assemble a lightweight markdown report from current state + recent events.
fn build_report(app: &tauri::AppHandle) -> String {
    let state = state_response(app);
    let events = buffer()
        .since(0)
        .into_iter()
        .map(|(_, v)| v)
        .collect::<Vec<_>>();
    let mut md = String::new();
    md.push_str("# Tachyon Control Run Report\n\n");
    md.push_str(&format!(
        "- Generated: {}\n- Run ID: `{}`\n- Route: `{}`\n- Server: {}\n\n",
        chrono::Utc::now().to_rfc3339(),
        state.get("run_id").and_then(|v| v.as_str()).unwrap_or("?"),
        state.get("route").and_then(|v| v.as_str()).unwrap_or("?"),
        state.get("server_url").and_then(|v| v.as_str()).unwrap_or("?"),
    ));
    md.push_str(&format!("## Events ({})\n\n", events.len()));
    md.push_str("```json\n");
    for e in events.iter().rev().take(100).rev() {
        md.push_str(&serde_json::to_string(e).unwrap_or_default());
        md.push('\n');
    }
    md.push_str("```\n");
    md.push_str("\n## Artifacts\n\n");
    if let Ok(entries) = std::fs::read_dir("/tmp/tachyon-screenshots") {
        for e in entries.flatten() {
            md.push_str(&format!("- /tmp/tachyon-screenshots/{}\n", e.file_name().to_string_lossy()));
        }
    }
    md
}

fn respond(mut stream: std::net::TcpStream, status: u16, payload: &Value) -> Result<(), std::io::Error> {
    let body = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        reason,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}
