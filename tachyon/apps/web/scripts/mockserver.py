#!/usr/bin/env python3
"""Local mock of the Tachyon document API for editor development/testing.

Serves static files (like devserver.py) but /api/* resolves against an
in-memory document store — no staging required.

Usage: python3 scripts/mockserver.py [port]
"""
import http.server
import json
import sys
import urllib.parse
from pathlib import Path

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 4440
DIST = Path(__file__).resolve().parent.parent / "dist"

DOCS = {
    "test-doc": {
        "id": "test-doc",
        "title": "WASM Editor Test",
        "content": "# Hello Editor\n\nType here.\n",
        "status": "draft",
        "word_count": 4,
        "updated_at": "2026-09-12T00:00:00Z",
    }
}
LAST_PUT = None

MIME = {
    ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
    ".css": "text/css", ".json": "application/json", ".svg": "image/svg+xml",
    ".png": "image/png", ".ico": "image/x-icon", ".woff2": "font/woff2",
    ".wasm": "application/wasm", ".map": "application/json",
}


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def _send(self, code, body, ctype="application/json"):
        data = body.encode() if isinstance(body, str) else body
        self.send_response(code)
        if ctype:
            self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def _api(self):
        global LAST_PUT
        path = urllib.parse.urlparse(self.path).path
        length = int(self.headers.get("Content-Length") or 0)
        body_in = self.rfile.read(length) if length else b""
        parts = path.strip("/").split("/")

        if path == "/api/v1/auth/password-reset/request" and self.command == "POST":
            self._send(200, json.dumps({"message": "If an account with that email exists, a password reset link has been sent."}))
            return
        if path == "/api/v1/auth/password-reset/confirm" and self.command == "POST":
            self._send(200, json.dumps({"message": "Password updated"}))
            return
        if path == "/api/v1/notifications" and self.command == "GET":
            self._send(200, json.dumps({"notifications": [
                {"id": "nt1", "title": "Mentioned by admin", "body": "admin mentioned you in Architecture Map",
                 "type": "mention", "link": "/documents/doc1", "read": False,
                 "created_at": "2026-09-12T09:00:00Z"},
                {"id": "nt2", "title": "Review requested", "body": None,
                 "type": "review", "link": None, "read": True,
                 "created_at": "2026-09-11T09:00:00Z"},
            ], "count": 2}))
            return
        if path == "/api/v1/notifications/unread-count":
            self._send(200, json.dumps({"count": 1}))
            return
        if path.startswith("/api/v1/notifications"):
            self._send(200, json.dumps({"ok": True}))
            return

        # mock auth
        if path == "/api/v1/auth/me" and self.command == "GET":
            self._send(200, json.dumps({
                "id": "mock-user", "username": "admin",
                "display_name": "Administrator", "email": "admin@tachyon.local",
            }))
            return
        if path in ("/api/v1/auth/login", "/api/v1/auth/guest") and self.command == "POST":
            self._send(200, json.dumps({"success": True, "access_token": "mock-token"}))
            return
        if path == "/api/v1/documents":
            self._send(200, json.dumps({"results": [], "total": 0, "page": 1, "page_size": 12}))
            return
        if path == "/api/v1/spaces":
            self._send(200, json.dumps([
                {"id": "sp1", "name": "Personal", "description": "Your personal space",
                 "icon": "user", "color": "#2563eb", "document_count": 3,
                 "is_default": True, "parent_id": None},
            ]))
            return
        if path == "/api/v1/activity":
            self._send(200, json.dumps({"activities": []}))
            return
        if path == "/api/v1/billing/plans":
            self._send(200, json.dumps({"plans": [
                {"name": "free", "price_monthly_cents": 0, "max_documents": 100,
                 "max_members": 1, "features": ["Basic editor", "5GB storage"]},
                {"name": "pro", "price_monthly_cents": 1200, "max_documents": 10000,
                 "max_members": 5, "features": ["Advanced editor", "50GB storage"]},
            ]}))
            return
        if path == "/api/v1/billing/usage/default":
            self._send(200, json.dumps({"usage": {
                "organization_id": "default", "documents_total": 2, "members_total": 1,
                "storage_bytes": 1048576, "plan": "free",
                "period_start": "2026-09-01T00:00:00Z", "period_end": "2026-10-01T00:00:00Z",
                "documents_created": 2}}))
            return
        if path == "/api/v1/plugins":
            self._send(200, json.dumps([
                {"id": "plg1", "name": "word-count", "description": "Counts words",
                 "author": "tachyon", "enabled": True},
            ]))
            return
        if path == "/api/v1/projects":
            self._send(200, json.dumps([
                {"id": "prj1", "name": "Apollo", "description": "Lander docs"},
            ]))
            return
        if path == "/api/v1/catalog/stats":
            self._send(200, json.dumps({"projects": 1, "components": 5, "members": 2}))
            return
        if path == "/api/v1/admin/audit":
            self._send(200, json.dumps({"entries": [
                {"id": "a1", "timestamp": "2026-09-12T10:00:00Z", "action": "document.create",
                 "user_id": "admin", "resource": "doc/123"},
            ]}))
            return
        if path == "/api/v1/ssg/config":
            self._send(200, json.dumps({
                "site_title": "Tachyon Docs", "site_description": "A knowledge base",
                "base_url": "https://docs.example.com", "theme": "auto"}))
            return
        if path == "/api/v1/graph/nodes":
            self._send(200, json.dumps({"nodes": [
                {"id": "n1", "name": "Rust", "node_type": "concept", "weight": 5},
                {"id": "n2", "name": "Tachyon", "node_type": "project", "weight": 8},
                {"id": "n3", "name": "Knowledge", "node_type": "concept", "weight": 3},
            ], "total": 3, "page": 1, "page_size": 200}))
            return
        if path == "/api/v1/graph/edges":
            self._send(200, json.dumps({"edges": [
                {"id": "e1", "source_id": "n1", "target_id": "n2", "label": "built_with"},
                {"id": "e2", "source_id": "n2", "target_id": "n3", "label": "manages"},
            ], "total": 2}))
            return
        if path == "/api/v1/canvases" and self.command == "GET":
            self._send(200, json.dumps({"canvases": [
                {"id": "cv1", "title": "Architecture Map", "updated_at": "2026-09-12T00:00:00Z"},
            ], "total": 1}))
            return
        if path == "/api/v1/canvases" and self.command == "POST":
            body = json.loads(body_in or b"{}")
            self._send(200, json.dumps({"canvas": {
                "id": "cv-new", "title": body.get("title", "Untitled"),
                "owner_id": body.get("owner_id", "mock-user"),
                "created_at": "2026-09-12T00:00:00Z", "updated_at": "2026-09-12T00:00:00Z"}}))
            return
        if path == "/api/v1/canvases/cv1/nodes" and self.command == "GET":
            self._send(200, json.dumps([
                {"id": "cn1", "canvas_id": "cv1", "node_type": "note",
                 "data": {"label": "API Gateway"}, "position_x": 100, "position_y": 80,
                 "created_at": "2026-09-12T00:00:00Z", "updated_at": "2026-09-12T00:00:00Z"},
                {"id": "cn2", "canvas_id": "cv1", "node_type": "note",
                 "data": {"label": "Auth Service"}, "position_x": 420, "position_y": 200,
                 "created_at": "2026-09-12T00:00:00Z", "updated_at": "2026-09-12T00:00:00Z"},
            ]))
            return
        if path == "/api/v1/canvases/cv1/edges" and self.command == "GET":
            self._send(200, json.dumps([
                {"id": "ce1", "canvas_id": "cv1", "source_id": "cn1", "target_id": "cn2",
                 "edge_type": "default", "style": {},
                 "created_at": "2026-09-12T00:00:00Z", "updated_at": "2026-09-12T00:00:00Z"},
            ]))
            return
        if path == "/api/v1/canvases/cv1" and self.command == "GET":
            self._send(200, json.dumps({"canvas": {
                "id": "cv1", "title": "Architecture Map", "owner_id": "mock-user",
                "created_at": "2026-09-12T00:00:00Z", "updated_at": "2026-09-12T00:00:00Z"}}))
            return
        if path.startswith("/api/v1/canvases") and self.command in ("POST", "PUT", "DELETE"):
            self._send(200, json.dumps({"ok": True,
                "node": {"id": "cn-new", "canvas_id": "cv1", "node_type": "note",
                         "data": {"label": "New"}, "position_x": 10, "position_y": 10,
                         "created_at": "", "updated_at": ""},
                "edge": {"id": "ce-new", "canvas_id": "cv1", "source_id": "cn1",
                         "target_id": "cn2", "edge_type": "default", "style": {},
                         "created_at": "", "updated_at": ""}}))
            return
        if path == "/api/v1/tags":
            self._send(200, json.dumps({"tags": [
                {"tag": "rust", "count": 4}, {"tag": "notes", "count": 2},
            ], "total": 2}))
            return
        if path == "/api/v1/search":
            self._send(200, json.dumps({"results": [
                {"id": "doc1", "title": "Search hit", "headline": "matched <mark>term</mark>",
                 "tags": ["rust"], "status": "published", "word_count": 42,
                 "rank": 1.0, "created_at": None, "updated_at": None},
            ], "total": 1, "page": 1, "page_size": 20, "facets": {}}))
            return

        # GET /api/v1/documents/{id}
        if self.command == "GET" and len(parts) == 4 and parts[2] == "documents":
            doc = DOCS.get(parts[3])
            if doc:
                self._send(200, json.dumps(doc))
            else:
                self._send(404, json.dumps({"error": "not found"}))
            return

        # PUT /api/v1/documents/{id}
        if self.command == "PUT" and len(parts) == 4 and parts[2] == "documents":
            payload = json.loads(body_in or b"{}")
            DOCS[parts[3]]["content"] = payload.get("content", "")
            DOCS[parts[3]]["title"] = payload.get("title", DOCS[parts[3]]["title"])
            LAST_PUT = {"id": parts[3], "content": DOCS[parts[3]]["content"]}
            self._send(200, json.dumps({"ok": True}))
            return

        if path == "/__mock/last-put":
            self._send(200, json.dumps(LAST_PUT or {}))
            return

        if path == "/health":
            self._send(200, json.dumps({"status": "healthy", "mock": True}))
            return

        self._send(404, json.dumps({"error": "mock: no route"}))

    def _static(self):
        path = urllib.parse.urlparse(self.path).path
        if path == "/":
            path = "/index.html"
        f = (DIST / path.lstrip("/")).resolve()
        if not str(f).startswith(str(DIST)) or not f.is_file():
            f = DIST / "index.html"
        if not f.is_file():
            self._send(404, "not built - run: bun run build", "text/plain")
            return
        self._send(200, f.read_bytes(), MIME.get(f.suffix, "application/octet-stream"))

    do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = do_OPTIONS = lambda self: (
        self._api()
        if (self.path.startswith("/api") or self.path == "/health" or self.path.startswith("/__mock"))
        else self._static()
    )


if __name__ == "__main__":
    if not (DIST / "index.html").is_file():
        print("dist/index.html missing - run: bun run build", file=sys.stderr)
        sys.exit(1)
    print(f"mock server on http://127.0.0.1:{PORT} (dist={DIST})")
    http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
