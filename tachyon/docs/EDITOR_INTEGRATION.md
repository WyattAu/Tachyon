# External editor integration

Tachyon vaults are plain Markdown directories, so they can be opened in VS Code, Neovim, Zed, JetBrains, or any other editor.

## Pull and push

```bash
export DATABASE_URL=postgresql://...
tachyon pull ./vault
tachyon push ./vault --dry-run
tachyon push ./vault
```

`pull` writes one Markdown file per document and creates `.tachyon-sync.json` with content hashes and timestamps. The manifest is useful for review and should normally be committed alongside the vault. `pull` refuses to write into a non-empty directory unless `--force` is supplied.

`push` parses YAML frontmatter, tags, embeds, and Obsidian wiki-links using the existing import pipeline. Existing documents are updated by slug. New documents are currently reported but not created because author/project ownership context must be supplied explicitly; use the authenticated import API for new-document creation until that context is configured.

Always run `--dry-run` before pushing a large change. Push does not delete documents that are absent locally. When a pulled document has changed on the server, push stops with a conflict; resolve it locally or use `--force` only after reviewing the overwrite.

## VS Code

Open the pulled directory:

```bash
code ./vault
```

The `tachyon` CLI is editor-agnostic. Frontmatter validation can be enabled with VS Code's YAML extension by associating `tachyon/docs/frontmatter.schema.json` with frontmatter documents. Wiki-links remain readable Markdown and are compatible with Obsidian-style extensions.

## Safety

- Do not put API keys in the vault or manifest.
- Review `git diff` before `push`.
- Use a separate staging database for testing sync.
- A future LSP will add wiki-link completion, go-to-definition, and broken-link diagnostics without changing the vault format.
