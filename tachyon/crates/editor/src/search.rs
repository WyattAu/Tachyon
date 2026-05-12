use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::buffer::TextBuffer;
use crate::cursor::Cursor;
use crate::transaction::{EditKind, Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub line_text: String,
    pub match_text: String,
}

pub(crate) struct Search {
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
}

impl Search {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
        }
    }

    fn build_regex(&self, query: &str) -> Option<Regex> {
        if query.is_empty() {
            return None;
        }

        let pattern = if self.use_regex {
            query.to_string()
        } else {
            regex::escape(query)
        };

        let pattern = if self.whole_word {
            format!(r"\b{}\b", pattern)
        } else {
            pattern
        };

        let regex = if self.case_sensitive {
            Regex::new(&pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        };

        regex.ok()
    }

    pub fn find(&self, query: &str, buffer: &TextBuffer) -> Vec<SearchResult> {
        let re = match self.build_regex(query) {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        for line_idx in 0..buffer.len_lines() {
            let line = buffer.line(line_idx);
            let line_text = line.trim_end_matches('\n');
            let line_results = Self::find_in_line_with_regex(&re, line_text, line_idx);
            results.extend(line_results);
        }
        results
    }

    /// Search within a single line.
    ///
    /// Reserved for future use: per-line incremental search.
    #[allow(dead_code)] // used by future search UI integration
    pub fn find_in_line(&self, query: &str, line: &str, line_num: usize) -> Vec<SearchResult> {
        let re = match self.build_regex(query) {
            Some(r) => r,
            None => return Vec::new(),
        };
        Self::find_in_line_with_regex(&re, line, line_num)
    }

    fn find_in_line_with_regex(re: &Regex, line: &str, line_num: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();
        for m in re.find_iter(line) {
            results.push(SearchResult {
                line: line_num,
                start_col: m.start(),
                end_col: m.end(),
                line_text: line.to_string(),
                match_text: m.as_str().to_string(),
            });
        }
        results
    }

    pub fn replace_next(
        &self,
        buffer: &mut TextBuffer,
        replacement: &str,
        current_match: &SearchResult,
    ) -> Option<Transaction> {
        let start = Cursor::new(current_match.line, current_match.start_col);
        let old_text = current_match.match_text.clone();

        buffer.delete_range(
            current_match.line,
            current_match.start_col,
            current_match.line,
            current_match.end_col,
        );
        buffer.insert(current_match.line, current_match.start_col, replacement);

        let new_col = current_match.start_col + replacement.len();
        Some(Transaction {
            kind: EditKind::Replace {
                old_text: old_text.clone(),
                new_text: replacement.to_string(),
            },
            start,
            end: Cursor::new(current_match.line, new_col),
            timestamp: 0,
        })
    }

    pub fn replace_all(
        &self,
        buffer: &mut TextBuffer,
        query: &str,
        replacement: &str,
    ) -> Vec<Transaction> {
        let re = match self.build_regex(query) {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut transactions = Vec::new();
        let lines: Vec<String> = (0..buffer.len_lines()).map(|i| buffer.line(i)).collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_text = line.trim_end_matches('\n');
            let matches: Vec<_> = re.find_iter(line_text).collect();
            if matches.is_empty() {
                continue;
            }

            for m in matches.iter().rev() {
                let abs_start = m.start();
                let abs_end = m.end();
                let old_text = m.as_str().to_string();

                buffer.delete_range(line_idx, abs_start, line_idx, abs_end);
                buffer.insert(line_idx, abs_start, replacement);

                transactions.push(Transaction {
                    kind: EditKind::Replace {
                        old_text,
                        new_text: replacement.to_string(),
                    },
                    start: Cursor::new(line_idx, abs_start),
                    end: Cursor::new(line_idx, abs_start + replacement.len()),
                    timestamp: 0,
                });
            }
        }

        transactions
    }

    /// Count total matches for a query in the buffer.
    ///
    /// Reserved for future use: match count display in search UI.
    #[allow(dead_code)] // used by future search UI integration
    pub fn count_matches(&self, query: &str, buffer: &TextBuffer) -> usize {
        self.find(query, buffer).len()
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_simple() {
        let buf = TextBuffer::from_str("hello world\nhello again");
        let search = Search::new();
        let results = search.find("hello", &buf);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].match_text, "hello");
        assert_eq!(results[0].line, 0);
        assert_eq!(results[1].line, 1);
    }

    #[test]
    fn find_case_insensitive() {
        let buf = TextBuffer::from_str("Hello HELLO hello");
        let search = Search::new();
        let results = search.find("hello", &buf);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn find_no_results() {
        let buf = TextBuffer::from_str("hello world");
        let search = Search::new();
        let results = search.find("xyz", &buf);
        assert!(results.is_empty());
    }

    #[test]
    fn replace_next_basic() {
        let mut buf = TextBuffer::from_str("hello world");
        let search = Search::new();
        let results = search.find("hello", &buf);
        let tx = search.replace_next(&mut buf, "hi", &results[0]).unwrap();
        assert_eq!(buf.to_string(), "hi world");
        assert!(matches!(tx.kind, EditKind::Replace { .. }));
    }

    #[test]
    fn replace_all_basic() {
        let mut buf = TextBuffer::from_str("hello hello hello");
        let search = Search::new();
        let txs = search.replace_all(&mut buf, "hello", "hi");
        assert_eq!(txs.len(), 3);
        assert_eq!(buf.to_string(), "hi hi hi");
    }

    #[test]
    fn count_matches() {
        let buf = TextBuffer::from_str("one two three one two");
        let search = Search::new();
        assert_eq!(search.count_matches("one", &buf), 2);
        assert_eq!(search.count_matches("four", &buf), 0);
    }

    #[test]
    fn find_in_line() {
        let search = Search::new();
        let results = search.find_in_line("hello", "hello world hello", 5);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn replace_all_empty_query() {
        let mut buf = TextBuffer::from_str("hello");
        let search = Search::new();
        let txs = search.replace_all(&mut buf, "", "x");
        assert!(txs.is_empty());
    }
}
