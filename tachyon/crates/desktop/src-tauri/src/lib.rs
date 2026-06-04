// Tauri Desktop Application Library
// IPC bridge between WebView and Tauri backend

// Module declarations
mod commands;
mod events;
mod file_dialog;
mod filesystem;
mod import_export;
mod state;
mod sync;
#[cfg(feature = "tray-icon")]
mod tray;

// Re-export public API
pub use events::{
    DesktopEvent, EventEmitter, FileChangeKind, NotificationLevel, RepositoryStatus,
    SyncStatus as EventSyncStatus,
};
pub use file_dialog::{
    FileContent, FileDialogManager, FileDialogOptions, FileDialogResult, FileWriteResult,
};
pub use filesystem::{FileWatchHandle, MarkdownFile, VaultEntry};
pub use state::{ConnectionStatus, DesktopAppState, DesktopState, DesktopStateManager, SyncStatus};
pub use sync::{AutoSyncManager, CommitQueueEntry, SyncConfig, SyncResult};

use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Embedded server state shared between Tauri and the frontend.
#[derive(Default)]
struct EmbeddedServerState {
    /// Port the embedded server is listening on (0 if not started).
    port: u16,
    /// Whether the server has started successfully.
    started: bool,
}

/// Workaround for WebKitGTK DMA-BUF renderer failures on NVIDIA + Wayland.
///
/// On NVIDIA GPUs with KWin/Sway compositors, WebKitGTK's DMA-BUF renderer
/// fails with "Failed to create GBM buffer of size NxM: Invalid argument"
/// because the NVIDIA EGL/GBM implementation doesn't support the specific
/// buffer modifiers WebKit requests. Setting WEBKIT_DISABLE_DMABUF_RENDERER=1
/// forces WebKit to fall back to shared-memory (shm) rendering which works
/// on all GPU/compositor combinations.
///
/// This only sets the env var if it's not already explicitly configured,
/// so users can override with WEBKIT_DISABLE_DMABUF_RENDERER=0 if needed.
fn fix_webkit_dmabuf_on_nvidia() {
    if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
        // Detect NVIDIA GPU via the DRM subsystem
        let has_nvidia = std::fs::read_dir("/dev/dri/by-path")
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .contains("nvidia")
                })
            })
            .unwrap_or(false);

        // Also check the classic /proc/driver/nvidia path
        let has_nvidia_proc =
            std::path::Path::new("/proc/driver/nvidia").exists();

        if has_nvidia || has_nvidia_proc {
            // SAFETY: This runs before any threads are spawned (single-threaded
            // init phase of the Tauri app). set_var is unsafe in Rust 2024 to
            // prevent data races with concurrent getenv, but we're still in the
            // main thread's sequential setup code.
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
            tracing::info!(
                "NVIDIA GPU detected — set WEBKIT_DISABLE_DMABUF_RENDERER=1 \
                 to avoid DMA-BUF GBM buffer failures on Wayland"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Debug JavaScript — injected via webview.eval() when TACHYON_DEBUG is set.
//
// Two scripts, injected sequentially on page load:
//   1. DEBUG_HOOKS_JS  — error/network/console/DOM hooks (always with debug)
//   2. DEBUG_TRAVERSE_JS — automated UI traversal (only with TACHYON_DEBUG=2)
//
// eval() bypasses CSP entirely — runs with page's origin regardless of nonces.
// All data reported via Tauri IPC to the debug_report command → /tmp/tachyon-debug.jsonl.
//
// Usage:
//   TACHYON_DEBUG=1 ./tachyon-desktop-app   # hooks + DOM snapshots
//   TACHYON_DEBUG=2 ./tachyon-desktop-app   # hooks + DOM + automated traversal
// ---------------------------------------------------------------------------

/// Core debug hooks: captures all JS errors, unhandled rejections, network
/// failures, console output, DOM state, computed styles, and resources.
/// Injected when TACHYON_DEBUG is set to any value.
static DEBUG_HOOKS_JS: &str = r##"
(function() {
    var invoke = null;
    try {
        if (window.__TAURI__) {
            invoke = window.__TAURI__.core && window.__TAURI__.core.invoke
                ? window.__TAURI__.core.invoke.bind(window.__TAURI__.core)
                : (window.__TAURI__.invoke ? window.__TAURI__.invoke.bind(window.__TAURI__) : null);
        }
    } catch(e) { console.error('invoke setup failed: ' + String(e)); }

    function report(data) {
        if (invoke) {
            try { invoke('debug_report', { data: data }).catch(function() {}); }
            catch(e) { console.error('report fail: ' + data.type + ' ' + String(e)); }
        }
    }

    // === Error hooks ===
    window.onerror = function(msg, src, line, col, err) {
        report({ type: 'js_error', message: String(msg), source: src || '', line: line || 0, col: col || 0, stack: err ? (err.stack || '') : '' });
    };
    window.addEventListener('unhandledrejection', function(e) {
        report({ type: 'promise_error', reason: String(e.reason), stack: e.reason && e.reason.stack ? e.reason.stack : '' });
    });
    window.addEventListener('error', function(e) {
        if (e.target) {
            if (e.target.tagName === 'SCRIPT') report({ type: 'script_load_error', src: e.target.src || '', typeAttr: e.target.type || '' });
            else if (e.target.tagName === 'LINK') report({ type: 'link_load_error', href: e.target.href || '', rel: e.target.rel || '' });
            else if (e.target.tagName === 'IMG') report({ type: 'img_load_error', src: e.target.src || '' });
        }
    }, true);

    // === Console hooks ===
    var origError = console.error, origWarn = console.warn, origInfo = console.info, origLog = console.log;
    console.error = function() { var m = Array.prototype.slice.call(arguments).join(' '); origError.apply(console, arguments); report({ type: 'console_error', message: m }); };
    console.warn  = function() { var m = Array.prototype.slice.call(arguments).join(' '); origWarn.apply(console, arguments);  report({ type: 'console_warn', message: m }); };
    console.info  = function() { var m = Array.prototype.slice.call(arguments).join(' '); origInfo.apply(console, arguments);  report({ type: 'console_info', message: m }); };
    console.log   = function() { var m = Array.prototype.slice.call(arguments).join(' '); origLog.apply(console, arguments);  report({ type: 'console_log', message: m }); };

    // === Network hooks ===
    var xhrOpen = XMLHttpRequest.prototype.open, xhrSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function(method, url) { this._du = url; this._dm = method; return xhrOpen.apply(this, arguments); };
    XMLHttpRequest.prototype.send = function() {
        var self = this, oL = this.onload, oE = this.onerror, start = Date.now();
        this.onerror = function() {
            report({ type: 'network_error', method: self._dm, url: self._du, status: self.status, statusText: self.statusText, duration: Date.now() - start });
            if (oE) oE.apply(this, arguments);
        };
        this.onload = function() {
            if (self.status >= 400) report({ type: 'network_fail', method: self._dm, url: self._du, status: self.status, statusText: self.statusText, duration: Date.now() - start });
            if (oL) oL.apply(this, arguments);
        };
        return xhrSend.apply(this, arguments);
    };
    var origFetch = window.fetch;
    window.fetch = function(input, init) {
        var url = typeof input === 'string' ? input : (input && input.url ? input.url : ''), start = Date.now();
        return origFetch.apply(this, arguments).then(function(resp) {
            report({ type: 'fetch_response', url: url, method: init && init.method ? init.method : 'GET', status: resp.status, ok: resp.ok, duration: Date.now() - start });
            return resp;
        }).catch(function(err) {
            report({ type: 'fetch_error', url: url, error: String(err.message || err), duration: Date.now() - start });
            throw err;
        });
    };

    // === Signal loaded ===
    report({ type: 'debugger_loaded', url: location.href, timestamp: Date.now(), tauriExists: !!window.__TAURI__ });

    // === DOM + Style snapshot ===
    function domSnapshot(label) {
        var body = document.body, html = document.documentElement;
        var app = document.getElementById('app');
        var bodyCh = [], appCh = [];
        if (body) for (var i = 0; i < body.children.length; i++) { var c = body.children[i]; bodyCh.push({ tag: c.tagName, id: c.id, cls: (c.className||'').substring(0,100), rect: c.getBoundingClientRect ? { x: Math.round(c.getBoundingClientRect().x), y: Math.round(c.getBoundingClientRect().y), w: Math.round(c.getBoundingClientRect().width), h: Math.round(c.getBoundingClientRect().height) } : null }); }
        if (app) for (var i = 0; i < Math.min(app.children.length, 30); i++) { var c = app.children[i]; appCh.push({ tag: c.tagName, id: c.id, text: c.textContent.substring(0,80) }); }
        var s_app = app ? window.getComputedStyle(app) : null;
        var s_body = body ? window.getComputedStyle(body) : null;
        report({
            type: 'dom_' + label, url: location.href, title: document.title, readyState: document.readyState,
            totalElements: document.querySelectorAll('*').length,
            styledElements: document.querySelectorAll('[class]').length,
            bodyChildren: bodyCh, appExists: !!app,
            appStyle: s_app ? { display: s_app.display, height: s_app.height, width: s_app.width, bg: s_app.backgroundColor } : null,
            bodyStyle: s_body ? { height: s_body.height, width: s_body.width, bg: s_body.backgroundColor } : null,
            hiddenCount: document.querySelectorAll('[style*="display:none"],[style*="visibility:hidden"],[hidden]').length,
            label: label
        });
    }

    // === Scheduled snapshots ===
    setTimeout(function() { domSnapshot('3s'); }, 3000);
    setTimeout(function() { domSnapshot('10s'); }, 10000);
    setTimeout(function() { domSnapshot('30s'); }, 30000);
    setTimeout(function() { domSnapshot('60s'); }, 60000);

    // === Resource audit at 6s ===
    setTimeout(function() {
        var perfEntries = [];
        if (window.performance && window.performance.getEntriesByType) {
            perfEntries = window.performance.getEntriesByType('resource').map(function(e) {
                return { name: e.name, initiator: e.initiatorType, duration: Math.round(e.duration), size: e.transferSize || 0 };
            }).filter(function(e) { return e.duration > 500 || (e.size === 0 && e.name.indexOf('data:') !== 0); });
        }
        if (perfEntries.length > 0) report({ type: 'slow_resources', entries: perfEntries });
    }, 6000);
})();
"##;

/// Automated UI traversal script. Crawls the entire app, clicks all interactive
/// elements, fills forms, navigates routes, and captures all errors/state.
/// Injected when TACHYON_DEBUG=2.
static DEBUG_TRAVERSE_JS: &str = r##"
(function() {
    function report(data) {
        if (window.__TAURI__) {
            var invoke = window.__TAURI__.core ? window.__TAURI__.core.invoke : (window.__TAURI__.invoke || null);
            if (invoke) { try { invoke('debug_report', { data: data }).catch(function(){}); } catch(e){} }
        }
    }
    function delay(ms) { return new Promise(function(r) { setTimeout(r, ms); }); }

    // === Traversal engine ===
    var visitedUrls = {};
    var clickLog = [];
    var errorLog = [];
    var formLog = [];
    var navigationLog = [];
    var startTime = Date.now();

    function elInfo(el) {
        var r = el.getBoundingClientRect();
        return { tag: el.tagName, id: el.id, cls: (el.className||'').toString().substring(0,100), text: el.textContent.substring(0,60), href: el.href || '', type: el.type || '', name: el.name || '', value: el.value ? String(el.value).substring(0,40) : '', placeholder: el.placeholder || '', rect: { x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height) }, visible: r.width > 0 && r.height > 0, disabled: !!el.disabled };
    }

    // Collect all interactive elements
    function getInteractive() {
        var els = [];
        document.querySelectorAll('a[href], button, input, select, textarea, [role="button"], [tabindex], [onclick]').forEach(function(el) {
            var r = el.getBoundingClientRect();
            if (r.width > 0 && r.height > 0 && !el.disabled) els.push(el);
        });
        return els;
    }

    // Click an element and capture results
    async function clickElement(el) {
        var info = elInfo(el);
        var snapshot = document.body ? document.body.innerHTML.length : 0;
        try {
            el.click();
            el.focus();
            await delay(500);
            var newSnapshot = document.body ? document.body.innerHTML.length : 0;
            var changed = newSnapshot !== snapshot;
            clickLog.push({ action: 'click', el: info, domChanged: changed, timestamp: Date.now() });
            // Check for new elements after click
            var newEls = getInteractive().filter(function(e) { return !visitedUrls[elInfo(e).text + elInfo(e).href]; });
            return { success: true, newElements: newEls.length };
        } catch(e) {
            errorLog.push({ action: 'click', el: info, error: String(e.message || e) });
            return { success: false, error: String(e.message || e) };
        }
    }

    // Fill a form input
    async function fillInput(el) {
        var info = elInfo(el);
        var testValues = {
            'text': 'test-input', 'search': 'knowledge', 'password': 'TestPass123!',
            'email': 'test@example.com', 'number': '42', 'tel': '555-1234',
            'url': 'https://example.com', 'date': '2026-01-01'
        };
        var val = testValues[el.type] || testValues['text'];
        try {
            // Use native input setter to trigger React/Leptos change events
            var nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
            nativeSetter.call(el, val);
            el.dispatchEvent(new Event('input', { bubbles: true }));
            el.dispatchEvent(new Event('change', { bubbles: true }));
            formLog.push({ action: 'fill', el: info, value: val });
            await delay(200);
        } catch(e) {
            errorLog.push({ action: 'fill', el: info, error: String(e.message || e) });
        }
    }

    // Navigate to a URL
    async function navigateTo(url) {
        if (visitedUrls[url]) return false;
        visitedUrls[url] = true;
        navigationLog.push({ url: url, timestamp: Date.now() });
        try {
            // Try clicking a link with that href first
            var link = document.querySelector('a[href="' + url + '"]');
            if (link) {
                link.click();
            } else {
                // Navigate via location
                window.location.href = url;
            }
            await delay(1500);
            return true;
        } catch(e) {
            errorLog.push({ action: 'navigate', url: url, error: String(e.message || e) });
            return false;
        }
    }

    // Main traversal
    async function traverse() {
        report({ type: 'traverse_start', timestamp: Date.now(), url: location.href });

        // Phase 1: Document initial state
        var interactive = getInteractive();
        report({ type: 'traverse_state', url: location.href, interactiveCount: interactive.length, buttons: interactive.filter(function(e){ return e.tagName==='BUTTON' || e.getAttribute('role')==='button'; }).length, links: interactive.filter(function(e){ return e.tagName==='A'; }).length, inputs: interactive.filter(function(e){ return ['INPUT','SELECT','TEXTAREA'].indexOf(e.tagName) >= 0; }).length });

        // Phase 2: Collect all navigation links
        var links = [];
        interactive.forEach(function(el) {
            if (el.tagName === 'A' && el.href && el.href.indexOf('http') !== 0 && el.href.indexOf('#') !== 0) {
                links.push(el.href);
            }
        });
        links = links.filter(function(v, i, a) { return a.indexOf(v) === i; }); // unique
        report({ type: 'traverse_links', links: links, count: links.length });

        // Phase 3: Click all buttons on current page
        var buttons = interactive.filter(function(e){ return e.tagName==='BUTTON' || e.getAttribute('role')==='button' || e.getAttribute('tabindex') !== null; });
        report({ type: 'traverse_buttons', count: buttons.length });
        for (var i = 0; i < buttons.length; i++) {
            await clickElement(buttons[i]);
            await delay(300);
        }

        // Phase 4: Fill all visible form inputs
        var inputs = interactive.filter(function(e){ return ['INPUT','SELECT','TEXTAREA'].indexOf(e.tagName) >= 0; });
        report({ type: 'traverse_inputs', count: inputs.length });
        for (var i = 0; i < inputs.length; i++) {
            await fillInput(inputs[i]);
            await delay(200);
        }

        // Phase 5: Click form submit buttons (after filling)
        var submitButtons = Array.from(document.querySelectorAll('button[type="submit"], input[type="submit"]'));
        for (var i = 0; i < submitButtons.length; i++) {
            var r = submitButtons[i].getBoundingClientRect();
            if (r.width > 0 && r.height > 0) {
                await clickElement(submitButtons[i]);
                await delay(500);
            }
        }

        // Phase 6: Navigate to each link and repeat
        for (var i = 0; i < Math.min(links.length, 20); i++) {
            report({ type: 'traverse_navigate', target: links[i], index: i, total: links.length });
            await navigateTo(links[i]);
            await delay(1000);

            // Document new page state
            var newInteractive = getInteractive();
            report({ type: 'traverse_page_state', url: location.href, interactiveCount: newInteractive.length });

            // Click buttons on new page
            var newButtons = newInteractive.filter(function(e){ return e.tagName==='BUTTON' || e.getAttribute('role')==='button'; });
            for (var j = 0; j < newButtons.length; j++) {
                await clickElement(newButtons[j]);
                await delay(300);
            }

            // Fill inputs on new page
            var newInputs = newInteractive.filter(function(e){ return ['INPUT','SELECT','TEXTAREA'].indexOf(e.tagName) >= 0; });
            for (var j = 0; j < newInputs.length; j++) {
                await fillInput(newInputs[j]);
                await delay(200);
            }

            // Go back to home
            try {
                var homeLink = document.querySelector('a[href="/"], a[href="#/"]');
                if (homeLink) { homeLink.click(); await delay(1000); }
                else { window.location.href = '/'; await delay(1000); }
            } catch(e) {}
        }

        // Phase 7: Final report
        var elapsed = Date.now() - startTime;
        report({
            type: 'traverse_complete',
            elapsed: elapsed,
            pagesVisited: Object.keys(visitedUrls).length,
            linksExplored: links.length,
            totalClicks: clickLog.length,
            totalErrors: errorLog.length,
            totalFormActions: formLog.length,
            totalNavigations: navigationLog.length,
            clickLog: clickLog,
            errorLog: errorLog,
            formLog: formLog,
            navigationLog: navigationLog
        });

        // Keep watching for errors after traversal
        setInterval(function() {
            if (errorLog.length > 0) {
                report({ type: 'traverse_late_errors', count: errorLog.length, errors: errorLog.slice(-10) });
                errorLog.length = 0; // flush
            }
        }, 5000);
    }

    // Start traversal after app has mounted (4s to let WASM init + mount)
    setTimeout(traverse, 4000);
})();
"##;

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fix_webkit_dmabuf_on_nvidia();

    // Check TACHYON_DEBUG env var for debug mode:
    //   TACHYON_DEBUG=0 or unset → no debug injection
    //   TACHYON_DEBUG=1 → hooks + DOM snapshots
    //   TACHYON_DEBUG=2 → hooks + DOM + automated traversal
    let debug_level = std::env::var("TACHYON_DEBUG")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0);

    if debug_level >= 1 {
        tracing::info!("TACHYON_DEBUG={} — debug hooks enabled, traversal={}", debug_level, debug_level >= 2);
    }

    let embedded_server = Arc::new(Mutex::new(EmbeddedServerState::default()));
    let dl = debug_level;

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::default().build());

    // Build the command handler list — include debug_report when debug is on
    if debug_level >= 1 {
        builder = builder.invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::set_server_url,
            commands::authenticate,
            commands::logout,
            commands::is_authenticated,
            commands::is_connected,
            commands::has_repository,
            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::read_file,
            commands::write_file,
            commands::file_exists,
            commands::delete_file,
            commands::create_directory,
            commands::set_repository_path,
            commands::initialize_repository,
            commands::commit_pending,
            commands::push_to_remote,
            commands::pull_from_remote,
            commands::get_sync_status,
            commands::get_queue_size,
            commands::clear_queue,
            commands::queue_file_change,
            commands::enable_auto_sync,
            commands::disable_auto_sync,
            commands::start_file_watcher,
            commands::stop_file_watcher,
            commands::is_file_watching,
            commands::show_error_dialog,
            commands::show_warning_dialog,
            commands::show_info_dialog,
            import_export::import_obsidian_vault,
            import_export::import_markdown_zip,
            import_export::export_html,
            commands::get_embedded_server_port,
            commands::start_embedded_server,
            commands::stop_embedded_server,
            commands::get_local_db_stats,
            commands::init_local_database,
            commands::get_local_tags,
            commands::search_local_documents,
            commands::sync_enqueue,
            commands::sync_queue_summary,
            commands::sync_queue_pending,
            commands::sync_mark_synced,
            commands::sync_mark_failed,
            commands::sync_purge_synced,
            commands::set_connection_status,
            commands::is_online,
            commands::authenticate_offline,
            commands::read_vault,
            commands::read_markdown_file,
            commands::write_markdown_file,
            commands::list_vault_files,
            commands::watch_directory,
            commands::stop_directory_watch,
            commands::is_directory_watched,
            commands::get_app_data_dir,
            commands::open_path,
            // Debug
            commands::debug_report,
            // API proxy (bypasses WebView CORS for tauri:// origin)
            commands::api_proxy,
        ]);
    } else {
        builder = builder.invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::set_server_url,
            commands::authenticate,
            commands::logout,
            commands::is_authenticated,
            commands::is_connected,
            commands::has_repository,
            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::read_file,
            commands::write_file,
            commands::file_exists,
            commands::delete_file,
            commands::create_directory,
            commands::set_repository_path,
            commands::initialize_repository,
            commands::commit_pending,
            commands::push_to_remote,
            commands::pull_from_remote,
            commands::get_sync_status,
            commands::get_queue_size,
            commands::clear_queue,
            commands::queue_file_change,
            commands::enable_auto_sync,
            commands::disable_auto_sync,
            commands::start_file_watcher,
            commands::stop_file_watcher,
            commands::is_file_watching,
            commands::show_error_dialog,
            commands::show_warning_dialog,
            commands::show_info_dialog,
            import_export::import_obsidian_vault,
            import_export::import_markdown_zip,
            import_export::export_html,
            commands::get_embedded_server_port,
            commands::start_embedded_server,
            commands::stop_embedded_server,
            commands::get_local_db_stats,
            commands::init_local_database,
            commands::get_local_tags,
            commands::search_local_documents,
            commands::sync_enqueue,
            commands::sync_queue_summary,
            commands::sync_queue_pending,
            commands::sync_mark_synced,
            commands::sync_mark_failed,
            commands::sync_purge_synced,
            commands::set_connection_status,
            commands::is_online,
            commands::authenticate_offline,
            commands::read_vault,
            commands::read_markdown_file,
            commands::write_markdown_file,
            commands::list_vault_files,
            commands::watch_directory,
            commands::stop_directory_watch,
            commands::is_directory_watched,
            commands::get_app_data_dir,
            commands::open_path,
            // API proxy (bypasses WebView CORS for tauri:// origin)
            commands::api_proxy,
        ]);
    }

    builder = builder.manage(embedded_server);

    // Inject API base URL + debug JS on every page load.
    // The API URL must be set BEFORE the Leptos WASM App initializes.
    builder = builder.on_page_load(move |webview, payload| {
        use std::io::Write;
        let url = payload.url();
        tracing::info!("[setup] on_page_load: {}", url);

        // Set API URL so ApiClient::default() picks it up via window.tachyonApiUrl
        let api_url = std::env::var("TACHYON_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080/api/v1".to_string());
        if let Err(e) = webview.eval(&format!("window.tachyonApiUrl = \"{}\";", api_url)) {
            tracing::warn!("[setup] failed to set API URL: {}", e);
        }

        // Debug hooks and traversal (only when TACHYON_DEBUG >= 1)
        if debug_level >= 1 {
            let _ = std::fs::remove_file("/tmp/tachyon-debug.jsonl");
            if let Ok(mut f) = std::fs::File::create("/tmp/tachyon-debug.jsonl") {
                let _ = writeln!(f, "{{\"type\":\"page_loaded\",\"url\":\"{}\",\"debug_level\":{}}}", url, dl);
            }

            if let Err(e) = webview.eval(DEBUG_HOOKS_JS) {
                tracing::error!("[debug] hooks eval failed: {}", e);
                if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open("/tmp/tachyon-debug.jsonl") {
                    let _ = writeln!(f, "{{\"type\":\"eval_error\",\"msg\":\"{}\"}}", e);
                }
            }

            if dl >= 2 {
                if let Err(e) = webview.eval(DEBUG_TRAVERSE_JS) {
                    tracing::error!("[debug] traversal eval failed: {}", e);
                    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open("/tmp/tachyon-debug.jsonl") {
                        let _ = writeln!(f, "{{\"type\":\"traverse_eval_error\",\"msg\":\"{}\"}}", e);
                    }
                }
            }
        }
    });

    builder.setup(move |app| {
        // Initialize state manager
        let state_manager = DesktopStateManager::new(DesktopState::default());
        let sync_manager = AutoSyncManager::new(SyncConfig::default());
        let app_state = DesktopAppState::new();

        app.manage(state_manager);
        app.manage(sync_manager);
        app.manage(app_state);

        // Set up system tray (only when tray-icon feature is enabled)
        #[cfg(feature = "tray-icon")]
        if let Err(e) = tray::setup_tray(app) {
            tracing::warn!("Failed to set up system tray: {}", e);
        }

        Ok(())
    })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_display() {
        assert_eq!(format!("{}", ConnectionStatus::Connected), "Connected");
        assert_eq!(
            format!("{}", ConnectionStatus::Disconnected),
            "Disconnected"
        );
        assert_eq!(format!("{}", ConnectionStatus::Connecting), "Connecting");
        assert_eq!(format!("{}", ConnectionStatus::Error), "Error");
    }
}
