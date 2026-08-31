# Tachyon Roadmap

**Revision:** 2026-08-31
**Status:** Staging validated; production launch gates remain

This is the authoritative roadmap. Superseded execution, gap-analysis, competitive, and SSG planning documents are retained in [`docs.archived/archived/`](docs.archived/archived/) for historical context. They are not current commitments.

## 1. Product position

Tachyon is a self-hostable, Rust-native knowledge platform for developers and technical teams. Its defensible position is **Markdown-first authoring, local ownership, fast search/rendering, and collaborative editing**—not parity with every block-editor feature in Notion or Confluence.

Primary comparison sets:

- Documentation: GitBook, Docusaurus, VitePress, ReadMe, Mintlify
- Team knowledge: Outline, Slite, Nuclino, Confluence, Guru
- Personal knowledge: Obsidian, Logseq, Roam, Anytype, AFFiNE
- Collaborative editors: Notion, Coda, Dropbox Paper, Craft

The archived competitive analyses contain the detailed feature matrices; all claims below must be verified against running behavior before being marketed as product parity.

## 2. System shape

```text
Web (Leptos/WASM)   Desktop (Tauri)   CLI
        \                 |            /
             HTTP / WebSocket
                    |
              Axum server
       REST + GraphQL + WebSocket
                    |
     PostgreSQL/pgvector   Redis   Tantivy
                    |
          documents, users, RBAC,
          audit, search, collaboration
```

The workspace is split into domain/core, server, database, renderer, search, editor, frontend, desktop, SSG, import/export, storage, plugin-runtime, CLI, testing, and benchmark crates.

## 3. Verified baseline

### Code and tests

- `cargo check --workspace` passed during the runtime hardening work.
- Verified targeted suites: `tachyon-core` **145**, `tachyon-database` **156**, `tachyon-frontend` **181**, and `tachyon-server` **648** tests passed.
- Prometheus recorder initialization is idempotent; repeated metrics requests no longer panic.
- Poisoned mutexes in the review panel recover instead of taking down requests.
- Redis-backed rate limiting is exercised with `429` responses and `Retry-After` recovery; in-memory fallback remains available.
- A shared `reqwest::Client` is injected through application state rather than recreated by feature modules.
- Graph edge insertion, Argon2 low-memory parameters, public health/metrics routing, and k6 harness defects were corrected.

### CachyOS staging

Staging is running on the user-provided CachyOS host:

- Host: `192.168.1.191`
- Application: `:8082`
- Persistence: dedicated PostgreSQL-backed instance
- Process supervision: user-level systemd service
- Smoke: **12/12 checks, 0% failures**
- Application load: **25 VUs, 0% expected-response failures**
- Rate limit: **21 observed 429s, 100% rate-limit headers, 100% valid Retry-After**
- WebSocket: **10 VUs / 10 seconds, 37 cycles, 100% connection success, p95 connect 47ms**

These are LAN/debug-or-staging measurements, not production capacity claims. A 24-hour soak, TLS endpoint, external network path, and multi-instance test are still required.

## 4. Current release gates

| Gate | Status | Evidence / next action |
|---|---|---|
| Runtime quality fixes | **Complete** | Targeted tests and live checks above |
| Local compilation | **Complete** | Workspace `cargo check` passed |
| Staging deployment | **Complete** | CachyOS service live on `:8082` |
| HTTP smoke/load validation | **Complete** | k6 smoke and 25-VU run passed |
| Rate-limit validation | **Complete** | Dedicated k6 run observed 429/header/recovery behavior |
| WebSocket lifecycle validation | **Baseline complete** | Connection stability measured; protocol-specific broadcast semantics need deeper tests |
| Explicit CORS defaults | **Complete and active** | Safe localhost default, explicit env parsing, production wildcard rejection; verified on staging with allow/deny origin checks |
| Production TLS/DNS | **Pending** | Configure reverse proxy and certificate; verify from an external client |
| CI/CD end-to-end deployment | **Pending** | Configure secrets and prove push → image → deploy → health |
| 24-hour soak and resource profile | **Pending** | Run k6 soak; record RSS, DB pool, Redis, errors, and reconnects |
| OWASP ZAP against live staging | **Pending** | Scan the deployed HTTPS surface and remediate high findings |
| Backup/restore drill | **Pending** | Restore PostgreSQL backup into an isolated database and exercise recovery |
| Production release | **Blocked** | Requires all preceding production gates plus rollback rehearsal |

## 5. Near-term execution order

### Phase A — Harden staging into a repeatable environment

