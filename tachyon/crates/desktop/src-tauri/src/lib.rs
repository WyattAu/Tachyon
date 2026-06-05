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

/// Workaround for WebKitGTK rendering failures on NVIDIA + Wayland.
///
/// Two issues on NVIDIA + Wayland compositors (KWin/Sway/etc.):
///
/// 1. **DMA-BUF GBM buffer failure**: WebKitGTK's DMA-BUF renderer fails with
///    "Failed to create GBM buffer of size NxM: Invalid argument" because
///    NVIDIA's EGL/GBM doesn't support the buffer modifiers WebKit requests.
///    Fix: `WEBKIT_DISABLE_DMABUF_RENDERER=1` → shared-memory (shm) fallback.
///
/// 2. **Wayland protocol error**: WebKitGTK crashes with
///    "Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display"
///    when the compositor's Wayland protocol implementation is incompatible.
///    Fix: `GDK_BACKEND=x11` → use XWayland bridge instead of native Wayland.
///
/// These only set env vars if not already explicitly configured, so users can
/// override (e.g., `GDK_BACKEND=wayland WEBKIT_DISABLE_DMABUF_RENDERER=0`).
fn fix_webkit_dmabuf_on_nvidia() {
    // Detect NVIDIA GPU
    if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
        let has_nvidia = std::fs::read_dir("/dev/dri/by-path")
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .contains("nvidia")
                })
            })
            .unwrap_or(false);

        let has_nvidia_proc =
            std::path::Path::new("/proc/driver/nvidia").exists();

        if has_nvidia || has_nvidia_proc {
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
            tracing::info!(
                "NVIDIA GPU detected — set WEBKIT_DISABLE_DMABUF_RENDERER=1"
            );
        }
    }

    // Fix Wayland protocol errors by forcing X11 backend via XWayland.
    // This avoids "Error 71 (Protocol error)" crashes that occur with
    // various Wayland compositors when WebKitGTK sends unsupported
    // protocol messages (e.g., fractional-scale-v1, linux-dmabuf-v1).
    if std::env::var("GDK_BACKEND").is_err() {
        // Only force X11 if we're actually on a Wayland session
        let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
        let x11_display = std::env::var("DISPLAY").is_ok();

        if wayland_display {
            unsafe {
                std::env::set_var("GDK_BACKEND", "x11");
            }
            tracing::info!(
                "Wayland session detected — set GDK_BACKEND=x11 to avoid \
                 WebKitGTK protocol errors (XWayland bridge will be used)"
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
// v2: added skipCount, allTextDump without getComputedStyle
(function() {
    var invoke = null;
    try {
        if (window.__TAURI__) {
            invoke = window.__TAURI__.core && window.__TAURI__.core.invoke
                ? window.__TAURI__.core.invoke.bind(window.__TAURI__.core)
                : (window.__TAURI__.invoke ? window.__TAURI__.invoke.bind(window.__TAURI__) : null);
        }
    } catch(e) { console.error('invoke setup failed: ' + String(e)); }

    // === Buffered report: collects entries and flushes in batches to avoid IPC storm ===
    var _buf = [];
    var _flushTimer = null;
    var _flushScheduled = false;
    var MAX_BUFFER = 200;
    var FLUSH_INTERVAL = 500;

    function scheduleFlush() {
        if (_flushScheduled) return;
        _flushScheduled = true;
        _flushTimer = setTimeout(doFlush, FLUSH_INTERVAL);
    }

    function doFlush() {
        _flushScheduled = false;
        if (_buf.length === 0) return;
        var batch = _buf.splice(0, MAX_BUFFER);
        if (invoke) {
            try { invoke('debug_report', { data: JSON.stringify(batch) }).catch(function() {}); }
            catch(e) {}
        }
        if (_buf.length > 0) scheduleFlush();
    }

    var CRITICAL_TYPES = {
        js_error: 1, promise_error: 1, script_load_error: 1, wasm_trap: 1,
        invalid_character_error: 1, layout_shift: 1, excessive_rerender: 1,
        content_dump: 1, content_dump_error: 1
    };

    function report(data) {
        _buf.push(data);
        if (CRITICAL_TYPES[data.type]) {
            if (invoke) {
                try { invoke('debug_report', { data: JSON.stringify([data]) }).catch(function() {}); }
                catch(e) {}
            }
            return;
        }
        if (_buf.length >= MAX_BUFFER) doFlush();
        else scheduleFlush();
    }

    // === Error hooks ===
    window.onerror = function(msg, src, line, col, err) {
        report({ type: 'js_error', message: String(msg), source: src || '', line: line || 0, col: col || 0, stack: err ? (err.stack || '') : '' });
    };
    window.addEventListener('unhandledrejection', function(e) {
        var reason = e.reason;
        var stack = '';
        var reasonStr = '';
        if (reason instanceof Error) {
            reasonStr = reason.message || String(reason);
            stack = reason.stack || '';
        } else {
            reasonStr = String(reason);
            try { stack = reason && reason.stack ? reason.stack : ''; } catch(_) {}
        }
        var wasmInfo = null;
        try {
            var msg = String(reasonStr);
            if (msg.indexOf('unreachable') !== -1 || msg.indexOf('call stack exhausted') !== -1 || msg.indexOf('RuntimeError') !== -1) {
                wasmInfo = detectWasmTrap(msg, stack);
            }
        } catch(_) {}
        report({ type: 'promise_error', reason: reasonStr, stack: stack, wasm: wasmInfo });
    });
    window.addEventListener('error', function(e) {
        if (e.target) {
            var tag = e.target.tagName;
            if (tag === 'SCRIPT') {
                report({ type: 'script_load_error', src: e.target.src || '', typeAttr: e.target.type || '' });
            } else if (tag === 'LINK') {
                report({ type: 'link_load_error', href: e.target.href || '', rel: e.target.rel || '' });
            } else if (tag === 'IMG') {
                report({ type: 'img_load_error', src: e.target.src || '', alt: e.target.alt || '' });
            } else if (tag === 'VIDEO') {
                report({ type: 'video_load_error', src: e.target.src || '', poster: e.target.poster || '' });
            } else if (tag === 'AUDIO') {
                report({ type: 'audio_load_error', src: e.target.src || '' });
            }
        }
    }, true);

    // === WASM trap detection ===
    var wasmTrapsSeen = {};
    function detectWasmTrap(msg, stack) {
        var trapType = 'unknown';
        if (msg.indexOf('unreachable') !== -1) trapType = 'unreachable';
        else if (msg.indexOf('call stack exhausted') !== -1) trapType = 'stack_exhausted';
        var key = trapType + ':' + (stack || '').substring(0, 80);
        if (wasmTrapsSeen[key]) return null;
        wasmTrapsSeen[key] = true;
        return { trapType: trapType, message: msg, stack: stack || '', timestamp: Date.now() };
    }
    var origWasmInstantiate = WebAssembly.instantiate;
    WebAssembly.instantiate = function() {
        return origWasmInstantiate.apply(this, arguments).catch(function(err) {
            if (err && (err.name === 'CompileError' || err.name === 'LinkError')) {
                report({ type: 'wasm_instantiate_error', error: String(err.message || err), name: err.name });
            }
            throw err;
        });
    };
    var origWasmInstantiateStreaming = WebAssembly.instantiateStreaming;
    if (origWasmInstantiateStreaming) {
        WebAssembly.instantiateStreaming = function() {
            return origWasmInstantiateStreaming.apply(this, arguments).catch(function(err) {
                report({ type: 'wasm_instantiate_error', error: String(err.message || err), name: err.name, streaming: true });
                throw err;
            });
        };
    }
    var _origConsoleErr = console.error;
    console.error = (function(orig) {
        return function() {
            var args = Array.prototype.slice.call(arguments);
            var msg = args.join(' ');
            orig.apply(console, arguments);
            if (msg.indexOf('RuntimeError') !== -1) {
                var trapInfo = detectWasmTrap(msg, '');
                if (trapInfo) report({ type: 'wasm_trap', subtype: trapInfo.trapType, message: msg });
            }
            report({ type: 'console_error', message: msg });
        };
    })(_origConsoleErr);

    // === Console hooks (error already replaced above for wasm detection) ===
    var origWarn = console.warn, origInfo = console.info, origLog = console.log;
    console.warn = function() { var m = Array.prototype.slice.call(arguments).join(' '); origWarn.apply(console, arguments); report({ type: 'console_warn', message: m }); };
    console.info = function() { var m = Array.prototype.slice.call(arguments).join(' '); origInfo.apply(console, arguments); report({ type: 'console_info', message: m }); };
    console.log  = function() { var m = Array.prototype.slice.call(arguments).join(' '); origLog.apply(console, arguments); report({ type: 'console_log', message: m }); };

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
        var url = typeof input === 'string' ? input : (input && input.url ? input.url : String(input));
        if (url.indexOf('ipc://') === 0 || url.indexOf('tauri:') === 0) {
            return origFetch.apply(this, arguments);
        }
        var start = Date.now();
        return origFetch.apply(this, arguments).then(function(resp) {
            report({ type: 'fetch_response', url: url, method: init && init.method ? init.method : 'GET', status: resp.status, ok: resp.ok, duration: Date.now() - start });
            return resp;
        }).catch(function(err) {
            report({ type: 'fetch_error', url: url, error: String(err.message || err), duration: Date.now() - start });
            throw err;
        });
    };

    // === Layout shift detection via ResizeObserver on #app ===
    var _layoutShiftThrottle = null;
    var _layoutShiftCount = 0;
    var _layoutShiftWindowStart = 0;
    function observeLayoutShifts() {
        var app = document.getElementById('app');
        if (!app || typeof ResizeObserver === 'undefined') return;
        var lastW = app.offsetWidth || 0, lastH = app.offsetHeight || 0;
        var ro = new ResizeObserver(function(entries) {
            var now = Date.now();
            var entry = entries[0];
            if (!entry || !entry.contentRect) return;
            var w = entry.contentRect.width, h = entry.contentRect.height;
            if (w === lastW && h === lastH) return;
            _layoutShiftCount++;
            if (_layoutShiftCount === 1) _layoutShiftWindowStart = now;
            if (_layoutShiftCount <= 10) {
                report({ type: 'layout_shift', width: Math.round(w), height: Math.round(h), prevWidth: Math.round(lastW), prevHeight: Math.round(h), count: _layoutShiftCount });
            }
            if (_layoutShiftCount > 10 && _layoutShiftCount % 50 === 0) {
                report({ type: 'layout_shift', width: Math.round(w), height: Math.round(h), count: _layoutShiftCount, burst: true });
            }
            if (_layoutShiftCount >= 50 && (now - _layoutShiftWindowStart) < 1000 && _layoutShiftCount > 49) {
                report({ type: 'excessive_rerender', source: 'resize_observer', count: _layoutShiftCount, windowMs: now - _layoutShiftWindowStart });
                _layoutShiftCount = 0;
                _layoutShiftWindowStart = 0;
            } else if ((now - _layoutShiftWindowStart) >= 1000) {
                _layoutShiftCount = 0;
                _layoutShiftWindowStart = 0;
            }
            lastW = w; lastH = h;
        });
        ro.observe(app);
    }
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', observeLayoutShifts);
    } else {
        observeLayoutShifts();
    }

    // === MutationObserver: detect excessive Leptos re-renders ===
    var _mutCount = 0;
    var _mutWindowStart = 0;
    var _rerenderWarned = {};
    function startMutationObserver() {
        var app = document.getElementById('app');
        if (!app) { document.body && startMutationObserverOn(document.body); return; }
        startMutationObserverOn(app);
    }
    function startMutationObserverOn(target) {
        if (!target || typeof MutationObserver === 'undefined') return;
        var mo = new MutationObserver(function(mutations) {
            var now = Date.now();
            _mutCount += mutations.length;
            if (_mutWindowStart === 0) _mutWindowStart = now;
            if (_mutCount > 50 && (now - _mutWindowStart) < 1000) {
                var key = Math.floor(now / 2000);
                if (!_rerenderWarned[key]) {
                    _rerenderWarned[key] = true;
                    var added = [], removed = [];
                    for (var i = 0, len = Math.min(mutations.length, 10); i < len; i++) {
                        var m = mutations[i];
                        if (m.type === 'childList') {
                            added.push(m.addedNodes.length);
                            removed.push(m.removedNodes.length);
                        } else if (m.type === 'attributes') {
                            added.push(0);
                            removed.push(0);
                        }
                    }
                    report({ type: 'excessive_rerender', source: 'mutation_observer', count: _mutCount, windowMs: now - _mutWindowStart, addedNodes: added.slice(0, 5), removedNodes: removed.slice(0, 5), sampleMutationTypes: mutations.slice(0, 5).map(function(m) { return m.type + ':' + (m.target && m.target.tagName ? m.target.tagName.toLowerCase() : '?'); }) });
                }
                _mutCount = 0;
                _mutWindowStart = 0;
            } else if ((now - _mutWindowStart) >= 1000) {
                _mutCount = 0;
                _mutWindowStart = 0;
            }
        });
        mo.observe(target, { childList: true, subtree: true, attributes: true, attributeFilter: ['class', 'style', 'data-leptos-'] });
    }
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', startMutationObserver);
    } else {
        startMutationObserver();
    }

    // === Accessibility audit at 5s ===
    setTimeout(function() {
        var interactiveElements = document.querySelectorAll('button, a[href], input, select, textarea, [role="button"], [role="link"], [role="tab"], [role="menuitem"], [tabindex]');
        var missingLabels = [];
        for (var i = 0; i < interactiveElements.length; i++) {
            var el = interactiveElements[i];
            var hasLabel = false;
            if (el.getAttribute('aria-label')) hasLabel = true;
            else if (el.getAttribute('aria-labelledby')) hasLabel = true;
            else if (el.id && document.querySelector('label[for="' + el.id + '"]')) hasLabel = true;
            else if (el.tagName === 'INPUT' && el.type === 'hidden') continue;
            else if (el.tagName === 'INPUT' && el.type === 'submit') hasLabel = true;
            else if (el.tagName === 'BUTTON' && el.textContent && el.textContent.trim().length > 0) hasLabel = true;
            else if (el.tagName === 'A' && el.textContent && el.textContent.trim().length > 0) hasLabel = true;
            if (!hasLabel) {
                missingLabels.push({ tag: el.tagName, id: el.id || '', role: el.getAttribute('role') || '', type: el.type || '', tabindex: el.tabIndex, classes: (el.className || '').substring(0, 60) });
            }
        }
        var invalidRoles = [];
        var allRoleEls = document.querySelectorAll('[role]');
        var validRoles = ['alert', 'alertdialog', 'application', 'article', 'banner', 'button', 'cell', 'checkbox', 'columnheader', 'combobox', 'complementary', 'contentinfo', 'definition', 'dialog', 'directory', 'document', 'feed', 'figure', 'form', 'grid', 'gridcell', 'group', 'heading', 'img', 'link', 'list', 'listbox', 'listitem', 'log', 'main', 'marquee', 'math', 'menu', 'menubar', 'menuitem', 'menuitemcheckbox', 'menuitemradio', 'navigation', 'note', 'option', 'presentation', 'progressbar', 'radio', 'radiogroup', 'range', 'region', 'row', 'rowgroup', 'rowheader', 'scrollbar', 'search', 'searchbox', 'separator', 'slider', 'spinbutton', 'status', 'switch', 'tab', 'tablist', 'tabpanel', 'textbox', 'timer', 'toolbar', 'tooltip', 'tree', 'treegrid', 'treeitem'];
        for (var i = 0; i < allRoleEls.length; i++) {
            var role = allRoleEls[i].getAttribute('role');
            if (role && validRoles.indexOf(role) === -1) {
                invalidRoles.push({ tag: allRoleEls[i].tagName, id: allRoleEls[i].id || '', role: role });
            }
        }
        var imgsWithoutAlt = [];
        var imgs = document.querySelectorAll('img');
        for (var i = 0; i < imgs.length; i++) {
            if (!imgs[i].hasAttribute('alt') && !imgs[i].getAttribute('role')) {
                imgsWithoutAlt.push({ src: (imgs[i].src || '').substring(0, 100), id: imgs[i].id || '' });
            }
        }
        if (missingLabels.length > 0 || invalidRoles.length > 0 || imgsWithoutAlt.length > 0) {
            report({ type: 'a11y_audit', interactiveCount: interactiveElements.length, missingLabels: missingLabels.slice(0, 20), invalidRoles: invalidRoles.slice(0, 20), imgsWithoutAlt: imgsWithoutAlt.slice(0, 10) });
        }
    }, 5000);

    // === InvalidCharacterError detection for classList multi-word bug ===
    var _iceSeen = {};
    function wrapClassListProto(proto) {
        if (!proto) return;
        var methods = ['add', 'remove', 'toggle', 'contains'];
        for (var mi = 0; mi < methods.length; mi++) {
            (function(methodName) {
                var orig = proto[methodName];
                if (typeof orig !== 'function') return;
                proto[methodName] = function() {
                    try { return orig.apply(this, arguments); }
                    catch(e) {
                        if (e && e.name === 'InvalidCharacterError') {
                            var args = [];
                            for (var ai = 0; ai < arguments.length; ai++) args.push(String(arguments[ai]));
                            var key = methodName + ':' + args.join(',');
                            if (!_iceSeen[key]) {
                                _iceSeen[key] = true;
                                var el = this && this.ownerElement ? this.ownerElement : (this && this.tagName ? this : null);
                                report({ type: 'invalid_character_error', method: methodName, args: args, elementTag: el ? el.tagName : '?', elementId: el ? (el.id || '') : '', message: String(e.message || e) });
                            }
                        }
                        throw e;
                    }
                };
            })(methods[mi]);
        }
    }
    wrapClassListProto(DOMTokenList.prototype);

    // === Signal loaded ===
    report({ type: 'debugger_loaded', url: location.href, timestamp: Date.now(), tauriExists: !!window.__TAURI__, features: ['wasm_trap', 'layout_shift', 'mutation_observer', 'a11y_audit', 'invalid_character_error', 'resource_load'] });

    // === DOM + Style snapshot ===
    function domSnapshot(label) {
        var body = document.body, html = document.documentElement;
        var app = document.getElementById('app');
        var bodyCh = [], appCh = [];
        if (body) for (var i = 0; i < body.children.length; i++) { var c = body.children[i]; bodyCh.push({ tag: c.tagName, id: c.id, cls: (c.className||'').substring(0,100), rect: c.getBoundingClientRect ? { x: Math.round(c.getBoundingClientRect().x), y: Math.round(c.getBoundingClientRect().y), w: Math.round(c.getBoundingClientRect().width), h: Math.round(c.getBoundingClientRect().height) } : null }); }
        if (app) for (var i = 0; i < Math.min(app.children.length, 30); i++) { var c = app.children[i]; appCh.push({ tag: c.tagName, id: c.id, text: c.textContent.substring(0,80) }); }
        var s_app = app ? window.getComputedStyle(app) : null;
        var s_body = body ? window.getComputedStyle(body) : null;
        // Capture visible text and HTML snippet for debugging
        var visText = [], htmlSnippet = '', allTextDump = [];
        if (app) htmlSnippet = app.innerHTML.substring(0, 2000);
        var allEls = document.querySelectorAll('*');
        var skipCount = 0;
        for (var vi2 = 0; vi2 < allEls.length; vi2++) {
            var ve = allEls[vi2];
            if (ve.tagName === 'SCRIPT' || ve.tagName === 'STYLE' || ve.tagName === 'SVG') { skipCount++; continue; }
            var vtxt = ve.textContent.trim().substring(0, 40);
            if (allTextDump.length < 50) allTextDump.push({ t: ve.tagName, tx: vtxt });
            if (visText.length < 25 && vtxt.length > 0) visText.push({ t: ve.tagName, id: ve.id||'', tx: vtxt });
        }
        report({
            type: 'dom_' + label, url: location.href, title: document.title, readyState: document.readyState,
            totalElements: allEls.length,
            styledElements: document.querySelectorAll('[class]').length,
            bodyChildren: bodyCh, appExists: !!app,
            appStyle: s_app ? { display: s_app.display, height: s_app.height, width: s_app.width, bg: s_app.backgroundColor } : null,
            bodyStyle: s_body ? { height: s_body.height, width: s_body.width, bg: s_body.backgroundColor } : null,
            hiddenCount: document.querySelectorAll('[style*="display:none"],[style*="visibility:hidden"],[hidden]').length,
            visText: visText, htmlSnippet: htmlSnippet, allTextDump: allTextDump, skipCount: skipCount,
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

    // Final flush after 2 seconds to catch everything
    setTimeout(doFlush, 2000);
})();
"##;

/// Automated UI traversal script. Crawls the entire app: auto-login, sidebar
/// navigation, interactive element testing, form filling, and continuous
/// error/health monitoring. Injected when TACHYON_DEBUG=2.
static DEBUG_TRAVERSE_JS: &str = r##"
(async function() {
    var MAX_MS = 120000;
    var startTime = Date.now();
    var errorLog = [];
    var sessionErrors = [];
    var pagesVisited = {};

    function elapsed() { return Date.now() - startTime; }
    function timeout() { return elapsed() >= MAX_MS; }

    function report(data) {
        if (timeout()) return;
        try {
            var invoke = window.__TAURI__ && (window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke);
            if (invoke) { invoke('debug_report', { data: data }).catch(function(){}); }
        } catch(e) {}
    }

    function logError(cat, msg, extra) {
        var entry = { category: cat, message: msg, url: location.href, time: elapsed(), page: document.title };
        if (extra) { for (var k in extra) { if (extra.hasOwnProperty(k)) entry[k] = extra[k]; } }
        errorLog.push(entry);
        sessionErrors.push(entry);
    }

    function delay(ms) {
        if (timeout()) return Promise.resolve();
        return new Promise(function(r) { setTimeout(r, Math.min(ms, MAX_MS - elapsed() > 0 ? MAX_MS - elapsed() : 0)); });
    }

    function visible(el) {
        var r = el.getBoundingClientRect();
        return r.width > 0 && r.height > 0 && getComputedStyle(el).visibility !== 'hidden';
    }

    function shortText(el) { return (el.textContent || '').replace(/\s+/g, ' ').trim().substring(0, 80); }
    var _nativeInputSetter = null;
    try { _nativeInputSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set; } catch(e) {}
    var _nativeTextareaSetter = null;
    try { _nativeTextareaSetter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value').set; } catch(e) {}
    function nativeInputSetter(el, val) {
        var setter = el.tagName === 'TEXTAREA' ? _nativeTextareaSetter : _nativeInputSetter;
        if (setter) { setter.call(el, val); } else { el.value = val; }
        el.dispatchEvent(new Event('input', { bubbles: true }));
        el.dispatchEvent(new Event('change', { bubbles: true }));
    }

    function countByTag(root) {
        root = root || document;
        var counts = { button: 0, link: 0, input: 0, select: 0, textarea: 0, total: 0 };
        var els = root.querySelectorAll('button, a[href], input, select, textarea, [role="button"], [tabindex]');
        for (var i = 0; i < els.length; i++) {
            if (!visible(els[i])) continue;
            counts.total++;
            var t = els[i].tagName.toLowerCase();
            var role = els[i].getAttribute('role') || '';
            if (t === 'button' || role === 'button') counts.button++;
            else if (t === 'a') counts.link++;
            else if (t === 'input') counts.input++;
            else if (t === 'select') counts.select++;
            else if (t === 'textarea') counts.textarea++;
        }
        return counts;
    }

    function snapshotPage(label) {
        var counts = countByTag();
        var zeroSize = [];
        var overlaps = [];
        var visEls = [];
        var allVis = document.querySelectorAll('button, a[href], input, select, [role="button"]');
        for (var i = 0; i < allVis.length; i++) {
            var r = allVis[i].getBoundingClientRect();
            if (r.width === 0 || r.height === 0) {
                if (getComputedStyle(allVis[i]).display !== 'none') {
                    zeroSize.push({ tag: allVis[i].tagName, text: shortText(allVis[i]).substring(0, 40) });
                }
            } else if (r.width > 0 && r.height > 0) {
                visEls.push({ r: r, tag: allVis[i].tagName, text: shortText(allVis[i]).substring(0, 40) });
            }
        }
        if (zeroSize.length > 10) zeroSize = zeroSize.slice(0, 10);
        var data = {
            type: 'traverse_' + label,
            url: location.href,
            title: document.title,
            elapsed: elapsed(),
            elements: counts,
            zeroSize: zeroSize,
            domSize: document.querySelectorAll('*').length
        };
        report(data);
        return data;
    }

    function isDestructive(el) {
        var t = shortText(el).toLowerCase();
        return t.indexOf('delete') !== -1 || t.indexOf('remove') !== -1 || t.indexOf('cancel subscription') !== -1 ||
               t.indexOf('drop') !== -1 || t.indexOf('purge') !== -1 || t.indexOf('destroy') !== -1 ||
               t.indexOf('nuke') !== -1 || t.indexOf('erase') !== -1 || t.indexOf('unlink') !== -1 ||
               (el.getAttribute('data-destructive') === 'true');
    }

    async function tryGuestLogin() {
        var guestBtn = null;
        var btns = document.querySelectorAll('button, a, [role="button"]');
        for (var i = 0; i < btns.length; i++) {
            if (visible(btns[i]) && shortText(btns[i]).toLowerCase().indexOf('guest') !== -1) {
                guestBtn = btns[i]; break;
            }
        }
        if (!guestBtn) return false;
        report({ type: 'traverse_guest_attempt' });
        guestBtn.click();
        await delay(3000);
        if (location.href.indexOf('/dashboard') !== -1 || location.href.indexOf('dashboard') !== -1) {
            report({ type: 'traverse_guest_success', url: location.href });
            return true;
        }
        report({ type: 'traverse_guest_failed', url: location.href });
        return false;
    }

    async function tryFormLogin() {
        var user = document.querySelector('input[type="text"], input[name="username"], input[name="email"], input[placeholder*="user" i], input[placeholder*="email" i]');
        var pass = document.querySelector('input[type="password"]');
        var signBtn = null;
        var btns = document.querySelectorAll('button, input[type="submit"]');
        for (var i = 0; i < btns.length; i++) {
            if (visible(btns[i])) {
                var t = shortText(btns[i]).toLowerCase();
                if (t.indexOf('sign in') !== -1 || t.indexOf('login') !== -1 || t.indexOf('log in') !== -1 ||
                    btns[i].getAttribute('type') === 'submit') {
                    signBtn = btns[i]; break;
                }
            }
        }
        if (!user || !pass || !signBtn) {
            report({ type: 'traverse_login_form_not_found', hasUser: !!user, hasPass: !!pass, hasBtn: !!signBtn });
            return false;
        }
        var userVal = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
        userVal.call(user, 'admin');
        user.dispatchEvent(new Event('input', { bubbles: true }));
        user.dispatchEvent(new Event('change', { bubbles: true }));
        await delay(200);
        userVal.call(pass, 'admin');
        pass.dispatchEvent(new Event('input', { bubbles: true }));
        pass.dispatchEvent(new Event('change', { bubbles: true }));
        await delay(200);
        report({ type: 'traverse_form_login_attempt', user: user.name || user.type, btn: shortText(signBtn) });
        signBtn.click();
        await delay(3000);
        if (location.href.indexOf('/dashboard') !== -1 || location.href.indexOf('dashboard') !== -1) {
            report({ type: 'traverse_form_login_success', url: location.href });
            return true;
        }
        report({ type: 'traverse_form_login_failed', url: location.href });
        return false;
    }

     async function phase0_login() {
         report({ type: 'traverse_phase0', msg: 'checking authentication' });
         var avatarEls = document.querySelectorAll('[class*="avatar"], [class*="user-menu"], [class*="profile"], img[alt*="avatar" i]');
         var loggedIn = false;
         for (var i = 0; i < avatarEls.length; i++) {
             if (visible(avatarEls[i])) { loggedIn = true; break; }
         }
         var dashboardEls = document.querySelectorAll('[class*="sidebar"], [class*="nav"], [class*="dashboard"]');
         for (var i = 0; i < dashboardEls.length; i++) {
             if (visible(dashboardEls[i])) { loggedIn = true; break; }
         }
         if (loggedIn) {
             report({ type: 'traverse_phase0', msg: 'already authenticated' });
             return;
         }

         // Try guest login first (simplest path)
         var guestOk = await tryGuestLogin();
         if (guestOk) return;

         // Try form login with testuser credentials
         var formOk = await tryFormLogin();
         if (formOk) return;

         // If not on login page, navigate there first
         if (location.href.indexOf('login') === -1 && location.href.indexOf('register') === -1) {
             // Navigate to /register to create an account
             var registerLink = document.querySelector('a[href*="register"]');
             if (registerLink) {
                 registerLink.click();
                 await delay(2000);
             } else {
                 try { window.location.href = '/register'; await delay(2000); } catch(e) {}
             }
         }

         // Try register: fill username, email, password, confirm, display_name
         var regForm = document.querySelector('form');
         var regInputs = document.querySelectorAll('input');
         if (regInputs.length >= 4) {
             report({ type: 'traverse_register_attempt' });
             var usernameInp = null, emailInp = null, passInp = null, confirmInp = null, nameInp = null;
             for (var i = 0; i < regInputs.length; i++) {
                 var inp = regInputs[i];
                 var nm = (inp.name || '').toLowerCase();
                 var ph = (inp.placeholder || '').toLowerCase();
                 var tp = inp.type || 'text';
                 if (tp === 'text' && (nm.indexOf('user') !== -1 || ph.indexOf('user') !== -1 || ph.indexOf('name') === -1)) usernameInp = inp;
                 if (tp === 'text' && (nm.indexOf('display') !== -1 || ph.indexOf('display') !== -1 || (nm.indexOf('name') !== -1 && !usernameInp))) nameInp = inp;
                 if (tp === 'email' || nm.indexOf('email') !== -1) emailInp = inp;
                 if (tp === 'password' && !passInp) passInp = inp;
                 if (tp === 'password' && passInp) confirmInp = inp;
             }
             if (usernameInp) { nativeInputSetter(usernameInp, 'traverser' + Math.floor(Math.random()*1000)); }
             if (emailInp) { nativeInputSetter(emailInp, 'traverse@test.com'); }
             if (nameInp) { nativeInputSetter(nameInp, 'Traverse Bot'); }
             if (passInp) { nativeInputSetter(passInp, 'testpass123'); }
             if (confirmInp) { nativeInputSetter(confirmInp, 'testpass123'); }

             // Check for terms checkbox
             var termsBox = document.querySelector('input[type="checkbox"]');
             if (termsBox && !termsBox.checked) { termsBox.click(); await delay(200); }

             // Click Create / Register button
             var regBtns = document.querySelectorAll('button');
             for (var i = 0; i < regBtns.length; i++) {
                 var t = shortText(regBtns[i]).toLowerCase();
                 if (t.indexOf('create') !== -1 || t.indexOf('register') !== -1 || t.indexOf('sign up') !== -1 || t.indexOf('submit') !== -1) {
                     regBtns[i].click();
                     await delay(3000);
                     report({ type: 'traverse_register_result', url: location.href, loggedIn: location.href.indexOf('dashboard') !== -1 || location.href.indexOf('login') === -1 });
                     if (location.href.indexOf('dashboard') !== -1 || location.href.indexOf('documents') !== -1) return;
                 }
             }
         }

         logError('auth_error', 'Failed all login methods (guest, form, register)', { url: location.href });
         report({ type: 'traverse_phase0', msg: 'login failed, continuing unauthenticated' });
     }

    function phase1_captureState() {
        report({ type: 'traverse_phase1', msg: 'capturing initial state' });
        var visText = [];
        var allEls = document.querySelectorAll('body > *, #app > *');
        for (var i = 0; i < allEls.length; i++) {
            var el = allEls[i];
            var r = el.getBoundingClientRect();
            if (r.width > 0 && r.height > 0) {
                visText.push({ tag: el.tagName, id: el.id || '', text: shortText(el).substring(0, 60),
                    x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height) });
            }
        }
        if (visText.length > 30) visText = visText.slice(0, 30);
        var counts = countByTag();
        report({ type: 'traverse_initial_state', url: location.href, title: document.title, elements: counts,
            visibleTopElements: visText, domNodes: document.querySelectorAll('*').length });
    }

    function findNavLinks() {
        var links = [];
        var seen = {};
        var candidates = document.querySelectorAll('nav a[href^="/"], [class*="sidebar"] a[href^="/"], [class*="nav"] a[href^="/"], aside a[href^="/"], a[href^="/"]');
        for (var i = 0; i < candidates.length; i++) {
            var el = candidates[i];
            if (!visible(el)) continue;
            var href = el.getAttribute('href') || '';
            if (href === '#' || href === '/' || href === '#/' || href.indexOf('javascript:') === 0) continue;
            if (!seen[href]) {
                seen[href] = true;
                links.push({ href: href, text: shortText(el), el: el });
            }
        }
        return links.slice(0, 15);
    }

    async function phase2_sidebarNav() {
        report({ type: 'traverse_phase2', msg: 'sidebar navigation' });
        var navLinks = findNavLinks();
        report({ type: 'traverse_nav_links', count: navLinks.length,
            links: navLinks.map(function(l) { return { href: l.href, text: l.text }; }) });
        for (var i = 0; i < navLinks.length; i++) {
            if (timeout()) break;
            var link = navLinks[i];
            var href = link.href;
            if (pagesVisited[href]) continue;
            report({ type: 'traverse_nav_click', index: i, href: href, text: link.text });
            link.el.click();
            await delay(1500);
            var authRedirected = location.href.indexOf('/login') !== -1;
            if (authRedirected) {
                logError('auth_error', 'AuthGuard redirected to /login for ' + href, { page: link.text });
                var guestOk = await tryGuestLogin();
                if (!guestOk) {
                    var formOk = await tryFormLogin();
                    if (!formOk) break;
                }
                await delay(1000);
                if (timeout()) break;
                link.el.click();
                await delay(1500);
            }
            pagesVisited[href] = true;
            var pageSnap = snapshotPage('page_' + i + '_' + href.replace(/[^a-zA-Z0-9]/g, '_'));
            var btnsOnPage = [];
            var inputsOnPage = [];
            var visBtns = document.querySelectorAll('button, [role="button"]');
            for (var j = 0; j < visBtns.length; j++) { if (visible(visBtns[j])) btnsOnPage.push(shortText(visBtns[j]).substring(0, 50)); }
            var visInputs = document.querySelectorAll('input, textarea, select');
            for (var j = 0; j < visInputs.length; j++) { if (visible(visInputs[j])) inputsOnPage.push({ tag: visInputs[j].tagName, type: visInputs[j].type || '', name: visInputs[j].name || '', placeholder: visInputs[j].placeholder || '' }); }
            report({ type: 'traverse_page_detail', href: href, buttons: btnsOnPage.slice(0, 20), inputs: inputsOnPage.slice(0, 20), errorsOnPage: sessionErrors.length });
            await phase3_clickElements();
            await phase4_fillForms();
        }
    }

    async function phase3_clickElements() {
        var btns = document.querySelectorAll('button, [role="button"], [onclick], a[href^="#"], [tabindex]');
        var clicked = 0;
        for (var i = 0; i < btns.length; i++) {
            if (timeout()) break;
            var el = btns[i];
            if (!visible(el) || el.disabled) continue;
            if (isDestructive(el)) continue;
            var text = shortText(el).toLowerCase();
            if (text.indexOf('submit') !== -1 || text.indexOf('save') !== -1) continue;
            clicked++;
            report({ type: 'traverse_click', index: clicked, tag: el.tagName, text: shortText(el).substring(0, 50), href: el.href || '' });
            try {
                el.click();
                await delay(500);
            } catch(e) {
                logError('js_error', 'Click failed: ' + (e.message || e), { element: shortText(el).substring(0, 40) });
            }
            var modal = document.querySelector('[role="dialog"], [class*="modal"], [class*="overlay"], [data-modal]');
            if (modal && visible(modal)) {
                report({ type: 'traverse_modal', content: shortText(modal).substring(0, 200) });
                var cancelBtn = null;
                var modalBtns = modal.querySelectorAll('button, [role="button"]');
                for (var j = 0; j < modalBtns.length; j++) {
                    var bt = shortText(modalBtns[j]).toLowerCase();
                    if (bt.indexOf('cancel') !== -1 || bt.indexOf('close') !== -1 || bt.indexOf('dismiss') !== -1 || bt.indexOf('x') !== -1) {
                        cancelBtn = modalBtns[j]; break;
                    }
                }
                if (cancelBtn) {
                    cancelBtn.click();
                    await delay(300);
                } else {
                    var backdrop = document.querySelector('[class*="backdrop"], [class*="overlay"]');
                    if (backdrop) { backdrop.click(); await delay(300); }
                }
            }
            var dropdown = document.querySelector('[class*="dropdown"]:not([style*="display:none"]), [class*="menu"]:not([style*="display:none"]), [role="listbox"]');
            if (dropdown && visible(dropdown) && clicked === 1) {
                var opts = dropdown.querySelectorAll('[role="option"], li, a');
                if (opts.length > 0 && !isDestructive(opts[0])) {
                    opts[0].click();
                    await delay(300);
                }
            }
        }
        var selects = document.querySelectorAll('select');
        for (var i = 0; i < selects.length; i++) {
            if (timeout() || !visible(selects[i])) continue;
            try {
                var sel = selects[i];
                if (sel.options.length > 1) {
                    sel.selectedIndex = 1;
                    sel.dispatchEvent(new Event('change', { bubbles: true }));
                    await delay(300);
                }
            } catch(e) {
                logError('js_error', 'Select change failed: ' + (e.message || e));
            }
        }
    }

    async function phase4_fillForms() {
        var testValues = {
            text: 'test-input', search: 'knowledge graph', password: 'TestPass123!',
            email: 'test@example.com', number: '42', tel: '555-1234',
            url: 'https://example.com', date: '2026-01-01', 'datetime-local': '2026-01-01T12:00'
        };
        var nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
        var textareaSetter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value').set;
        var filled = [];
        var inputs = document.querySelectorAll('input:not([type="hidden"]):not([type="submit"]):not([type="button"]):not([type="checkbox"]):not([type="radio"]):not([type="file"]), textarea');
        for (var i = 0; i < inputs.length; i++) {
            if (timeout()) break;
            var el = inputs[i];
            if (!visible(el) || el.disabled || el.readOnly) continue;
            var val = testValues[el.type] || testValues['text'];
            var ph = (el.placeholder || '').toLowerCase();
            if (ph.indexOf('title') !== -1) val = 'Test Document Title';
            else if (ph.indexOf('content') !== -1 || ph.indexOf('body') !== -1 || ph.indexOf('description') !== -1) val = 'This is test content for the traverse script.';
            else if (ph.indexOf('name') !== -1) val = 'Test Name';
            else if (ph.indexOf('tag') !== -1) val = 'test-tag';
            else if (ph.indexOf('search') !== -1 || ph.indexOf('query') !== -1) val = 'knowledge';
            else if (ph.indexOf('url') !== -1) val = 'https://example.com';
            try {
                if (el.tagName === 'TEXTAREA' && textareaSetter) {
                    textareaSetter.call(el, val);
                } else if (nativeSetter) {
                    nativeSetter.call(el, val);
                } else {
                    el.value = val;
                }
                el.dispatchEvent(new Event('input', { bubbles: true }));
                el.dispatchEvent(new Event('change', { bubbles: true }));
                if (el.tagName === 'TEXTAREA') {
                    el.dispatchEvent(new Event('input', { bubbles: true }));
                }
                filled.push({ tag: el.tagName, type: el.type, name: el.name || '', placeholder: el.placeholder || '', value: val });
            } catch(e) {
                logError('js_error', 'Form fill failed: ' + (e.message || e), { element: el.tagName + '.' + (el.name || el.type || '') });
            }
        }
        if (filled.length > 0) {
            report({ type: 'traverse_form_filled', count: filled.length, fields: filled.slice(0, 20) });
        }
    }

    function phase5_monitoring() {
        report({ type: 'traverse_phase5', msg: 'continuous monitoring started' });
        var checkCount = 0;
        var maxChecks = 12;
        var iv = setInterval(function() {
            checkCount++;
            if (checkCount > maxChecks) { clearInterval(iv); report({ type: 'traverse_monitoring_end' }); return; }
            var mem = null;
            if (window.performance && performance.memory) {
                mem = { usedJSHeapSize: Math.round(performance.memory.usedJSHeapSize / 1048576),
                        totalJSHeapSize: Math.round(performance.memory.totalJSHeapSize / 1048576),
                        jsHeapSizeLimit: Math.round(performance.memory.jsHeapSizeLimit / 1048576) };
            }
            report({ type: 'traverse_heartbeat', check: checkCount, elapsed: elapsed(),
                domNodes: document.querySelectorAll('*').length,
                errorCount: sessionErrors.length,
                memory: mem });
        }, 10000);
    }

    async function run() {
        report({ type: 'traverse_start', timestamp: Date.now(), url: location.href, maxMs: MAX_MS });
        try {
            await phase0_login();
            if (timeout()) return;
            phase1_captureState();
            if (timeout()) return;
            await phase2_sidebarNav();
            if (timeout()) return;
            await phase3_clickElements();
            if (timeout()) return;
            await phase4_fillForms();
            phase5_monitoring();
            var cats = { js_error: 0, wasm_trap: 0, network_error: 0, layout_error: 0, auth_error: 0, render_error: 0, other: 0 };
            for (var i = 0; i < errorLog.length; i++) {
                var c = errorLog[i].category || 'other';
                cats[c] = (cats[c] || 0) + 1;
            }
            report({ type: 'traverse_complete', elapsed: elapsed(), pagesVisited: Object.keys(pagesVisited).length,
                totalErrors: errorLog.length, errorsByCategory: cats,
                errors: errorLog.slice(0, 50), timedOut: timeout() });
        } catch(e) {
            logError('js_error', 'Traverse fatal: ' + (e.message || e), { stack: (e.stack || '').substring(0, 500) });
            report({ type: 'traverse_error', elapsed: elapsed(), error: String(e.message || e), stack: (e.stack || '').substring(0, 500),
                errorCount: errorLog.length, pagesVisited: Object.keys(pagesVisited).length });
        }
    }

    var safetyTimeout = setTimeout(function() {
        report({ type: 'traverse_safety_timeout', elapsed: elapsed(), errorCount: errorLog.length, pagesVisited: Object.keys(pagesVisited).length });
    }, MAX_MS + 5000);

    setTimeout(function() { run().then(function() { clearTimeout(safetyTimeout); }); }, 4000);
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

        // Set API URL for the frontend's ApiClient::default().
        // The frontend reads window.tachyonApiUrl and appends API paths like /documents.
        // The proxy command reads TACHYON_API_URL and appends the full path (including /api/v1).
        // So these two URLs differ: frontend base includes /api/v1, proxy base does not.
        let proxy_base = std::env::var("TACHYON_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let frontend_api_url = format!("{}/api/v1", proxy_base.trim_end_matches('/'));
        if let Err(e) = webview.eval(&format!("window.tachyonApiUrl = \"{}\";", frontend_api_url)) {
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

        // NOTE: We do NOT redirect to http://localhost:8080 because that
        // strips the Tauri IPC bridge (__TAURI__) from the page. All Tauri
        // commands (api_proxy, debug_report, events) require tauri:// origin.
        // Instead, the frontend uses the api_proxy Tauri command to make HTTP
        // requests from Rust, bypassing WebView CORS restrictions.

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
