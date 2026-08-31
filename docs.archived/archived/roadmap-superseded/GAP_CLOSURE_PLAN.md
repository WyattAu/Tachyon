# Comprehensive Gap Closure Plan

**Date:** 2026-06-12 | **Version:** 1.0 | **Author:** Tachyon Architecture Team
**Status:** ACTIVE | **Start Date:** Immediate | **Target:** Competitive parity

---

## Guiding Principles

1. **No feature ships without tests.** Every feature has unit tests, integration tests, and E2E tests before merge.
2. **No feature ships without documentation.** API docs, user guides, and architecture docs are written alongside code.
3. **No feature ships without security review.** Every new endpoint gets auth middleware, input validation, and rate limiting.
4. **No feature ships without performance validation.** Benchmarks run before and after. Regressions block merge.
5. **Formal verification for critical algorithms.** Any algorithm with safety implications gets a Lean4/Coq proof or exhaustive property-based testing.
6. **Existing patterns are reused.** New code follows the architecture established by existing code. No new frameworks, no new paradigms.

---

## Phase A: Foundation (Weeks 1-6)

### A1: PDF Export (Week 1)

**Problem:** PDF export exists as scaffolding but is not wired to routes or tested.

**Solution:**
- Wire `pdf_export.rs` to `POST /api/v1/documents/{id}/export/pdf`
- Use `wkhtmltopdf` or `print-pdf` crate for HTML-to-PDF conversion
- Support page size, margins, headers/footers, table of contents
- Add watermark option for draft documents
- Stream PDF to client (not buffer entire document in memory)

**Architecture:**
```
Document -> Markdown -> HTML (existing renderer) -> PDF (new pipeline)
                                                      |
                                                      v
                                                 Page layout engine
                                                 (margins, headers, footers, TOC)
```

**Testing:**
- Unit: PDF generation for 1-page, 10-page, 100-page documents
- Integration: PDF export endpoint with auth, rate limiting
- Property-based: PDF output is valid PDF/A, correct page count, no truncation
- Performance: 50-page document exports in <5 seconds

**Files:**
- `crates/import-export/src/pdf_export.rs` (rewrite)
- `crates/server/src/routes/document/document_export.rs` (add PDF endpoint)
- `crates/import-export/tests/pdf_tests.rs` (new)

**Quality Gate:** PDF output validates against PDF/A-1b standard.

---

### A2: Notion API Import (Weeks 2-3)

**Problem:** Only ZIP export parsing exists. Need OAuth flow + API pagination.

**Solution:**
- Implement Notion OAuth 2.0 flow (authorization code grant)
- API client with automatic pagination (Notion uses cursor-based pagination)
- Map Notion database properties to Tachyon tags/metadata
- Handle Notion block types: paragraph, heading, list, code, image, bookmark, callout, toggle, table
- Preserve Notion page tree structure as Tachyon space hierarchy
- Rate limit compliance (3 requests/second)

**Architecture:**
```
User -> /import/notion (OAuth redirect) -> Notion API -> Parse blocks -> Create documents
                                |                                    |
                                v                                    v
                         Notion OAuth                           Block type
                         token storage                          mapper
```

**Block Type Mapping:**
| Notion Type | Tachyon Output |
|-------------|----------------|
| paragraph | Markdown paragraph |
| heading_1/2/3 | `#`/`##`/`###` |
| bulleted_list_item | `- ` prefix |
| numbered_list_item | `1. ` prefix |
| to_do | `- [ ] ` or `- [x] ` |
| code | Fenced code block with language |
| image | `![alt](url)` with uploaded image |
| bookmark | `[title](url)` |
| callout | `> ` blockquote with emoji prefix |
| toggle | `<details><summary>` HTML |
| table | Markdown table |
| child_page | Nested document in space |
| database | Tachyon space with tag-based properties |

**Testing:**
- Unit: Block type mapping for all 15+ Notion types
- Integration: Full import of 100-page Notion workspace
- Property-based: Imported documents preserve structure, links, images
- Performance: 500-page import completes in <60 seconds

