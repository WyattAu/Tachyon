//! Minimal stdio LSP transport backed by [`VaultIndex`].

use crate::lsp::VaultIndex;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Diagnostic, DiagnosticSeverity,
    GotoDefinitionResponse, InitializeResult, Location, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use serde_json::{Value, json};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn run_stdio(root: PathBuf) -> io::Result<()> {
    let index = VaultIndex::from_directory(&root)?;
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut stdout = io::stdout().lock();

    while let Some(request) = read_message(&mut input)? {
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");

        match method {
            "initialize" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, initialize_result())?;
                }
            }
            "shutdown" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, Value::Null)?;
                }
            }
            "exit" => break,
            "textDocument/completion" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, completion(&index, &request))?;
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, definition(&index, &request))?;
                }
            }
            "textDocument/didOpen" | "textDocument/didChange" => {
                if let Some(notification) = diagnostics_notification(&index, &request) {
                    write_notification(
                        &mut stdout,
                        "textDocument/publishDiagnostics",
                        notification,
                    )?;
                }
            }
            _ => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, Value::Null)?;
                }
            }
        }
    }

    Ok(())
}

fn initialize_result() -> Value {
    serde_json::to_value(InitializeResult {
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            completion_provider: Some(Default::default()),
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            ..Default::default()
        },
        server_info: None,
    })
    .unwrap_or(Value::Null)
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
        .and_then(|uri| file_path(uri))
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
    let Ok(uri) = Uri::from_str(&file_uri(&document.path)) else {
        return Value::Null;
    };
    serde_json::to_value(GotoDefinitionResponse::Scalar(Location::new(
        uri,
        Range::default(),
    )))
    .unwrap_or(Value::Null)
}

fn diagnostics_notification(index: &VaultIndex, request: &Value) -> Option<Value> {
    let uri_text = request
        .pointer("/params/textDocument/uri")
        .and_then(Value::as_str)?;
    let uri = Uri::from_str(uri_text).ok()?;
    let content = request
        .pointer("/params/textDocument/text")
        .and_then(Value::as_str)
        .or_else(|| {
            request
                .pointer("/params/contentChanges/0/text")
                .and_then(Value::as_str)
        })
        .map(str::to_owned)
        .or_else(|| file_path(uri_text).and_then(|path| std::fs::read_to_string(path).ok()))
        .unwrap_or_default();
    let diagnostics = index
        .diagnostics(&content)
        .into_iter()
        .map(|diagnostic| Diagnostic {
            range: Range::new(
                Position::new(diagnostic.line as u32, diagnostic.column as u32),
                Position::new(diagnostic.line as u32, diagnostic.column as u32 + 1),
            ),
            severity: Some(DiagnosticSeverity::WARNING),
            message: diagnostic.message,
            ..Default::default()
        })
        .collect();
    serde_json::to_value(PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    })
    .ok()
}

fn file_path(uri: &str) -> Option<PathBuf> {
    let uri = Uri::from_str(uri).ok()?;
    uri.to_string().strip_prefix("file://").map(PathBuf::from)
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            );
        }
    }
    let length = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "LSP message missing Content-Length",
        )
    })?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

fn write_response(output: &mut impl Write, id: Value, result: Value) -> io::Result<()> {
    write_message(output, &json!({"jsonrpc":"2.0","id":id,"result":result}))
}

fn write_notification(output: &mut impl Write, method: &str, params: Value) -> io::Result<()> {
    write_message(
        output,
        &json!({"jsonrpc":"2.0","method":method,"params":params}),
    )
}

fn write_message(output: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
    output.write_all(&body)?;
    output.flush()
}
