//! Minimal stdio LSP transport backed by [`VaultIndex`].

use crate::lsp::VaultIndex;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Diagnostic, DiagnosticSeverity,
    GotoDefinitionResponse, InitializeParams, InitializeResult, Location, Position,
    PublishDiagnosticsParams, Range, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Uri,
};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;

pub fn run_stdio(root: PathBuf) -> io::Result<()> {
    let index = VaultIndex::from_directory(&root)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => serde_json::to_value(InitializeResult {
                capabilities: ServerCapabilities {
                    text_document_sync: Some(TextDocumentSyncCapability::Kind(
                        TextDocumentSyncKind::FULL,
                    )),
                    completion_provider: Some(Default::default()),
                    definition_provider: Some(lsp_types::OneOf::Left(true)),
                    ..Default::default()
                },
                server_info: None,
            })
            .unwrap_or(Value::Null),
            "shutdown" => Value::Null,
            "textDocument/completion" => completion(&index, &request),
            "textDocument/definition" => definition(&index, &request),
            "textDocument/publishDiagnostics" => Value::Null,
            _ => Value::Null,
        };
        if request.get("id").is_some() {
            write_message(
                &mut stdout,
                &json!({"jsonrpc":"2.0","id":id,"result":result}),
            )?;
        }
    }
    Ok(())
}

fn completion(index: &VaultIndex, request: &Value) -> Value {
    let prefix = request
        .pointer("/params/context/triggerCharacter")
        .and_then(Value::as_str)
        .unwrap_or("");
    let items: Vec<CompletionItem> = index
        .completions(prefix)
        .into_iter()
        .map(|doc| CompletionItem {
            label: doc.slug,
            detail: Some(doc.title),
            kind: Some(CompletionItemKind::REFERENCE),
            ..Default::default()
        })
        .collect();
    serde_json::to_value(CompletionResponse::Array(items)).unwrap_or(Value::Null)
}

fn definition(index: &VaultIndex, request: &Value) -> Value {
    let target = request
        .pointer("/params/textDocument/uri")
        .and_then(Value::as_str)
        .and_then(|uri| Uri::from_str(uri).ok())
        .and_then(|url| url.to_string().strip_prefix("file://").map(PathBuf::from))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| {
            text.lines().find_map(|line| {
                let start = line.find("[[")?;
                let end = line[start + 2..].find("]]")?;
                Some(line[start + 2..start + 2 + end].to_string())
            })
        })
        .and_then(|target| index.definition(&target).cloned());
    definition_location(target)
}

fn definition_location(document: Option<crate::lsp::EditorDocument>) -> Value {
    let Some(document) = document else {
        return Value::Null;
    };
    let uri = match Uri::from_str(&format!("file://{}", document.path.display())) {
        Ok(uri) => uri,
        Err(_) => return Value::Null,
    };
    serde_json::to_value(GotoDefinitionResponse::Scalar(Location::new(
        uri,
        Range::default(),
    )))
    .unwrap_or(Value::Null)
}

fn write_message(output: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
    output.write_all(&body)?;
    output.flush()
}

#[allow(dead_code)]
fn _diagnostic(document: &str, index: &VaultIndex) -> PublishDiagnosticsParams {
    let diagnostics = index
        .diagnostics(document)
        .into_iter()
        .map(|d| Diagnostic {
            range: Range::new(
                Position::new(d.line as u32, d.column as u32),
                Position::new(d.line as u32, d.column as u32 + 1),
            ),
            severity: Some(DiagnosticSeverity::WARNING),
            message: d.message,
            ..Default::default()
        })
        .collect();
    PublishDiagnosticsParams {
        uri: Uri::from_str("file:///unknown.md").unwrap(),
        diagnostics,
        version: None,
    }
}

#[allow(dead_code)]
fn _initialize_params(_: InitializeParams) {}
