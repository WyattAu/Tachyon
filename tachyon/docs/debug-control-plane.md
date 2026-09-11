# Debug Control Plane

Agent-driven testing for the Tachyon desktop app. Compiled into debug builds
only; at runtime requires `TACHYON_DEBUG >= 1`. Binds `127.0.0.1:17890`
(configurable via `TACHYON_CONTROL_PORT`), token-gated via
`/tmp/tachyon-control.token` (0600, generated per run or pinned via
`TACHYON_CONTROL_TOKEN`).

## Quick start (Linux/Nix, proven working)

```bash
# 1. Build
cargo build -p tachyon-desktop-app

# 2. Launch — the EGL fix is REQUIRED on NVIDIA + Xwayland with the
#    Nix-linked webkit: force the NIX Mesa vendor json (NOT the system one —
#    mixing system Mesa into a Nix process guarantees EGL_BAD_PARAMETER).
NIX_MESA=$(ls -d /nix/store/*mesa*/share/glvnd/egl_vendor.d/50_mesa.json | head -1)
TACHYON_DEBUG=1 \
TACHYON_API_URL=http://192.168.1.191:18080 \
TACHYON_TEST_SERVER=http://192.168.1.191:18080 \
TACHYON_TEST_USER=admin TACHYON_TEST_PASSWORD=admin123 \
TACHYON_RUN_ID=my-run \
__EGL_VENDOR_LIBRARY_FILENAMES="$NIX_MESA" \
./target/debug/tachyon-desktop-app &

# 3. Drive it
TOKEN=$(cat /tmp/tachyon-control.token)
H="Authorization: Bearer $TOKEN"; B=http://127.0.0.1:17890
curl -s -H "$H" $B/control/state
curl -s -H "$H" -X POST $B/control/eval -d '{"js":"1"}'        # warmup — first eval after page load drops its callback (IPC settle race)
curl -s -H "$H" -X POST $B/control/navigate -d '{"route":"/login"}'
curl -s -H "$H" -X POST $B/control/type -d '{"selector":"input[name=server_url]","text":"http://192.168.1.191:18080"}'
curl -s -H "$H" -X POST $B/control/type -d '{"selector":"input[name=username]","text":"admin"}'
curl -s -H "$H" -X POST $B/control/type -d '{"selector":"input[name=password]","text":"..."}'
curl -s -H "$H" -X POST $B/control/eval -d '{"js":"var b=Array.from(document.querySelectorAll(\"button\")).find(b=>b.textContent.trim()===\"Sign in\"); b?b.click():\"NO_BTN\""}'
sleep 5 && curl -s -H "$H" $B/control/state                    # route should read /dashboard
curl -s -H "$H" -X POST $B/control/screenshot -d '{}'          # PNG in /tmp/tachyon-screenshots/
curl -s -H "$H" $B/control/report
```

## Hard-won gotchas (all hit in practice)

1. **EGL abort**: `Could not create default EGL display: EGL_BAD_PARAMETER`
   aborts the WebKit WebProcess → blank webview. Fix: `__EGL_VENDOR_LIBRARY_FILENAMES`
   pointing at the **Nix** Mesa json (see above). `WEBKIT_DISABLE_DMABUF_RENDERER`,
   `WEBKIT_DISABLE_COMPOSITING_MODE`, `WEBKIT_SKIA_ENABLE_CPU_RENDERING` did
   NOT fix it; the system-Mesa vendor json made it worse (ABI mismatch).
2. **First eval after page load loses its callback.** Always send a throwaway
   warmup eval, then real commands. Callbacks also arrive in slow bursts when
   the page is busy — poll with generous waits.
3. **Eval JS runs in expression context** (eval() completion value):
   `{"js":"2+3"}` returns `'5'`. Statement-only code returns `undefined` —
   end with an expression or use an explicit `return`.
4. **Selector ambiguity**: `input[type=text]` matches a header search box or
   server_url field BEFORE the login form's username field. Always use
   `input[name=...]` or form-scoped selectors. Setting the login page's
   `server_url` field to garbage produces requests to `<garbage>/api/v1/...`.
5. **Click by text** when `button[type=submit]` is ambiguous (the login page
   has a dozen submit-typed buttons): use the eval-with-find-by-text pattern.
6. **Shell safety**: `pkill -f tachyon-desktop-app` kills the calling shell
   too (the pattern matches your own command line) — use `pkill tachyon-desktop`
   (comm match).
7. **Telemetry pays for itself immediately**: the injected hooks caught the
   click-endpoint JS escaping bug via `js_error` before any human noticed.

## Staging deploy notes (192.168.1.191, alias `cachyos-x8664`)