**Files:**
- `crates/import-export/src/notion.rs` (rewrite with API client)
- `crates/import-export/src/notion_oauth.rs` (new)
- `crates/server/src/routes/import/notion.rs` (new endpoint)
- `crates/import-export/tests/notion_tests.rs` (new)

**Quality Gate:** All 15 block types correctly mapped. No data loss on round-trip.

---

### A3: Confluence REST API Import (Weeks 4-5)

**Problem:** Only XML export parsing exists. Need REST API client.

**Solution:**
- Confluence REST API v2 client with pagination
- Support both Cloud and Data Center editions
- Map Confluence storage format (XHTML) to Markdown
- Preserve page tree, labels, attachments
- Handle Confluence-specific macros: code, info, warning, note, expand,toc

**Architecture:**
```
User -> /import/confluence (URL + credentials) -> Confluence API -> Parse XHTML -> Markdown -> Create documents
                                                                    |
                                                                    v
                                                           Macro expansion
                                                           (code, info, warning)
```

**Macro Mapping:**
| Confluence Macro | Tachyon Output |
|-----------------|----------------|
| code | Fenced code block |
| info/warning/note | Admonition (`> [!info]`) |
| expand | `<details><summary>` HTML |
| toc | Auto-generated TOC |
| jira | Link to Jira issue |
| gliffy/drawio | Image placeholder |
| include | Transcluded content |

**Testing:**
- Unit: XHTML-to-Macro conversion for all macro types
- Integration: Full import of 500-page Confluence space
- Property-based: Page tree preserved, labels become tags
- Performance: 500-page import completes in <120 seconds

**Files:**
- `crates/import-export/src/confluence.rs` (rewrite with API client)
- `crates/import-export/src/confluence_macros.rs` (new)
- `crates/server/src/routes/import/confluence.rs` (new endpoint)
- `crates/import-export/tests/confluence_tests.rs` (new)

**Quality Gate:** All macro types expanded correctly. Page tree structure preserved.

---

### A4: Admonitions in Editor (Week 6)

**Problem:** SSG renders admonitions but editor does not support them.

**Solution:**
- Add admonition syntax highlighting in editor (regex-based, like code blocks)
- Render admonitions in live preview with colored borders
- Support types: note, tip, info, warning, danger, caution
- Keyboard shortcut: `>` + space to start admonition

**Syntax:**
```markdown
> [!note]
> This is a note admonition

> [!warning]
> This is a warning

> [!danger]
> This is dangerous
```

**Testing:**
- Unit: Syntax highlighting for all 6 admonition types
- Integration: Admonition renders correctly in split view
- Visual: Admonition colors match SSG output
- Performance: No editor lag with 50+ admonitions

**Files:**
- `crates/editor/src/highlight/admonitions.rs` (new)
- `crates/renderer/src/markdown.rs` (extend admonition rendering)
- `crates/frontend/src/components/editor_preview.rs` (extend preview)

**Quality Gate:** Admonition rendering matches SSG output exactly.

---

## Phase B: Developer Experience (Weeks 7-14)

### B1: Vim Keybindings (Weeks 7-8)

**Problem:** Developer audience expects Vim/Emacs keybindings.

**Solution:**
- Create keybinding abstraction layer in editor
- Implement Vim normal mode (20 most-used motions)
- Implement Vim visual mode (basic selection)
- Implement Vim insert mode (standard editing)
- Configurable via settings (enable/disable, custom bindings)

**Vim Motions to Implement:**
| Motion | Description | Complexity |
|--------|-------------|------------|
| `h/j/k/l` | Cursor movement | Low |
| `w/b/e` | Word movement | Low |
| `0/$` | Line start/end | Low |
| `gg/G` | Document start/end | Low |
| `dd` | Delete line | Medium |
| `yy` | Yank line | Medium |
| `p/P` | Paste after/before | Medium |
| `u/C-r` | Undo/redo | Low |
| `i/a/o` | Enter insert mode | Low |
| `v/V` | Visual/line select | Medium |
| `:` | Command mode | Medium |
| `x` | Delete character | Low |
| `ciw/ciw/ci"` | Change inside word/bracket/quote | High |
| `f/F` | Find character | Medium |
| `;/,` | Repeat find | Low |
| `/` | Search | Low |
| `n/N` | Next/prev search | Low |
| `.` | Repeat last action | High |
| `J` | Join lines | Low |
| `~` | Toggle case | Low |

