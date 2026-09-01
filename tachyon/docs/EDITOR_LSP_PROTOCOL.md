# Editor intelligence

`tachyon-editor` exposes `VaultIndex`, a transport-independent core for editor adapters.

It currently supports:

- Markdown vault indexing
- Wiki-link completion by slug or title
- Wiki-link definition lookup
- Broken wiki-link diagnostics

This keeps editor behavior independent from VS Code or any particular protocol. The next transport adapter can expose these operations through LSP without duplicating parsing logic.

Recommended client behavior:

1. Run `tachyon pull ./vault`.
2. Build an index with the vault root.
3. Refresh the index on file creation, deletion, or rename.
4. Publish `VaultIndex::diagnostics` results after document changes.
5. Resolve `[[Target]]` through `VaultIndex::definition`.

The vault remains ordinary Markdown and does not require the editor adapter to render or rewrite document content.