- Source: `~/tachyon-staging/` (flat workspace root). Estate crates
  (`salting`, `cryptkit`, `tokenkit`, `media-kit`, `validkit`, `docs-pipeline`)
  must live at `~/` as siblings — they are workspace roots and cannot nest.
  All `path = ...` refs are patched (`../salting` from root, `../../../validkit`
  from `crates/X`). Patch script pattern: see `/tmp/sync-and-patch.sh`.
- Build: `cargo build --release -p tachyon-server` (rustc 1.95 present;
  the wasmtime-46 lock needs ≥ 1.94).
- Deploy: copy the binary to
  `~/tachyon-staging/releases/<rev>/tachyon/target/release/tachyon-server`,
  run via `~/start-tachyon.sh` with `setsid nohup` (append log to
  `/tmp/tachyon.log`). **The script MUST set `TACHYON_PORT=18080`** — without
  it the server defaults to 8080 (civit-core's port) and dies with a
  misleading "Address already in use".
- **Do NOT use `systemctl --user` for tachyon on this box**: the user has no
  linger (`loginctl enable-linger` fails without root), so user services die
  with the SSH session that started them. The old unit
  `tachyon-staging.service` was disabled for this reason (stale env file
  pointed at DB port 5432 with an old password; kept as
  `tachyon-staging.env.bak-*`).
- Old binary `~/tachyon-server-bin` (June build) is superseded; do not use.

## Server-side telemetry (this cycle)

- Query strings are redacted in request logs (`token`, `api_key`, `session`,
  `password`, `secret`, `auth`, `otp`, … → `[REDACTED]`) — verified live:
  `query":"Some(\"token=[REDACTED]&password=[REDACTED]\")"`.
- The `TraceLayer` span's `uri` field is redacted too (the raw query used to
  leak through DEBUG span lines even after the log-event fix).
- Prometheus metrics carry `method`/`route`/`status` labels; dynamic path
  segments normalized (`/documents/<uuid>` → `/documents/:id`). Verified live:
  `tachyon_requests_total{method="GET",route="/api/v1/documents",status="2xx"}`.
- Known gap: 401 rejections short-circuit inside the auth layer, before the
  request-completed logging middleware — they log "Authentication failed"
  but no request-completed line.

## Endpoints

| Method | Path | Body | Returns |
|---|---|---|---|
| GET | `/control/state` | — | run_id, route, config |
| POST | `/control/navigate` | `{route}` | `eval_id` |
| POST | `/control/eval` | `{js}` | `eval_id` (result via events) |
| POST | `/control/click` | `{selector}` | `eval_id` |
| POST | `/control/type` | `{selector, text}` | `eval_id` |
| POST | `/control/key` | `{key}` | `eval_id` |
| POST | `/control/screenshot` | `{}` | `{path}` |
| GET | `/control/events?since=N` | — | `{events:[{seq,kind,...}]}` |
| GET | `/control/report` | — | `{report: markdown}` |

Eval results are asynchronous (Tauri `eval()` is fire-and-forget): each action
returns an `eval_id`, and the outcome arrives as an event
`{kind:"eval_result", eval_id, ok, result}` on `/control/events`. Poll with
`since=<last seq>`.

## Correlation

- Every `api_proxy` HTTP request carries `x-request-id: <run_id>.<action_id>`;
  the server echoes it into its request logs.
- Every `debug_report` line carries `run_id`.
- Pin a scenario with `TACHYON_RUN_ID=<id>` to join desktop events, server
  request logs (`/tmp/tachyon.log` on staging), and audit trails by one id.

## Test fixture env vars

| Var | Default | Purpose |
|---|---|---|
| `TACHYON_TEST_SERVER` | `http://127.0.0.1:8080` | API base for auto-login |
| `TACHYON_TEST_USER` | `admin` | traversal login user |
| `TACHYON_TEST_PASSWORD` | `admin` | traversal login password |
| `TACHYON_TEST_DOC_ID` | (dev doc uuid) | doc used by `EDITOR_TEST` |
| `TACHYON_TEST_ROUTES` | core routes | comma-separated traverse list |
| `TACHYON_RUN_ID` | generated | correlation id |
| `TACHYON_DEBUG` | `0` | `1` hooks + control plane, `2` + auto-traverse |
| `TACHYON_CONTROL_PORT` | `17890` | control plane port |
| `TACHYON_CONTROL_TOKEN` | generated | pin the control token (CI) |

## Server-side telemetry (this cycle)

- Query strings are redacted in request logs (`token`, `api_key`, `session`,
  `password`, `secret`, `auth`, `otp`, … → `[REDACTED]`).
- Prometheus metrics now carry `method`/`route`/`status` labels; dynamic path
  segments are normalized (`/documents/<uuid>` → `/documents/:id`) to keep
  cardinality bounded.
