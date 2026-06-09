use serde::{Deserialize, Serialize};

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

    // Process ```csv blocks
    while let Some(start) = remaining.find("```csv") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 6..];
        if let Some(end) = after_open.find("```") {
            let csv_data = &after_open[..end];
            result.push_str(&csv_to_html_table(csv_data, ','));
            remaining = &after_open[end + 3..];
        } else {
            result.push_str("```csv");
            remaining = after_open;
        }
    }

    // Process ```tsv blocks
    while let Some(start) = remaining.find("```tsv") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 6..];
        if let Some(end) = after_open.find("```") {
            let tsv_data = &after_open[..end];
            result.push_str(&csv_to_html_table(tsv_data, '\t'));
            remaining = &after_open[end + 3..];
        } else {
            result.push_str("```tsv");
            remaining = after_open;
        }
    }

    result.push_str(remaining);
    result
}

fn csv_to_html_table(data: &str, delimiter: char) -> String {
    let lines: Vec<&str> = data.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut html = String::from("<table class=\"csv-table\">\n");

    // First line is header
    let headers: Vec<&str> = lines[0].split(delimiter).collect();
    html.push_str("  <thead>\n    <tr>\n");
    for header in &headers {
        html.push_str(&format!("      <th>{}</th>\n", html_escape(header.trim())));
    }
    html.push_str("    </tr>\n  </thead>\n");

    // Remaining lines are data rows
    if lines.len() > 1 {
        html.push_str("  <tbody>\n");
        for line in &lines[1..] {
            if line.trim().is_empty() {
                continue;
            }
            let cells: Vec<&str> = line.split(delimiter).collect();
            html.push_str("    <tr>\n");
            for cell in &cells {
                html.push_str(&format!("      <td>{}</td>\n", html_escape(cell.trim())));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </tbody>\n");
    }

    html.push_str("</table>\n");
    html
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
    fn test_passthrough_no_csv() {
        let input = r#"{"content":"# Hello\n\nSome text"}"#;
        let output = process(input);
        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_csv_block_replaced() {
        let input = r#"{"content":"# Data\n\n```csv\nName,Age\nAlice,30\nBob,25\n```\n\nDone"}"#;
        let output = process(input);
        assert!(output.contains("<table class=\"csv-table\">"));
        assert!(output.contains("<th>Name</th>"));
        assert!(output.contains("<th>Age</th>"));
        assert!(output.contains("<td>Alice</td>"));
        assert!(output.contains("<td>Bob</td>"));
        assert!(!output.contains("```csv"));
    }

    #[test]
    fn test_tsv_block_replaced() {
        let input = r#"{"content":"```tsv\nName\tAge\nAlice\t30\n```\n"}"#;
        let output = process(input);
        assert!(output.contains("<table class=\"csv-table\">"));
        assert!(output.contains("<th>Name</th>"));
        assert!(output.contains("<td>Alice</td>"));
    }

    #[test]
    fn test_csv_to_html_table() {
        let csv = "Name,Age\nAlice,30\nBob,25";
        let html = csv_to_html_table(csv, ',');
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("Name"));
        assert!(html.contains("Alice"));
        assert!(html.contains("Bob"));
    }

    #[test]
    fn test_csv_single_row() {
        let csv = "Name,Age";
        let html = csv_to_html_table(csv, ',');
        assert!(html.contains("<thead>"));
        assert!(!html.contains("<tbody>"));
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
