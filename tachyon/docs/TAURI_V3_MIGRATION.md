# Tauri v3 Migration Plan

## Current State

| Component | Version |
|---|---|
| tauri | 2.x (latest) |
| tauri-build | 2.x (latest) |
| tauri-plugin-shell | 2.x |
| tauri-plugin-dialog | 2.x |
| tauri-plugin-fs | 2.x |
| tauri-plugin-log | 2.x |
| tauri-plugin-notification | 2.x |
| Linux backend | GTK3 (libwebkit2gtk-4.0) |
| Config schema | `https://schema.tauri.app/config/2` |
| Features used | `protocol-asset`, `tray-icon`, `custom-protocol` |

### Tauri API Usage Summary

- **Commands**: 40+ `#[tauri::command]` handlers in `commands.rs`, `import_export.rs`
- **State management**: `tauri::State`, `app.manage()`, `app.try_state()`
- **Events**: `tauri::Emitter` trait, `app.emit()`
- **Window management**: `app.get_webview_window("main")`
- **Path resolver**: `app.path().app_data_dir()`
- **Tray**: `tauri::tray::TrayIconBuilder`, `tauri::menu::*`
- **Dialogs**: `tauri_plugin_dialog::DialogExt`
- **Async runtime**: `tauri::async_runtime::spawn`
- **IPC handler**: `tauri::generate_handler![]`
- **Build**: `tauri_build::build()`
- **Plugin init**: `.plugin(tauri_plugin_*::init())`

## Target State

| Component | Version |
|---|---|
| tauri | 3.x (NOT YET RELEASED) |
| tauri-build | 3.x |
| All plugins | 3.x |
| Linux backend | GTK4 + WebKitGTK 6.0 |
| Config schema | `https://schema.tauri.app/config/3` |