**Architecture:**
```
Key Event -> Mode State Machine -> Action Executor -> Editor State
     |              |                    |                  |
     v              v                    v                  v
  Raw keycode   Normal/Insert/     Delete/Yank/        Buffer
                Visual/Command     Move/Change         modification
```

**Testing:**
- Unit: Each motion produces correct buffer modification
- Integration: Vim mode enables/disables via settings
- Property-based: Any sequence of motions preserves buffer integrity
- Performance: No lag with 1000-line document

**Files:**
- `crates/editor/src/keybindings/mod.rs` (new module)
- `crates/editor/src/keybindings/vim.rs` (new)
- `crates/editor/src/keybindings/state_machine.rs` (new)
- `crates/editor/src/keybindings/motions.rs` (new)
- `crates/editor/src/keybindings/actions.rs` (new)

**Quality Gate:** All 20 motions work correctly. Buffer integrity preserved under all operations.

---

### B2: MDX Support (Weeks 9-12)

**Problem:** Documentation teams need MDX for interactive docs.

**Solution:**
- MDX parser that converts MDX to Tachyon's internal AST
- Support JSX components: `<Callout>`, `<Tabs>`, `<CodeBlock>`, `<Badge>`
- Component registry for custom components
- SSG renders MDX to static HTML with client-side hydration
- Editor shows MDX components as preview cards

**MDX Component Registry:**
```rust
struct MdxComponentRegistry {
    components: HashMap<String, Box<dyn MdxComponent>>,
}

trait MdxComponent {
    fn render(&self, props: &MdxProps, children: &str) -> String;
    fn preview(&self, props: &MdxProps) -> String; // For editor
}
```

**Built-in Components:**
| Component | Props | Description |
|-----------|-------|-------------|
| `<Callout>` | type, title | Admonition with title |
| `<Tabs>` | labels | Tabbed content |
| `<CodeBlock>` | lang, title, showLineNumbers | Enhanced code block |
| `<Badge>` | color, text | Status badge |
| `<Frame>` | src, caption | iframe with caption |
| `<Steps>` | - | Numbered steps |

**Testing:**
- Unit: MDX parser handles all JSX syntax variations
- Integration: MDX document renders correctly in SSG
- Property-based: MDX output is valid HTML, no XSS
- Performance: 100-component MDX document renders in <100ms

**Files:**
- `crates/renderer/src/mdx.rs` (new parser)
- `crates/renderer/src/mdx_components.rs` (new registry)
- `crates/ssg/src/mdx.rs` (new SSG integration)
- `crates/frontend/src/components/mdx_preview.rs` (new)

**Quality Gate:** MDX output matches Docusaurus MDX output for equivalent documents.

---

### B3: Admonition Editor Support (Week 13)

**Problem:** Admonitions render in SSG but not in editor preview.

**Solution:**
- Extend `editor_preview.rs` to render admonition blocks
- Add admonition toolbar button
- Support all 6 admonition types with distinct colors
- Click to change admonition type

**Testing:**
- Visual: Admonition renders with correct colors in preview
- Interaction: Toolbar button creates admonition, type selector works

**Files:**
- `crates/frontend/src/components/editor_preview.rs` (extend)

---

### B4: Slash Commands Full Wiring (Week 14)

**Problem:** Slash commands exist but don't insert markdown into editor.

**Solution:**
- Connect slash command selection to editor's `insert_text` method
- After insertion, position cursor correctly (e.g., after `## ` for heading)
- For code blocks, insert opening and closing fences with cursor between them
- For tables, insert header row and separator

