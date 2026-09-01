const vscode = require('vscode');
const cp = require('child_process');
const path = require('path');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function activate(context) {
  const command = vscode.workspace.getConfiguration('tachyon').get('lspCommand', 'tachyon-lsp');
  const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (root) {
    const serverOptions = () => cp.spawn(command, ['--root', root], { cwd: root });
    const clientOptions = {
      documentSelector: [{ scheme: 'file', language: 'markdown' }],
      synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/*.md') }
    };
    client = new LanguageClient('tachyonLsp', 'Tachyon LSP', serverOptions, clientOptions);
    context.subscriptions.push(client.start());
  }

  context.subscriptions.push(
    vscode.commands.registerCommand('tachyon.pull', () => runCli(context, 'pull')),
    vscode.commands.registerCommand('tachyon.push', () => runCli(context, 'push'))
  );
}

function runCli(context, action) {
  const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!root) return vscode.window.showErrorMessage('Open a Tachyon vault folder first.');
  const terminal = vscode.window.createTerminal('Tachyon');
  terminal.show();
  terminal.sendText(`tachyon ${action} ${shellQuote(root)}`);
}

function shellQuote(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function deactivate() {
  return client?.stop();
}

module.exports = { activate, deactivate };