> **BLOCKER**: Tauri v3 is not yet published on crates.io. The [3.0 milestone](https://github.com/tauri-apps/tauri/milestone/5)
> is at 14% complete (0 of 12 issues closed, all still open as of May 2026).
> No v3 crates, config schema, or migration guide exist yet.
> This codebase is prepared for v3 but cannot migrate until it ships.

## Planned Breaking Changes (from Tauri v3 milestone)

Based on the [Tauri 3.0 milestone](https://github.com/tauri-apps/tauri/milestone/5) (12 open issues, 14% complete as of May 2026):

### Critical (High Impact)

| Change | Impact | Affected Files |
|---|---|---|
| **Linux: GTK3 → GTK4 + WebKitGTK 6.0** (#14684) | Requires `libwebkit2gtk-6.0-dev` on Linux; Ubuntu 24+ native support | Build system, CI |
| **Drop Windows 7 support** (#12550) | Only affects Win7 users; minimal code change | CI matrix |
| **Remove `v1Compatible` config option** (#12516) | Config cleanup; we don't use it | `tauri.conf.json` |
| **Remove UNC paths in Rust APIs** (#12551) | Windows path handling changes | `commands.rs`, `file_dialog.rs`, `filesystem.rs` |

### Moderate Impact

| Change | Impact | Affected Files |
|---|---|---|
| **Clean up top-level exports** (#14011) | Reorganization of `tauri::*` imports | All source files |
| **`Webview::navigate` accepts `WebviewUrl`** (#14586) | URL type changes | `lib.rs`, `commands.rs` |
| **`WindowBuilder::parent` marked `unsafe`** (#13973) | If we use parented windows | `tray.rs` (not currently affected) |
| **`Builder::setup` no longer panics on failure** (#12815) | Error handling change in setup | `lib.rs` |

### Low Impact (Internal)

| Change | Impact | Affected Files |
|---|---|---|
| Remove unused `goblin` dependency | None for consumers | `Cargo.toml` |
| Migrate schemars to 1.x | Config schema validation | Build system |
| macOS `.app.tar.gz` includes version | Bundle naming | CI, `tauri.conf.json` |

## Migration Steps (Dependency Order)

### Phase 0: Preparatory Changes (DONE)

**Completed: May 2026**

- [x] Create this migration plan
- [x] Create `compat.rs` abstraction layer (simplified — direct v2 API calls)
- [x] Remove `tauri-v3` feature flag (v3 not available; will re-add when needed)
- [x] Audit and document all Tauri API surface usage
- [x] Ensure `tauri.conf.json` uses fields compatible with both v2 and v3
- [x] Update workspace `tauri`/`tauri-build` from `2.0.0-beta` to `2` (latest)
- [x] Unpin `tauri` version in `src-tauri/Cargo.toml` from `2.10.0` to `2` (latest)
- [x] Verify `cargo check -p tachyon-desktop-app` passes cleanly (zero warnings)

### Phase 1: Version Bump (BLOCKED — awaiting v3 release)

**Estimated: 1-2 hours**

- [ ] Bump `tauri` to 3.x in `crates/desktop/src-tauri/Cargo.toml`
- [ ] Bump `tauri-build` to 3.x in both workspace and local
- [ ] Bump all plugins to 3.x
- [ ] Update config schema to v3
- [ ] Re-add `tauri-v3` feature flag if needed for conditional compilation
- [ ] Fix compile errors from API changes

### Phase 2: GTK4 Migration (Linux) (BLOCKED — awaiting v3 release)

**Estimated: 4-8 hours**

- [ ] Install `libwebkit2gtk-6.0-dev` in CI and dev environments
- [ ] Update build scripts for GTK4 pkg-config names
- [ ] Test tray icon under GTK4
- [ ] Test file dialogs under GTK4
- [ ] Verify WebKitGTK 6.0 rendering

### Phase 3: API Migration (BLOCKED — awaiting v3 release)

**Estimated: 2-4 hours**

- [ ] Update imports for top-level export changes
- [ ] Migrate `Webview::navigate` to `WebviewUrl`
- [ ] Handle `Builder::setup` error return changes
- [ ] Update UNC path handling for Windows
- [ ] Audit `unsafe` requirements for `WindowBuilder::parent`

### Phase 4: Plugin Migration (BLOCKED — awaiting v3 release)

**Estimated: 2-4 hours**

- [ ] Migrate `tauri-plugin-shell` to v3 API
- [ ] Migrate `tauri-plugin-dialog` to v3 API
- [ ] Migrate `tauri-plugin-fs` to v3 API
- [ ] Migrate `tauri-plugin-log` to v3 API
- [ ] Migrate `tauri-plugin-notification` to v3 API
- [ ] Update capabilities permissions format

### Phase 5: Testing & Validation (BLOCKED — awaiting v3 release)

**Estimated: 2-4 hours**

- [ ] Full CI pipeline passes
- [ ] Desktop app launches on Linux (GTK4)
- [ ] All 40+ IPC commands work correctly
- [ ] Tray icon and menus function
- [ ] File dialogs (open/save) work
- [ ] File watcher works
- [ ] Import/export works
- [ ] Embedded server starts and responds

**Total estimated effort: 13-25 hours (all phases blocked on v3 release)**

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Tauri v3 release timeline uncertain | High | Low | Phase 0 work is forward-compatible; no rush to complete |
| GTK4 not available on older Linux distros | Medium | High | Document minimum distro versions; CI uses Ubuntu 24+ |
| Plugin v3 APIs significantly different | Medium | Medium | Compat layer in `compat.rs` isolates plugin interactions |
| `Builder::setup` error handling breaks | Low | Medium | Already using `?` propagation; compat layer handles this |
| WebKitGTK 6.0 rendering differences | Low | High | Extensive manual testing; trunk build output may need changes |

## Testing Strategy

### Unit Tests
- Existing tests in `commands.rs`, `state.rs`, `events.rs`, `sync.rs`, etc. continue to pass
- Compat layer tests in `compat.rs` (current: `TAURI_VERSION` == 2)

### Integration Tests
- CI `cargo check -p tachyon-desktop-app` catches compile errors
- CI `cargo clippy --workspace --all-targets` catches warnings
- CI `cargo test --workspace --lib` runs unit tests

### Manual Tests (Post-Migration)
- Launch app on Linux (GTK4), macOS, Windows
- Test all tray menu actions
- Open/save file dialogs
- File watcher start/stop
- Import Obsidian vault
- Export Markdown/HTML ZIP
- Embedded server start/stop
- Authentication flow (online + offline)

## Notes

- Tauri v3 is currently in early development (milestone at 14%)
- No official migration guide exists yet
- No v3 crates published on crates.io (latest is 2.11.1)
- This plan will be updated as v3 matures
- Phase 0 is complete and all changes are on `main`
- The flake.nix already has `gtk4` and `webkitgtk_6_0` ready for the GTK4 transition