1. Rebuild/restart the staging service from the current commit so the CORS hardening is active.
2. Add a documented deployment script or CI job that packages/builds the native staging artifact without manual shell quoting.
3. Record service configuration, database location, migration version, log location, and rollback command in the deployment runbook.
4. Keep staging isolated from unrelated containers and retain a dedicated database/volume.

**Exit:** a clean host or reboot can reproduce the service and health checks without ad-hoc process management.

### Phase B — Production validation

1. Put Nginx/Caddy in front of staging and configure TLS.
2. Set exact `TACHYON_CORS_ORIGINS`; never use `*` outside intentional development.
3. Connect Prometheus and Grafana to `/metrics/prometheus` and define alerts for errors, latency, DB pool exhaustion, memory, and WebSocket disconnects.
4. Run smoke, load, stress, WebSocket, and 24-hour soak tests against the deployed endpoint.
5. Run OWASP ZAP and manual authorization checks, especially GraphQL, document ownership, plugins, uploads, and WebSockets.
6. Perform one backup/restore and one rollback rehearsal.

**Exit:** all release-gate rows above are green with retained reports.

### Phase C — First production release

1. Configure CI/CD secrets and prove the full deployment path.
2. Deploy one production replica behind the reverse proxy.
3. Observe for a defined burn-in window before adding replicas.
4. Tag the release only after migrations, backup, rollback, monitoring, and external health checks are proven.

## 6. Product roadmap after launch

Prioritize outcomes over feature-count parity.

### P0 — Adoption and trust

- Markdown vault, Notion, and Confluence migration workflows with conflict reporting and data-loss tests.
- Reliable document ownership/RBAC review across REST, GraphQL, plugins, uploads, and WebSockets.
- PDF export and import/export round-trip fixtures.
- User-facing onboarding, admin documentation, upgrade/rollback instructions, and telemetry that is opt-in and privacy-safe.

### P1 — Core knowledge workflow

- Wiki-links, backlinks, graph navigation, daily notes, and block references.
- Editor preview parity for admonitions, embeds, tables, and slash commands.
- Search quality evaluation on a representative corpus, including semantic-search relevance and latency.
- Mobile-responsive web/PWA read access before native mobile applications.

### P2 — Collaboration and ecosystem

- Room-scoped WebSocket behavior with multi-client protocol tests.
- Comments, mentions, review state, notifications, and webhook management UI.
- Reference WASM plugins, signed manifests, permission boundaries, and a small curated registry.
- Frontend localization and accessibility validation on real browser/device combinations.

### P3 — Enterprise expansion

- Complete SAML runtime flow with XML-DSig verification and IdP interoperability tests.
- Wire DLP policies into request/content pipelines rather than presenting scaffolding as enforcement.
- SCIM interoperability tests, retention/eDiscovery workflows, audit exports, and compliance evidence automation.
- Read replicas, PgBouncer, CDN, and horizontal scaling only after measured workload data justifies them.

## 7. Engineering standards

Every roadmap item must ship with:

1. A narrow design note and explicit non-goals.
2. Unit and integration tests; E2E tests for user-visible or security-sensitive behavior.
3. Authorization, input validation, rate limiting, and audit behavior where applicable.
4. A benchmark or load measurement for hot paths.
5. Updated operator/user documentation.
6. A rollback or migration story for persistent data.

Maintain KISS at boundaries, DRY shared policy code rather than duplicating behavior, SOLID module responsibilities, explicit failure handling, and observable operations. Do not claim “implemented,” “enterprise-ready,” or “competitor parity” solely because a type, route, or scaffold exists.

## 8. Decision log

- **2026-08-31:** Staging is considered validated for smoke, baseline HTTP load, rate limiting, and WebSocket connection lifecycle; production readiness remains gated by TLS, external tests, soak, security scan, backups, and CI/CD.
- **2026-08-31:** CORS now defaults to `http://localhost:8080`, accepts explicit comma-separated origins, and rejects wildcard origins when development mode is disabled.
- **2026-08-31:** Superseded planning documents are archived; this file is the only active roadmap.
- **2026-06-09—2026-08-31:** Runtime quality fixes and test-harness corrections were validated incrementally; historical reports remain in `reports/` and the git history.

## 9. Definition of done for “production ready”

- [ ] Current commit is reproducibly deployed from CI/CD.
- [ ] HTTPS, DNS, explicit CORS, security headers, and secrets are verified externally.
- [ ] Smoke/load/stress/soak/WebSocket tests pass with retained reports.
- [ ] ZAP/manual security review has no unresolved high-severity findings.
- [ ] Backup restore and rollback rehearsals succeed.
- [ ] Prometheus/Grafana alerts are connected and tested.
- [ ] User/admin/API documentation matches the deployed behavior.
- [ ] Release notes identify scaffolding and known limitations honestly.
