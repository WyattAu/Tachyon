//! CSV import for table-style document data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvImportRow {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvImportResult {
    pub rows: Vec<CsvImportRow>,
    pub headers: Vec<String>,
    pub total_rows: usize,
    pub has_headers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvImportOptions {
    pub delimiter: char,
    pub has_headers: bool,
    pub max_rows: usize,
}

impl Default for CsvImportOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            has_headers: true,
            max_rows: 10_000,
        }
    }
}

#[derive(Debug)]
pub enum CsvImportError {
    Io(std::io::Error),
    Parse(String),
    TooManyRows { limit: usize, actual: usize },
}

impl std::fmt::Display for CsvImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
            Self::TooManyRows { limit, actual } => {
                write!(f, "Too many rows: {} exceeds limit {}", actual, limit)
            }
        }
    }
}

impl From<std::io::Error> for CsvImportError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn parse_csv(
    input: &str,
    options: &CsvImportOptions,
) -> Result<CsvImportResult, CsvImportError> {
    let mut rows = Vec::new();
    let mut headers = Vec::new();
    let mut has_headers = false;

    for (i, line) in input.lines().enumerate() {
        if i >= options.max_rows + if options.has_headers { 1 } else { 0 } {
            return Err(CsvImportError::TooManyRows {
                limit: options.max_rows,
                actual: i,
            });
        }

        let fields: Vec<String> = parse_csv_line(line, options.delimiter)
            .into_iter()
            .map(|s| s.trim().to_string())
            .collect();

        if i == 0 && options.has_headers && !fields.is_empty() {
            headers = fields;
            has_headers = true;
            continue;
        }

        if !fields.is_empty() {
            rows.push(CsvImportRow { values: fields });
        }
    }

    let total_rows = rows.len();
    Ok(CsvImportResult {
        rows,
        headers,
        total_rows,
        has_headers,
    })
}

fn parse_csv_line(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            }
            '"' => {
                in_quotes = true;
            }
            c if c == delimiter && !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current);
    fields
}

pub fn csv_to_markdown_table(result: &CsvImportResult) -> String {
    if result.rows.is_empty() {
        return String::new();
    }

    let mut md = String::new();
    let col_count = result
        .rows
        .iter()
        .map(|r| r.values.len())
        .max()
        .unwrap_or(0);

    if result.has_headers {
        md.push('|');
        for (i, h) in result.headers.iter().enumerate() {
            if i > 0 {
                md.push('|');
            }
            md.push_str(h);
        }
        md.push('|');
        md.push('\n');
        md.push('|');
        for i in 0..result.headers.len() {
            if i > 0 {
                md.push('|');
            }
            md.push_str("---");
        }
        md.push('|');
        md.push('\n');
    } else {
        md.push('|');
        for i in 0..col_count {
            if i > 0 {
                md.push('|');
            }
            md.push_str(&format!("Col {}", i + 1));
        }
        md.push('|');
        md.push('\n');
        md.push('|');
        for i in 0..col_count {
            if i > 0 {
                md.push('|');
            }
            md.push_str("---");
        }
        md.push('|');
        md.push('\n');
    }

    for row in &result.rows {
        md.push('|');
        for (i, v) in row.values.iter().enumerate() {
            if i > 0 {
                md.push('|');
            }
            md.push_str(v);
        }
        md.push('|');
        md.push('\n');
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_csv_parse() {
        let result = parse_csv("a,b,c\n1,2,3\n4,5,6", &CsvImportOptions::default()).unwrap();
        assert_eq!(result.headers, vec!["a", "b", "c"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].values, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_csv_without_headers() {
        let opts = CsvImportOptions {
            has_headers: false,
            ..Default::default()
        };
        let result = parse_csv("1,2,3\n4,5,6", &opts).unwrap();
        assert!(result.headers.is_empty());
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_csv_with_quotes() {
        let result = parse_csv(
            "name,value\n\"hello, world\",42",
            &CsvImportOptions::default(),
        )
        .unwrap();
        assert_eq!(result.rows[0].values[0], "hello, world");
    }

    #[test]
    fn test_csv_to_markdown() {
        let result = CsvImportResult {
            headers: vec!["Name".to_string(), "Value".to_string()],
            rows: vec![
                CsvImportRow {
                    values: vec!["A".to_string(), "1".to_string()],
                },
                CsvImportRow {
                    values: vec!["B".to_string(), "2".to_string()],
                },
            ],
            total_rows: 2,
            has_headers: true,
        };
        let md = csv_to_markdown_table(&result);
        assert!(md.contains("|Name|Value|"));
        assert!(md.contains("|---|---|"));
        assert!(md.contains("|A|1|"));
    }

    #[test]
    fn test_row_limit_enforcement() {
        let opts = CsvImportOptions {
            max_rows: 1,
            ..Default::default()
        };
        let result = parse_csv("h\n1\n2\n3", &opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_semicolon_delimiter() {
        let opts = CsvImportOptions {
            delimiter: ';',
            ..Default::default()
        };
        let result = parse_csv("a;b;c\n1;2;3", &opts).unwrap();
        assert_eq!(result.headers, vec!["a", "b", "c"]);
        assert_eq!(result.rows[0].values, vec!["1", "2", "3"]);
    }
}
