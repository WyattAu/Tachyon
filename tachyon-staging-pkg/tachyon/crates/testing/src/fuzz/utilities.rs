//! Fuzzing tests for core utility functions
//!
//! Property-based tests using random input to verify utility functions
//! handle edge cases gracefully without panicking.

#[allow(unused_imports)]
use tachyon_core::util::*;

#[test]
fn test_slugify_random_input_no_panic() {
    let repeated_10k = "a".repeat(10000);
    let inputs = vec![
        "",
        "a",
        "Hello World",
        "   leading spaces   ",
        "---trailing dashes---",
        "UPPERCASE TEXT",
        "MiXeD CaSe",
        "text-with_underscores",
        "special!@#$%^&*()chars",
        "🔥 emoji text 🚀",
        "newlines\nand\ttabs",
        repeated_10k.as_str(),
        "   ",
        "---",
        "a b c d e f g h i j",
        "multiple   spaces   between   words",
        "path/to/file.txt",
        "C:\\Windows\\Path",
        "https://example.com/path?query=value",
    ];

    for input in &inputs {
        let result = slugify(input);
        assert!(!result.contains(' '));
        assert!(!result.contains('\t'));
        assert!(!result.contains('\n'));
    }
}

#[test]
fn test_sanitize_filename_random_input_no_panic() {
    let repeated_10k = "a".repeat(10000);
    let repeated_1k = "file".repeat(1000);
    let inputs = vec![
        "",
        "a",
        "normal.txt",
        "file with spaces.txt",
        "special<>:\"/\\|?*chars.txt",
        "\x00\x01\x02\x03control",
        ".hidden",
        "..",
        "....",
        "  spaces  ",
        "file\nwith\nnewlines",
        "🔥 emoji file.txt",
        repeated_10k.as_str(),
        "CON",
        "AUX",
        "NUL",
        repeated_1k.as_str(),
    ];

    for input in &inputs {
        let result = sanitize_filename(input);
        assert!(!result.is_empty());
        assert!(!result.contains('\x00'));
        assert!(!result.starts_with('.') || result == "unnamed");
        assert!(!result.ends_with('.') || result == "unnamed");
    }
}

#[test]
fn test_sanitize_string_random_input_no_panic() {
    let repeated_10k = "a".repeat(10000);
    let repeated_script = "<script>alert('xss')</script>".repeat(100);
    let inputs_with_matched_tags = [
        "",
        "normal text",
        "<script>alert('xss')</script>",
        "<b>bold</b>",
        "<div class=\"test\">content</div>",
        "&amp; &lt; &gt; &quot;",
        "text with <tag>inside</tag>",
        "text &amp; more entities",
        "<>",
        "</>",
        "self-closing <br />",
        repeated_10k.as_str(),
        repeated_script.as_str(),
    ];

    for input in &inputs_with_matched_tags {
        let result = sanitize_string(input);
        assert!(!result.contains('<'), "input: {:?}", input);
        assert!(!result.contains('>'), "input: {:?}", input);
    }

    let inputs_with_unmatched = ["multiple << >>> <<< tags", "unclosed <tag"];

    for input in &inputs_with_unmatched {
        let _ = sanitize_string(input);
    }
}

#[test]
fn test_validate_tag_name_random_input_no_panic() {
    let repeated_50 = "a".repeat(50);
    let repeated_51 = "a".repeat(51);
    let inputs = vec![
        "",
        "a",
        "valid-tag",
        "valid_tag",
        "valid123",
        "-starts-with-dash",
        "ends-with-dash-",
        repeated_50.as_str(),
        repeated_51.as_str(),
        "invalid tag",
        "invalid@tag",
        "invalid.tag",
        "invalid tag with spaces",
        "invalid\ntag",
        "tag-with-special!chars",
    ];

    for input in &inputs {
        let _ = validate_tag_name(input);
    }
}

#[test]
fn test_truncate_text_random_input_no_panic() {
    let repeated_10k = "a".repeat(10000);
    let inputs = vec![
        ("", 0),
        ("", 10),
        ("hello", 5),
        ("hello", 3),
        ("hello world", 5),
        ("hello world", 11),
        (repeated_10k.as_str(), 100),
        (repeated_10k.as_str(), 0),
        (repeated_10k.as_str(), 10000),
        ("hello world text", 0),
    ];

    for (text, max_len) in &inputs {
        let result = truncate_text(text, *max_len);
        if *max_len > 0 && text.len() > *max_len {
            assert!(result.ends_with("..."));
        }
    }
}

#[test]
fn test_validate_path_random_input_no_panic() {
    let repeated_10k = "a".repeat(10000);
    let paths = vec![
        "",
        "valid/path",
        "path/with spaces",
        "path\0withnull",
        "relative/path",
        "/absolute/path",
        "../traversal",
        "./current",
        repeated_10k.as_str(),
    ];

    for path in &paths {
        let _ = validate_path(path);
    }
}

#[test]
fn test_format_parse_iso8601_roundtrip() {
    let times = vec![
        chrono::Utc::now(),
        chrono::Utc::now() - chrono::Duration::days(365),
        chrono::Utc::now() + chrono::Duration::days(365),
        chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    ];

    for dt in &times {
        let formatted = format_iso8601(*dt);
        let parsed = parse_iso8601(&formatted).unwrap();
        assert_eq!(dt.timestamp(), parsed.timestamp());
    }
}

#[test]
fn test_compute_content_hash_deterministic_random() {
    let repeated_10k = "a".repeat(10000);
    let contents = vec![
        "",
        "a",
        "hello world",
        repeated_10k.as_str(),
        "content with\nnewlines\r\nand\r",
        "  spaces  ",
        "tabs\t\t",
        "<html>content</html>",
        "🔥 unicode content",
    ];

    for content in &contents {
        let h1 = compute_content_hash(content);
        let h2 = compute_content_hash(content);
        assert_eq!(h1, h2);
    }
}
