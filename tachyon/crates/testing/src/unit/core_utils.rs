//! Unit tests for core utility functions
//!
//! Tests for string sanitization, slug generation, time utilities,
//! path utilities, hash utilities, and validation functions.

#[allow(unused_imports)]
use tachyon_core::util::PathError;
#[allow(unused_imports)]
use tachyon_core::util::*;

#[test]
fn test_slugify_basic() {
    assert_eq!(slugify("Hello World"), "hello-world");
}

#[test]
fn test_slugify_multiple_spaces() {
    assert_eq!(slugify("Hello   World"), "hello-world");
}

#[test]
fn test_slugify_special_chars() {
    assert_eq!(slugify("Hello@World#Test"), "helloworldtest");
}

#[test]
fn test_slugify_underscores_and_hyphens() {
    assert_eq!(slugify("Hello_World-Test"), "hello-world-test");
}

#[test]
fn test_slugify_leading_trailing() {
    assert_eq!(slugify("--hello--"), "hello");
    assert_eq!(slugify("  hello  "), "hello");
}

#[test]
fn test_slugify_empty() {
    assert_eq!(slugify(""), "");
}

#[test]
fn test_sanitize_filename_basic() {
    assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
}

#[test]
fn test_sanitize_filename_special_chars() {
    assert_eq!(sanitize_filename("test<>file.txt"), "test__file.txt");
    assert_eq!(sanitize_filename(""), "unnamed");
}

#[test]
fn test_sanitize_filename_empty() {
    assert_eq!(sanitize_filename(""), "unnamed");
}

#[test]
fn test_sanitize_filename_dots() {
    assert_eq!(sanitize_filename(".hidden"), "hidden");
    assert_eq!(sanitize_filename("trailing."), "trailing");
}

#[test]
fn test_sanitize_filename_control_chars() {
    assert_eq!(sanitize_filename("test\x00file"), "test_file");
}

#[test]
fn test_sanitize_string_html() {
    assert_eq!(
        sanitize_string("<script>alert('xss')</script>"),
        "alert('xss')"
    );
}

#[test]
fn test_sanitize_string_html_entities() {
    let result = sanitize_string("text &amp; more");
    assert!(!result.contains("&amp;"));
    assert!(result.contains("text") && result.contains("more"));
}

#[test]
fn test_sanitize_string_plain() {
    assert_eq!(sanitize_string("Normal text"), "Normal text");
}

#[test]
fn test_validate_tag_name_valid() {
    assert!(validate_tag_name("valid-tag").is_ok());
    assert!(validate_tag_name("valid_tag").is_ok());
    assert!(validate_tag_name("valid123").is_ok());
}

#[test]
fn test_validate_tag_name_empty() {
    assert!(validate_tag_name("").is_err());
}

#[test]
fn test_validate_tag_name_too_long() {
    assert!(validate_tag_name(&"a".repeat(51)).is_err());
    assert!(validate_tag_name(&"a".repeat(50)).is_ok());
}

#[test]
fn test_validate_tag_name_invalid_chars() {
    assert!(validate_tag_name("invalid tag").is_err());
    assert!(validate_tag_name("invalid@tag").is_err());
}

#[test]
fn test_validate_path_valid() {
    assert!(validate_path("valid/path").is_ok());
    assert!(validate_path("path/with spaces").is_ok());
}

#[test]
fn test_validate_path_empty() {
    assert!(validate_path("").is_err());
}

#[test]
fn test_validate_path_null_byte() {
    assert!(validate_path("path\0withnull").is_err());
}

#[test]
fn test_truncate_text_short() {
    assert_eq!(truncate_text("short", 10), "short");
}

#[test]
fn test_truncate_text_exact() {
    assert_eq!(truncate_text("exactly10!", 10), "exactly10!");
}

#[test]
fn test_truncate_text_long() {
    assert_eq!(truncate_text("Hello world text", 10), "Hello...");
}

#[test]
fn test_truncate_text_no_space() {
    assert_eq!(truncate_text("Helloworldtext", 10), "Helloworld...");
}

#[test]
fn test_truncate_text_zero_max() {
    assert_eq!(truncate_text("hello", 0), "...");
}

#[test]
fn test_format_parse_iso8601() {
    let dt = chrono::Utc::now();
    let formatted = format_iso8601(dt);
    let parsed = parse_iso8601(&formatted).expect("should parse");
    assert_eq!(dt.timestamp(), parsed.timestamp());
}

#[test]
fn test_parse_iso8601_invalid() {
    assert!(parse_iso8601("not-a-date").is_err());
}

#[test]
fn test_duration_seconds() {
    let start = chrono::Utc::now();
    let end = start + chrono::Duration::seconds(10);
    assert_eq!(duration_seconds(start, end), 10);
}

#[test]
fn test_duration_seconds_negative() {
    let start = chrono::Utc::now();
    let end = start - chrono::Duration::seconds(5);
    assert_eq!(duration_seconds(start, end), -5);
}

#[test]
fn test_compute_content_hash_deterministic() {
    let h1 = compute_content_hash("hello world");
    let h2 = compute_content_hash("hello world");
    assert_eq!(h1, h2);
}

#[test]
fn test_compute_content_hash_different() {
    let h1 = compute_content_hash("hello world");
    let h2 = compute_content_hash("hello moon");
    assert_ne!(h1, h2);
}

#[test]
fn test_compute_content_hash_line_endings() {
    let h1 = compute_content_hash("hello\nworld");
    let h2 = compute_content_hash("hello\r\nworld");
    let h3 = compute_content_hash("hello\rworld");
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

#[test]
fn test_compute_content_hash_trimming() {
    let h1 = compute_content_hash("hello  ");
    let h2 = compute_content_hash("hello");
    assert_eq!(h1, h2);
}

#[test]
fn test_path_error_display() {
    let err = PathError::TraversalAttempt("..".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("traversal"));

    let err = PathError::EmptyPath;
    let msg = format!("{}", err);
    assert!(msg.contains("Empty"));
}
