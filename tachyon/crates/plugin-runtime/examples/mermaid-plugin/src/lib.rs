use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
struct PluginInput {
    content: String,
}

#[derive(Serialize)]
struct PluginOutput {
    content: String,
}

#[no_mangle]
pub extern "C" fn on_document_render(input_ptr: *const u8, input_len: usize) -> *mut u8 {
    let input_bytes = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input_str = match std::str::from_utf8(input_bytes) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let output = process(input_str);
    let output_bytes = output.into_bytes();
    let len = output_bytes.len() as u32;
    let ptr = output_bytes.as_ptr();
    std::mem::forget(output_bytes);

    let result = unsafe {
        let buf = libc::malloc(len as usize + 4) as *mut u8;
        if buf.is_null() {
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(ptr, buf.add(4), len as usize);
        std::ptr::copy_nonoverlapping(
            &len as *const u32 as *const u8,
            buf,
            4,
        );
        buf
    };
    result
}

fn process(input: &str) -> String {
    let parsed: PluginInput = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => {
            return serde_json::to_string(&PluginOutput {
                content: input.to_string(),
            })
            .unwrap_or_default()
        }
    };

    let content = &parsed.content;
    let mut result = String::with_capacity(content.len());
    let mut remaining = content.as_str();

    while let Some(start) = remaining.find("```mermaid") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 10..];

        if let Some(end) = after_open.find("```") {
            let mermaid_code = &after_open[..end];
            result.push_str(&format!(
                "<div class=\"mermaid\" data-mermaid=\"{}\">{}</div>",
                html_escape(mermaid_code),
                mermaid_code
            ));
            remaining = &after_open[end + 3..];
        } else {
            result.push_str("```mermaid");
            remaining = after_open;
        }
    }

    result.push_str(remaining);
    result
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_no_mermaid() {
        let input = r#"{"content":"# Hello\n\nSome text"}"#;
        let output = process(input);
        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_mermaid_block_replaced() {
        let input = r#"{"content":"# Title\n\n```mermaid\ngraph TD\n  A-->B\n```\n\nDone"}"#;
        let output = process(input);
        assert!(output.contains("<div class=\"mermaid\""));
        assert!(output.contains("graph TD"));
        assert!(output.contains("A-->B"));
        assert!(!output.contains("```mermaid"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<b>"), "&lt;b&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_invalid_json_passthrough() {
        let output = process("not json");
        assert_eq!(output, "not json");
    }
}
