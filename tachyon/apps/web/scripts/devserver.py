#!/usr/bin/env python3
"""Tachyon web dev server: static files + /api reverse proxy, zero deps.

Serves apps/web/dist and forwards /api/* to the Tachyon server — same-origin
for the browser (no CORS / Private-Network-Access issues in dev).

Usage: python3 scripts/devserver.py [port] [api_target]
       python3 scripts/devserver.py 4407 http://192.168.1.191:18080
"""
import http.server
import sys
import urllib.request
import urllib.error
from pathlib import Path

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 4407
API_TARGET = (sys.argv[2] if len(sys.argv) > 2 else "http://192.168.1.191:18080").rstrip("/")
DIST = Path(__file__).resolve().parent.parent / "dist"

MIME = {
    ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
    ".css": "text/css", ".json": "application/json", ".svg": "image/svg+xml",
    ".png": "image/png", ".ico": "image/x-icon", ".woff2": "font/woff2",
    ".map": "application/json", ".wasm": "application/wasm",
}


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def _proxy(self):
        length = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(length) if length else None
        if "login" in self.path:
            print(f"[proxy] {self.command} {self.path} ct={self.headers.get('Content-Type')!r} body={body!r:.120}", flush=True)
        req = urllib.request.Request(
            API_TARGET + self.path, data=body, method=self.command
        )
        for h in ("Content-Type", "Authorization", "Accept", "x-request-id"):
            if self.headers.get(h):
                req.add_header(h, self.headers[h])
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                self._send(r.status, r.read(), r.headers.get("Content-Type"))
        except urllib.error.HTTPError as e:
            self._send(e.code, e.read(), e.headers.get("Content-Type") if e.headers else None)
        except Exception as e:
            self._send(502, str(e).encode(), "text/plain")

    def _send(self, code, body, ctype):
        self.send_response(code)
        if ctype:
            self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _static(self):
        path = self.path.split("?")[0]
        if path == "/":
            path = "/index.html"
        f = (DIST / path.lstrip("/")).resolve()
        if not str(f).startswith(str(DIST)) or not f.is_file():
            # SPA fallback
            f = DIST / "index.html"
        if not f.is_file():
            self._send(404, b"not built - run: bun run build", "text/plain")
            return
        self._send(200, f.read_bytes(), MIME.get(f.suffix, "application/octet-stream"))

    do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = do_OPTIONS = lambda self: (
        self._proxy()
        if (self.path.startswith("/api") or self.path == "/health")
        else self._static()
    )


if __name__ == "__main__":
    if not (DIST / "index.html").is_file():
        print("dist/index.html missing — run `bun run build` first", file=sys.stderr)
        sys.exit(1)
    print(f"serving {DIST} on http://127.0.0.1:{PORT}  (api → {API_TARGET})")
    http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
