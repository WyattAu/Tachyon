# Feature-Complete v1.0 Checklist (Spec Freeze)

> Frozen at migration kickoff. Every row: **Surface × Client × Test**.
> "FC" = feature-complete gate. Nothing ships 1.0 with an unclosed row in its
> phase. Update this file as rows close — it is the single source of truth.

Status legend: `[ ]` open · `[~]` in progress · `[x]` done · `[N]` won't do (descoped, reason required)

## 1. Auth & Identity — Phase C
- [ ] Email/password login — page: login.rs → Solid `LoginPage` — test: e2e/auth.spec
- [ ] Register + email verification (SMTP real) — test: e2e/auth.spec
- [ ] **Forgot password** (reset request → email → confirm) — currently DEAD LINK `/login/forgot`
- [ ] **OAuth Google** (real redirect flow; buttons are currently decorative)
- [ ] **OAuth GitHub** (same)
- [ ] Guest login — works (verified via control plane)
- [ ] MFA enable/verify/disable + login challenge
- [ ] Magic link, SMS OTP (conditional), sessions management UI
- [ ] API keys UI
- [ ] Profile edit (`/profile`), password change
- [ ] SSO: OIDC/SAML/LDAP — config-gated; FC = documented + sandbox-verified once

## 2. Documents domain — Phase B (migration) + E (lifecycles)
- [x] List/grid, search-in-list, status filter, pagination — Solid port verified (mock + local server)
- [x] **Editor**: create/edit/auto-save — WASM core (editor-wasm) + Solid view; save round-trip verified vs real server
- [x] `__tachyonEditor` bridge contract — frozen + 11/11 live contract checks PASS (control plane vs desktop); disposal-guard fix shipped
- [ ] Markdown preview + TOC
- [ ] Versions: list/diff/restore — drill in Phase E
- [ ] Branches: create/diff/merge — drill in Phase E
- [ ] Comments + reviews + conflict resolution UI
- [ ] Backlinks, references, citations UI
- [ ] Attachments upload (multipart, 50 MB)
- [ ] **PDF viewer + annotation persistence** (annotations currently local-only)
- [~] Templates: list + categories ported (verified); CRUD apply pending
- [ ] Wikilinks → graph edges (server-side, done) + UI link-following

## 3. Knowledge features — Phase B
- [ ] Graph visualization (sigma.js — DECIDED; next port)
- [x] Spaces list (typed /spaces)
- [ ] Graph time-travel (query/at/diff endpoints)
- [~] Search: full-text query + results verified; suggestions/saved/semantic pending
- [x] Tags browse/filter (verified)
- [ ] Spaces hierarchy + members + move-document
- [ ] Daily notes / journal pages
- [ ] **Canvas: persistence wired** (13 API methods currently dead code — build natively in Solid)
- [ ] **Flashcards: through ApiClient** (currently bypasses auth) + SRS review flow
- [ ] Blog: public feed + admin editor

## 4. Collaboration — Phase E
- [ ] **2-client CRDT editing E2E** (control plane drives two instances, one doc)
- [ ] Presence (PUT/GET/DELETE presence endpoints) + UI avatars
- [ ] Comments/mentions via collaboration endpoints
- [ ] Notifications: SSE stream → toast + `/notifications` page (page file currently dead)
- [ ] Activity feed

## 5. Teams/Orgs/Admin — Phase B/C
- [x] Audit log table + filter (port; Export CSV pending)
- [x] Catalog: stats + projects (port)
- [x] Plugins list (port)
- [x] SSG config + build page (port)
- [~] Teams list + detail + members ported; create/edit UI pending
- [ ] Organizations CRUD + members
- [ ] Roles/RBAC: `/admin/roles` + permission matrix drill
- [ ] **Admin users page** (file dead on disk — revive in Solid)
- [ ] **Admin analytics** (same) + `/analytics/*` endpoints
- [ ] Audit log + export CSV + stats
- [ ] Onboarding flow (status/suggestions/complete/sample-content)
- [ ] Webhooks UI + `/v2/webhooks/logs`
- [ ] Branding + **themes page** (currently orphaned) + notification preferences

