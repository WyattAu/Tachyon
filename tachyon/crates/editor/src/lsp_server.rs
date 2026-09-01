//! Minimal stdio LSP transport backed by [`VaultIndex`].

use crate::lsp::VaultIndex;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Diagnostic, DiagnosticSeverity,
    GotoDefinitionResponse, InitializeResult, Location, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn run_stdio(root: PathBuf) -> io::Result<()> {
    let index = VaultIndex::from_directory(&root)?;
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut stdout = io::stdout().lock();
    let mut documents = HashMap::<String, String>::new();

    while let Some(request) = read_message(&mut input)? {
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        match method {
            "initialize" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, initialize_result())?;
                }
            }
            "initialized" => {}
            "shutdown" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, Value::Null)?;
                }
            }
            "exit" => break,
            "textDocument/didOpen" => {
                if let Some(uri) = text_document_uri(&request) {
                    if let Some(text) = request
                        .pointer("/params/textDocument/text")
                        .and_then(Value::as_str)
                    {
                        documents.insert(uri.clone(), text.to_owned());
                    }
                    publish_diagnostics(
                        &mut stdout,
                        &index,
                        &uri,
                        documents.get(&uri).map(String::as_str),
                    )?;
                }
            }
            "textDocument/didChange" => {
                if let Some(uri) = text_document_uri(&request) {
                    if let Some(text) = request
                        .pointer("/params/contentChanges/0/text")
                        .and_then(Value::as_str)
                    {
                        documents.insert(uri.clone(), text.to_owned());
                    }
                    publish_diagnostics(
                        &mut stdout,
                        &index,
                        &uri,
                        documents.get(&uri).map(String::as_str),
                    )?;
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = text_document_uri(&request) {
                    documents.remove(&uri);
                    let params = serde_json::to_value(PublishDiagnosticsParams {
                        uri: Uri::from_str(&uri).map_err(io::Error::other)?,
                        diagnostics: vec![],
                        version: None,
                    })
                    .map_err(io::Error::other)?;
                    write_notification(&mut stdout, "textDocument/publishDiagnostics", params)?;
                }
            }
            "textDocument/completion" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, completion(&index, &request, &documents))?;
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    write_response(&mut stdout, id, definition(&index, &request, &documents))?;
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

fn completion(index: &VaultIndex, request: &Value, documents: &HashMap<String, String>) -> Value {
    let prefix = document_prefix(request, documents).unwrap_or_default();
    let items: Vec<CompletionItem> = index
        .completions(&prefix)
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

fn definition(index: &VaultIndex, request: &Value, documents: &HashMap<String, String>) -> Value {
    let target = document_prefix(request, documents)
        .and_then(|text| {
            let start = text.rfind("[[")?;
            let end = text[start + 2..].find("]]")?;
            Some(text[start + 2..start + 2 + end].to_string())
        })
        .and_then(|target| index.definition(&target).cloned());
    let Some(document) = target else {
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

fn document_prefix(request: &Value, documents: &HashMap<String, String>) -> Option<String> {
    let uri = text_document_uri(request)?;
    let text = documents
        .get(&uri)
        .cloned()
        .or_else(|| file_path(&uri).and_then(|p| std::fs::read_to_string(p).ok()))?;
    let line = request
        .pointer("/params/position/line")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let character = request
        .pointer("/params/position/character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    Some(
        text.lines()
            .take(line + 1)
            .last()
            .unwrap_or("")
            .chars()
            .take(character)
            .collect(),
    )
}

fn publish_diagnostics(
    output: &mut impl Write,
    index: &VaultIndex,
    uri: &str,
    text: Option<&str>,
) -> io::Result<()> {
    let diagnostics = index
        .diagnostics(text.unwrap_or(""))
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
    let params = serde_json::to_value(PublishDiagnosticsParams {
        uri: Uri::from_str(uri).map_err(io::Error::other)?,
        diagnostics,
        version: None,
    })
    .map_err(io::Error::other)?;
    write_notification(output, "textDocument/publishDiagnostics", params)
}

fn text_document_uri(request: &Value) -> Option<String> {
    request
        .pointer("/params/textDocument/uri")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn file_path(uri: &str) -> Option<PathBuf> {
    let value = uri.strip_prefix("file://")?;
    Some(PathBuf::from(percent_decode(value)))
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                output.push(byte);
                i += 3;
                continue;
            }
        }
        output.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
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
            content_length = Some(value.trim().parse::<usize>().map_err(io::Error::other)?);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_file_uri() {
        assert_eq!(
            file_path("file:///tmp/a%20b.md").unwrap(),
            PathBuf::from("/tmp/a b.md")
        );
    }
    #[test]
    fn frames_json_messages() {
        let body = br#"{"jsonrpc":"2.0","id":1}"#;
        let input = format!(
            "Content-Length: {}\r\n\r\n{}",
            body.len(),
            String::from_utf8_lossy(body)
        );
        let parsed = read_message(&mut input.as_bytes()).unwrap().unwrap();
        assert_eq!(parsed["id"], 1);
    }
}