**Testing:**
- Unit: Each command inserts correct markdown
- Integration: Slash menu appears, selection works, markdown appears in editor
- Visual: Cursor positioned correctly after insertion

**Files:**
- `crates/frontend/src/components/slash_commands.rs` (extend)
- `crates/frontend/src/components/native_editor.rs` (connect insert)

---

## Phase C: Collaboration & Integration (Weeks 15-24)

### C1: Graph View Interactivity (Weeks 15-16)

**Problem:** Graph view is basic. Needs force-directed layout, zoom, filter.

**Solution:**
- Replace current SVG rendering with Canvas-based force-directed graph
- Use `petgraph` crate for graph algorithms
- Implement force simulation (Barnes-Hut approximation for O(n log n))
- Add zoom/pan with mouse wheel and drag
- Add node search/filter with highlight
- Add cluster view (group by tag/space)
- Export graph as PNG/SVG

**Architecture:**
```
Document Graph (petgraph) -> Force Simulation -> Canvas Renderer
                                |                      |
                                v                      v
                          Force parameters        Animation loop
                          (spring, repulsion,     (requestAnimationFrame)
                           damping, gravity)
```

**Performance Requirements:**
- 100 nodes: 60fps
- 500 nodes: 30fps
- 1000 nodes: 15fps (with level-of-detail)
- 5000 nodes: 5fps (cluster mode only)

**Testing:**
- Unit: Force simulation converges to stable state
- Integration: Graph renders with 100, 500, 1000 nodes
- Performance: FPS benchmarks at each node count
- Visual: Nodes don't overlap, edges don't cross excessively

**Files:**
- `crates/frontend/src/pages/graph.rs` (major rewrite)
- `crates/frontend/src/graph/force.rs` (new)
- `crates/frontend/src/graph/canvas.rs` (new)
- `crates/frontend/src/graph/layout.rs` (new)

**Quality Gate:** 500-node graph renders at 30fps on mid-range hardware.

---

### C2: Embed Blocks (Week 17)

**Problem:** No support for embedding external content.

**Solution:**
- YouTube: `![youtube](VIDEO_ID)` -> iframe embed
- Figma: `![figma](URL)` -> iframe embed
- Mermaid: `![mermaid](CODE)` -> SVG rendering (existing)
- GitHub Gist: `![gist](URL)` -> embedded gist
- CodePen: `![codepen](URL)` -> iframe embed
- Twitter/X: `![tweet](URL)` -> embedded tweet

**Security:**
- Only whitelisted domains in iframe src
- Sandbox attribute on all iframes
- No `allow-scripts` unless explicitly enabled
- CSP policy blocks untrusted embeds

