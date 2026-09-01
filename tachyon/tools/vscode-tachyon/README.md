# Tachyon VS Code client

This development extension starts the Tachyon LSP for Markdown files and adds
commands for pulling and pushing the current vault.

## Development

1. Install dependencies in this directory.
2. Open the repository in VS Code.
3. Run the extension from the Run and Debug view.
4. Set `tachyon.lspCommand` if `tachyon-lsp` is not on `PATH`.

The extension is intentionally private and not marketplace-ready yet. The
server executable must accept `--root <vault>` and speak LSP over stdio.
