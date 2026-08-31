use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MergeResult {
    Clean(String),
    Conflicted {
        content: String,
        conflict_count: usize,
    },
}

fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();
    if m == 0 || n == 0 {
        return Vec::new();
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

#[derive(Debug, Clone)]
struct Change {
    base_start: usize,
    base_end: usize,
    new_lines: Vec<String>,
}

fn compute_changes(base: &[&str], changed: &[&str]) -> Vec<Change> {
    let mut changes = Vec::new();
    let lcs = longest_common_subsequence(base, changed);

    let mut bi = 0usize;
    let mut ci = 0usize;
    let mut pending_base_start: Option<usize> = None;
    let mut pending_new_lines: Vec<String> = Vec::new();

    for &lcs_line in &lcs {
        while bi < base.len() && base[bi] != lcs_line {
            if pending_base_start.is_none() {
                pending_base_start = Some(bi);
            }
            bi += 1;
        }
        while ci < changed.len() && changed[ci] != lcs_line {
            if pending_base_start.is_none() {
                pending_base_start = Some(bi);
            }
            pending_new_lines.push(changed[ci].to_string());
            ci += 1;
        }

        if pending_base_start.is_some() || !pending_new_lines.is_empty() {
            let start = pending_base_start.unwrap_or(bi);
            changes.push(Change {
                base_start: start,
                base_end: bi,
                new_lines: std::mem::take(&mut pending_new_lines),
            });
            pending_base_start = None;
        }

        bi += 1;
        ci += 1;
    }

    while bi < base.len() || ci < changed.len() {
        if pending_base_start.is_none() {
            pending_base_start = Some(bi);
        }
        if ci < changed.len() {
            pending_new_lines.push(changed[ci].to_string());
            ci += 1;
        }
        bi += 1;
    }

    if pending_base_start.is_some() || !pending_new_lines.is_empty() {
        let start = pending_base_start.unwrap_or(bi);
        changes.push(Change {
            base_start: start,
            base_end: bi,
            new_lines: std::mem::take(&mut pending_new_lines),
        });
    }

    changes
}

fn ranges_overlap(s1: usize, e1: usize, s2: usize, e2: usize) -> bool {
    s1 < e2 && s2 < e1
}

pub fn merge3(base: &str, ours: &str, theirs: &str) -> MergeResult {
    if base == ours && base == theirs {
        return MergeResult::Clean(base.to_string());
    }
    if base == theirs {
        return MergeResult::Clean(ours.to_string());
    }
    if base == ours {
        return MergeResult::Clean(theirs.to_string());
    }
    if ours == theirs {
        return MergeResult::Clean(ours.to_string());
    }

    let base_lines: Vec<&str> = base.lines().collect();
    let ours_lines: Vec<&str> = ours.lines().collect();
    let theirs_lines: Vec<&str> = theirs.lines().collect();

    let ours_changes = compute_changes(&base_lines, &ours_lines);
    let theirs_changes = compute_changes(&base_lines, &theirs_lines);

    let mut result_lines: Vec<String> = Vec::new();
    let mut conflict_count = 0usize;
    let mut base_pos = 0usize;

    let mut oi = 0usize;
    let mut ti = 0usize;

    while oi < ours_changes.len() || ti < theirs_changes.len() {
        let ours_next = if oi < ours_changes.len() {
            Some(&ours_changes[oi])
        } else {
            None
        };
        let theirs_next = if ti < theirs_changes.len() {
            Some(&theirs_changes[ti])
        } else {
            None
        };

        match (ours_next, theirs_next) {
            (Some(oc), None) => {
                while base_pos < oc.base_start && base_pos < base_lines.len() {
                    result_lines.push(base_lines[base_pos].to_string());
                    base_pos += 1;
                }
                for line in &oc.new_lines {
                    result_lines.push(line.clone());
                }
                base_pos = oc.base_end;
                oi += 1;
            }
            (None, Some(tc)) => {
                while base_pos < tc.base_start && base_pos < base_lines.len() {
                    result_lines.push(base_lines[base_pos].to_string());
                    base_pos += 1;
                }
                for line in &tc.new_lines {
                    result_lines.push(line.clone());
                }
                base_pos = tc.base_end;
                ti += 1;
            }
            (Some(oc), Some(tc)) => {
                if !ranges_overlap(oc.base_start, oc.base_end, tc.base_start, tc.base_end) {
                    let oc_first = oc.base_start <= tc.base_start;

                    let (first_change, _second_change) = if oc_first { (oc, tc) } else { (tc, oc) };

                    while base_pos < first_change.base_start && base_pos < base_lines.len() {
                        result_lines.push(base_lines[base_pos].to_string());
                        base_pos += 1;
                    }
                    for line in &first_change.new_lines {
                        result_lines.push(line.clone());
                    }
                    base_pos = first_change.base_end;

                    if oc_first {
                        oi += 1;
                    } else {
                        ti += 1;
                    }
                } else {
                    while base_pos < oc.base_start.min(tc.base_start) && base_pos < base_lines.len()
                    {
                        result_lines.push(base_lines[base_pos].to_string());
                        base_pos += 1;
                    }

                    result_lines.push("<<<<<<< ours".to_string());
                    for line in &oc.new_lines {
                        result_lines.push(line.clone());
                    }
                    result_lines.push("=======".to_string());
                    for line in &tc.new_lines {
                        result_lines.push(line.clone());
                    }
                    result_lines.push(">>>>>>> theirs".to_string());

                    base_pos = oc.base_end.max(tc.base_end);
                    oi += 1;
                    ti += 1;
                    conflict_count += 1;
                }
            }
            (None, None) => break,
        }
    }

    while base_pos < base_lines.len() {
        result_lines.push(base_lines[base_pos].to_string());
        base_pos += 1;
    }

    if conflict_count > 0 {
        MergeResult::Conflicted {
            content: result_lines.join("\n"),
            conflict_count,
        }
    } else {
        MergeResult::Clean(result_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge3_no_changes() {
        let base = "line1\nline2\nline3";
        let result = merge3(base, base, base);
        assert_eq!(result, MergeResult::Clean(base.to_string()));
    }

    #[test]
    fn test_merge3_ours_changed_only() {
        let base = "line1\nline2\nline3";
        let ours = "line1\nmodified\nline3";
        let theirs = base;
        let result = merge3(base, ours, theirs);
        assert_eq!(result, MergeResult::Clean(ours.to_string()));
    }

    #[test]
    fn test_merge3_theirs_changed_only() {
        let base = "line1\nline2\nline3";
        let ours = base;
        let theirs = "line1\nmodified\nline3";
        let result = merge3(base, ours, theirs);
        assert_eq!(result, MergeResult::Clean(theirs.to_string()));
    }

    #[test]
    fn test_merge3_both_same_change() {
        let base = "line1\nline2\nline3";
        let ours = "line1\nmodified\nline3";
        let theirs = "line1\nmodified\nline3";
        let result = merge3(base, ours, theirs);
        match result {
            MergeResult::Clean(content) => assert_eq!(content, ours),
            _ => panic!("Expected clean merge"),
        }
    }

    #[test]
    fn test_merge3_conflict() {
        let base = "line1\nline2\nline3";
        let ours = "line1\nours_change\nline3";
        let theirs = "line1\ntheirs_change\nline3";
        let result = merge3(base, ours, theirs);
        match result {
            MergeResult::Conflicted {
                content,
                conflict_count,
            } => {
                assert_eq!(conflict_count, 1);
                assert!(content.contains("<<<<<<< ours"));
                assert!(content.contains("ours_change"));
                assert!(content.contains("======="));
                assert!(content.contains("theirs_change"));
                assert!(content.contains(">>>>>>> theirs"));
            }
            MergeResult::Clean(_) => panic!("Expected conflict"),
        }
    }

    #[test]
    fn test_merge3_non_overlapping_changes() {
        let base = "line1\nline2\nline3\nline4\nline5";
        let ours = "line1\nmodified_by_ours\nline3\nline4\nline5";
        let theirs = "line1\nline2\nline3\nline4\nmodified_by_theirs";
        let result = merge3(base, ours, theirs);
        match result {
            MergeResult::Clean(content) => {
                assert!(content.contains("modified_by_ours"), "content: {}", content);
                assert!(
                    content.contains("modified_by_theirs"),
                    "content: {}",
                    content
                );
            }
            MergeResult::Conflicted {
                content,
                conflict_count,
            } => {
                panic!(
                    "Expected clean merge, got {} conflicts: {}",
                    conflict_count, content
                );
            }
        }
    }

    #[test]
    fn test_merge3_empty_base() {
        let base = "";
        let ours = "our content";
        let theirs = "their content";
        let result = merge3(base, ours, theirs);
        match result {
            MergeResult::Conflicted { conflict_count, .. } => {
                assert_eq!(conflict_count, 1);
            }
            MergeResult::Clean(_) => panic!("Expected conflict when both sides add to empty base"),
        }
    }

    #[test]
    fn test_merge3_multiple_conflicts() {
        let base = "a\nb\nc\nd\ne";
        let ours = "a\nB_ours\nc\nD_ours\ne";
        let theirs = "a\nB_theirs\nc\nD_theirs\ne";
        let result = merge3(base, ours, theirs);
        match result {
            MergeResult::Conflicted { conflict_count, .. } => {
                assert!(
                    conflict_count >= 2,
                    "Expected at least 2 conflicts, got {}",
                    conflict_count
                );
            }
            MergeResult::Clean(_) => panic!("Expected conflicts"),
        }
    }
}