**Testing:**
- Unit: Each embed type produces correct HTML
- Integration: Embeds render in preview and SSG
- Security: Untrusted URLs are blocked
- Performance: Embeds lazy-load (don't block page render)

**Files:**
- `crates/renderer/src/embeds.rs` (new)
- `crates/frontend/src/components/embed_preview.rs` (new)
- `crates/ssg/src/embeds.rs` (new SSG integration)

---

### C3: TOC Scroll Sync (Week 18)

**Problem:** TOC doesn't highlight current heading.

**Solution:**
- Use `IntersectionObserver` API to track heading visibility
- Update TOC highlight as user scrolls
- Smooth scroll to heading when TOC item clicked
- Collapse/expand nested headings

**Testing:**
- Visual: TOC highlight updates on scroll
- Interaction: Click TOC item scrolls to heading
- Performance: No jank during scroll

**Files:**
- `crates/frontend/src/components/table_of_contents.rs` (rewrite)

---

### C4: Webhook Management UI (Week 19)

**Problem:** Webhook delivery exists but no management UI.

**Solution:**
- Create `/settings/webhooks` page
- List existing webhooks with status
- Create new webhook with URL, events, secret
- Test webhook (send test payload)
- View webhook delivery history
- Enable/disable webhooks

**Testing:**
- CRUD: Create, read, update, delete webhooks
- Integration: Webhook fires on document create/update/delete
- Security: HMAC signature verification works

**Files:**
- `crates/frontend/src/pages/webhooks.rs` (new)
- `crates/server/src/routes/webhook.rs` (extend)

---

### C5: i18n for Frontend (Weeks 20-21)

**Problem:** SSG has i18n but frontend does not.

**Solution:**
- Extend existing `i18n/mod.rs` to cover all frontend strings
- Use ICU message format for plurals, date/number formatting
- Language switcher in settings
- Persist language preference in localStorage
- Support: EN, ZH, JA, DE, FR, ES, KO, PT (existing SSG languages)

**Architecture:**
```
Component -> t!("key") -> i18n::translate(key, locale) -> Translated string
                                      |
                                      v
                              Translation files
                              (one per locale)
```

**Testing:**
- Unit: All translation keys resolve
- Integration: Language switcher changes all visible text
- Visual: No text overflow with longer translations (DE, FR)

**Files:**
- `crates/frontend/src/i18n/mod.rs` (extend)
- `crates/frontend/src/i18n/translations/` (new directory)
- `crates/frontend/src/components/language_picker.rs` (new)

---

### C6: File Drag-and-Drop (Week 22)

**Problem:** Basic drop zone exists but doesn't upload images.

**Solution:**
- Detect image drops on editor area
- Upload image to server via `POST /api/v1/files/upload`
- Insert `![alt](uploaded-url)` markdown at cursor position
- Show upload progress indicator
- Handle paste events for clipboard images
- Compress images >1MB before upload

**Testing:**
- Unit: Image detection, upload, markdown insertion
- Integration: Drop image, verify upload, verify markdown in editor
- Performance: 5MB image uploads in <3 seconds
- Edge cases: Network failure, invalid file type, oversized file

**Files:**
- `crates/frontend/src/components/drop_zone.rs` (extend)
- `crates/server/src/routes/files/upload.rs` (extend)

---

### C7: Custom CSS Themes (Week 23)

**Problem:** Only color customization exists. Need full CSS themes.

**Solution:**
- Theme editor at `/settings/themes`
- CSS custom properties for all design tokens
- Import/export themes as JSON
- Theme marketplace (future)
- Preview panel showing theme applied to sample content

**Testing:**
- Unit: Theme application doesn't break layout
- Visual: Theme changes are immediate and consistent
- Export: Theme JSON round-trips correctly

**Files:**
- `crates/frontend/src/pages/themes.rs` (new)
- `crates/frontend/src/components/theme_editor.rs` (new)

---

### C8: Analytics Dashboard (Week 24)

**Problem:** No usage analytics for admins.

**Solution:**
- Dashboard at `/admin/analytics`
- Metrics: documents created/updated/deleted per day
- Metrics: active users per day/week/month
- Metrics: search queries per day
- Metrics: API request volume
- Charts using simple SVG (no external dependencies)
- Date range picker

**Testing:**
- Unit: Metrics calculation from audit log
- Integration: Dashboard loads with real data
- Performance: Dashboard loads in <2 seconds

**Files:**
- `crates/frontend/src/pages/analytics.rs` (new)
- `crates/server/src/routes/analytics.rs` (new)
- `crates/database/src/analytics.rs` (new)

---

## Phase D: Platform Expansion (Weeks 25-38)

### D1: Canvas/Whiteboard (Weeks 25-28)

**Problem:** No visual knowledge organization.

**Solution:**
- Canvas component with infinite scrolling
- Node types: text, image, link, document, shape
- Edge types: arrow, line, dotted
- Drag-and-drop to create/connect nodes
- Real-time collaboration on canvas (CRDT for node positions)
- Auto-layout algorithms (force-directed, hierarchical, radial)
- Export as PNG/SVG/PDF

**Architecture:**
```
Canvas State (CRDT) -> Renderer (Canvas API) -> User Interaction
      |                      |                        |
      v                      v                        v
  Node/Edge CRUD      Animation loop           Mouse/Touch events
  via Yrs CRDT        (requestAnimationFrame)  (drag, click, scroll)
```

**Node Types:**
| Type | Properties | Rendering |
|------|-----------|-----------|
| Text | content, fontSize, color | Rich text block |
| Image | src, alt, width, height | Rendered image |
| Link | url, title, description | Bookmark card |
| Document | documentId, title | Document preview |
| Shape | type(rect/circle/diamond), fill, stroke | Geometric shape |

**Testing:**
- Unit: Node CRUD, edge CRUD, auto-layout
- Integration: Two users edit canvas simultaneously
- Performance: 100 nodes at 60fps, 500 nodes at 30fps
- Persistence: Canvas saves/loads correctly

**Files:**
- `crates/frontend/src/canvas/mod.rs` (new module)
- `crates/frontend/src/canvas/node.rs` (new)
- `crates/frontend/src/canvas/edge.rs` (new)
- `crates/frontend/src/canvas/renderer.rs` (new)
- `crates/frontend/src/canvas/layout.rs` (new)
- `crates/frontend/src/pages/canvas.rs` (new)
- `crates/server/src/routes/canvas.rs` (new)
- `crates/database/src/canvas.rs` (new)

---

### D2: PWA Offline Support (Weeks 29-30)

**Problem:** PWA exists but doesn't support offline access.

**Solution:**
- Service worker caches all static assets
- Cache-first strategy for documents (read offline)
- Network-first strategy for API calls
- Offline queue for writes (sync when online)
- Conflict resolution using CRDT (already implemented)
- Background sync for pending changes

**Testing:**
- Unit: Service worker caches correctly
- Integration: Load page, go offline, verify cached content loads
- Integration: Make changes offline, verify they sync when online
- Performance: Cached page loads in <1 second

**Files:**
- `public/sw.js` (rewrite)
- `crates/frontend/src/offline.rs` (extend)
- `crates/frontend/src/sync_bridge.rs` (extend)

---

### D3: Push Notifications (Week 31)

**Problem:** No push notifications for mentions/comments.

**Solution:**
- Web Push API integration
- Server sends push via FCM (Android) and APNs (iOS via Web Push)
- Notification types: mention, comment, review, assignment
- User controls which notifications to receive
- Notification preferences page

**Testing:**
- Unit: Push subscription management
- Integration: Notification sent on document mention
- Edge cases: Push fails, subscription expired, browser blocks

**Files:**
- `crates/server/src/routes/push.rs` (new)
- `crates/frontend/src/push.rs` (new)
- `crates/frontend/src/pages/notification_settings.rs` (new)

---

### D4: Outliner Mode (Weeks 32-35)

**Problem:** Logseq-style outliner not available.

**Solution:**
- Outliner mode toggle in editor
- Bullet-based hierarchy with indentation
- Indent/outdent with Tab/Shift+Tab
- Move up/down with Alt+Arrow
- Collapse/expand children
- Block references (transclude blocks)
- Daily outliner (journal mode)

**Architecture:**
```
Outliner State -> Tree Structure -> Renderer -> DOM
      |              |                |           |
      v              v                v           v
  Cursor pos    Parent/child     Indentation    Nested
  tracking      relationships    calculation    bullet list
```

**Testing:**
- Unit: Indent/outdent, move, collapse/expand
- Integration: Outliner and editor modes switch cleanly
- Property-based: Any sequence of operations preserves tree integrity
- Performance: 1000-block outline at 60fps

**Files:**
- `crates/editor/src/outliner.rs` (new)
- `crates/frontend/src/components/outliner.rs` (new)
- `crates/frontend/src/pages/journal.rs` (new)

---

### D5: Flashcards/Spaced Repetition (Weeks 36-37)

**Problem:** No learning features.

**Solution:**
- Flashcard creation from document sections
- SRS algorithm (SM-2 or FSRS)
- Review queue with daily reminders
- Progress tracking
- Export/import flashcard decks

**Testing:**
- Unit: SM-2 algorithm produces correct intervals
- Integration: Create flashcard from heading, review it
- Property-based: SRS algorithm converges to correct intervals

**Files:**
- `crates/server/src/routes/flashcards.rs` (new)
- `crates/database/src/flashcard.rs` (new)
- `crates/frontend/src/pages/flashcards.rs` (new)

---

### D6: PDF Annotation (Week 38)

**Problem:** No PDF viewing/annotation.

**Solution:**
- PDF viewer component using `pdf.js` via wasm-bindgen
- Highlight, underline, strikethrough annotations
- Sticky notes on PDF pages
- Annotations linked to document
- Export annotated PDF

**Testing:**
- Unit: PDF renders correctly
- Integration: Annotations save/load
- Performance: 100-page PDF renders in <2 seconds

**Files:**
- `crates/frontend/src/components/pdf_viewer.rs` (new)
- `crates/frontend/src/components/pdf_annotation.rs` (new)

---

## Phase E: Enterprise & Scale (Weeks 39-50)

### E1: SCIM Provisioning (Weeks 39-40)

**Problem:** SCIM types exist but no runtime flow.

**Solution:**
- SCIM 2.0 server implementation
- Endpoints: `/scim/v2/Users`, `/scim/v2/Groups`
- Support: create, read, update, delete, patch
- Filter: `userName eq "..."`, `displayName co "..."`
- Bearer token authentication
- Test with Okta and Azure AD

**Testing:**
- Unit: SCIM request/response format
- Integration: Okta provisioning creates users
- Security: Unauthorized SCIM requests rejected

**Files:**
- `crates/server/src/routes/scim.rs` (rewrite)
- `crates/server/src/scim/` (new module)

---

### E2: SOC 2 Automation (Weeks 41-44)

**Problem:** SOC 2 checklist is static. Needs automation.

**Solution:**
- Automated evidence collection from audit log
- Access control evidence (who accessed what, when)
- Change management evidence (deployments, code reviews)
- Monitoring evidence (uptime, alerts, incidents)
- Generate SOC 2 report PDF
- Continuous compliance monitoring

**Testing:**
- Unit: Evidence collection for each control
- Integration: Report generated with all evidence
- Validation: Report meets SOC 2 Type II requirements

**Files:**
- `crates/server/src/compliance/soc2.rs` (rewrite)
- `crates/server/src/compliance/evidence.rs` (new)
- `crates/server/src/routes/compliance/report.rs` (new)

---

### E3: E2E Encryption (Weeks 45-47)

**Problem:** E2E encryption is scaffolding only.

**Solution:**
- Client-side encryption using Web Crypto API
- Key generation: AES-256-GCM per document
- Key exchange: X25519 key agreement
- Key backup: Encrypted key escrow
- Zero-knowledge: Server never sees plaintext
- Device key management: Multi-device support

**Architecture:**
```
User -> Generate key pair (X25519) -> Store public key on server
      |
      v
Document -> Generate document key (AES-256-GCM) -> Encrypt content
      |                                                    |
      v                                                    v
Encrypt doc key with user public key              Encrypted content
      |                                                    |
      v                                                    v
Store encrypted key + encrypted content         Send to server
```

**Testing:**
- Unit: Key generation, encryption, decryption
- Integration: Encrypted document can only be read by authorized user
- Property-based: Decryption(Encryption(key, plaintext)) == plaintext
- Security: Server cannot decrypt without user key

**Files:**
- `crates/frontend/src/crypto.rs` (new)
- `crates/server/src/routes/e2e.rs` (rewrite)
- `crates/frontend/src/components/encryption_indicator.rs` (new)

---

### E4: Versioned Documentation (Weeks 48-49)

**Problem:** SSG has basic versioning. Needs full version branches.

**Solution:**
- Version branches (like git branches for docs)
- Create version from current state
- Edit version independently
- Compare versions (diff view)
- Publish specific version
- Version rollback

**Testing:**
- Unit: Version create, edit, compare, rollback
- Integration: Published version matches expected content
- Performance: Version diff for 100-page doc in <5 seconds

**Files:**
- `crates/ssg/src/versioning.rs` (rewrite)
- `crates/frontend/src/pages/versions.rs` (new)
- `crates/frontend/src/components/version_diff.rs` (new)

---

### E5: Blog System (Week 50)

**Problem:** No blog for SSG sites.

**Solution:**
- Blog section in SSG with posts, categories, tags
- RSS/Atom feed generation
- Social media meta tags (Open Graph, Twitter Cards)
- Newsletter subscription (email integration)
- Comment system (Disqus or self-hosted)

**Testing:**
- Unit: RSS feed validates, meta tags correct
- Integration: Blog post appears in SSG output
- SEO: Lighthouse score >90

**Files:**
- `crates/ssg/src/blog.rs` (new)
- `crates/frontend/src/pages/blog.rs` (new)
- `crates/server/src/routes/blog.rs` (new)

---

## Implementation Schedule

```
Week  1-6:   Phase A (Foundation) - PDF, Notion, Confluence, Admonitions
Week  7-14:  Phase B (DX) - Vim, MDX, Admonitions, Slash commands
Week 15-24:  Phase C (Collab) - Graph, Embeds, TOC, Webhooks, i18n, Drop, Themes, Analytics
Week 25-38:  Phase D (Platform) - Canvas, PWA, Push, Outliner, Flashcards, PDF annot
Week 39-50:  Phase E (Enterprise) - SCIM, SOC2, E2E crypto, Versions, Blog
```

**Total: 50 weeks (12.5 months)**

---

## Resource Requirements

| Phase | Weeks | Senior Dev | Junior Dev | Frontend | Security | Total |
|-------|-------|------------|------------|----------|----------|-------|
| A | 6 | 1 | 0 | 0 | 0 | 1 |
| B | 8 | 1 | 1 | 0 | 0 | 2 |
| C | 10 | 1 | 0 | 1 | 0 | 2 |
| D | 14 | 1 | 1 | 1 | 0 | 3 |
| E | 12 | 1 | 0 | 0 | 1 | 2 |
| **Total** | **50** | **5** | **2** | **2** | **1** | **10** |

---

## Quality Gates

### Per-Feature Quality Gates
- [ ] Unit tests pass (100% of new code)
- [ ] Integration tests pass
- [ ] E2E test added
- [ ] Performance benchmark meets target
- [ ] Security review complete
- [ ] Documentation written
- [ ] Clippy clean (0 warnings)
- [ ] Code review approved

### Per-Phase Quality Gates
- [ ] All features in phase complete
- [ ] All quality gates pass
- [ ] Regression test suite passes
- [ ] No critical bugs open
- [ ] Performance regression check passes
- [ ] Documentation complete

### Release Quality Gates
- [ ] All phases complete
- [ ] 100% test coverage on critical paths
- [ ] Formal verification for safety-critical algorithms
- [ ] Security audit complete
- [ ] Performance benchmarks documented
- [ ] Migration guide written
- [ ] Changelog complete

---

## Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Canvas complexity exceeds estimates | High | High | Start with minimal viable canvas, iterate |
| Vim keybinding scope creep | Medium | Medium | Implement only 20 most-used motions |
| MDX parser correctness | Medium | High | Use existing MDX test suite as reference |
| PWA offline sync conflicts | High | High | CRDT handles conflicts, comprehensive testing |
| SCIM protocol compliance | Low | High | Use existing SCIM library, test with Okta/Azure |
| E2E encryption key management | Medium | Critical | Start with single-device, add multi-device later |
| Performance regression | Medium | High | Benchmark before/after each feature |
| Security vulnerability in new code | Medium | Critical | Security review for every new endpoint |

---

## Success Criteria

| Metric | Current | Target (v3.0) |
|--------|---------|---------------|
| Features implemented | 103 | 130 |
| Test coverage | 1,504 tests | 2,500+ tests |
| E2E tests | 12 | 50+ |
| Competitor parity | 75% | 98% |
| Performance (API p99) | <200ms | <100ms |
| Security vulnerabilities | 0 critical | 0 critical |
| Documentation pages | 20 | 60+ |
| Supported languages | 8 | 8 (maintained) |