## 6. Money — Phase D
- [ ] Plans display (sentinel fix shipped) + subscribe/change/cancel via UI
- [ ] **TrueLayer sandbox E2E**: mandate → payment → HMAC webhook → subscription state
- [ ] **Stripe as second provider** (new integration behind the same mandate/payment model)
- [ ] Invoices + payment history UI
- [ ] Usage page (fixed this cycle: 0/100, 1/1 verified)

## 7. Data lifecycles — Phase E
- [ ] Import: Obsidian vault E2E (real vault fixture)
- [ ] Import: Notion export E2E · Confluence E2E
- [ ] SSG: configure → build → download → publish E2E
- [ ] Plugins: install → invoke → slash-commands E2E (wasmtime runtime)
- [ ] **Plugin marketplace** (server module is an unmounted stub — build real backend + UI)
- [ ] Repositories (git-like): init/commit/push/status UI
- [ ] Local-first: offline queue → sync (flaky-network E2E)
- [ ] AI endpoints: complete/summarize/improve/tags/embed/question UI wiring

## 8. Security & trust — Phase D/F
- [ ] **E2EE full UI adoption** (crypto.rs AES-256-GCM done; needs enable-flow + key escrow UX + indicator)
- [ ] GDPR export/erase routes (unmounted stub — finish + mount)
- [ ] Rate limiting ON in production config; CORS tightened; JWT rotation
- [ ] cargo-audit + cargo-deny clean in CI
- [ ] SOC2 checklist pages verified against data

## 9. i18n & a11y — Phase C/F
- [ ] i18n framework adopted app-wide (string extraction), EN + 1 locale (locale TBD)
- [ ] a11y audit (hooks already measure layout shifts/a11y in debug runs) — zero critical findings
- [ ] Mobile PWA polish: installable, offline page, responsive shell

## 10. Scale & ops — Phase F
- [ ] k6: 100 VUs sustained, p95 < target, 0 errors on core paths
- [ ] 24h soak clean
- [ ] Postgres backup/restore DRILL executed (backup.sh exists, restore.sh unverified)
- [ ] Grafana dashboards: per-route latency/errors, collab WS, DB pool
- [ ] TLS + real domain + CloudFlare
- [ ] OpenAPI client regeneration CI gate (diff = fail)

## Migration status (Phase B): 31/34 routes real
- Remaining: /graph (sigma.js — decided), /canvas + /canvas/:id (Solid renderer + the 13 API methods)
- Verification infra: LOCAL server (Postgres 16 @ 15433 + rustc 1.94 server @ 18080) replaces staging while offline — full stack verified live (login, documents list, WASM editor load+type+save round-trip vs real Postgres, redaction, contract E2E 11/11)

## 11. Release — Phase G
- [ ] Docker image builds (rustc ≥ 1.94 toolchain fix)
- [ ] Desktop installers: deb + AppImage + NSIS
- [ ] CLI release binaries; TUI parity for core flows
- [ ] Docs site (dogfood Tachyon SSG)
- [ ] CHANGELOG + semver + signed v1.0.0 tag

---
## Explicit descopes (won't-do for 1.0)
- [N] Native mobile apps (PWA only)
- [N] Marketplace payouts/revenue-share (marketplace = free listing only)
- [N] Real-time collaborative cursors beyond presence (CRDT text merge is the bar)
- [N] Self-hosted SCIM/SAML full matrix — document + sandbox-verify one provider

## Parity-gate rule (migration, Phase B)
A Leptos page may only be deleted when, on the Solid build:
1. All its rows here are `[x]` or `[N]`
2. Control-plane screenshot matrix matches (manual review OK for visual diffs)
3. Its e2e spec passes against staging
