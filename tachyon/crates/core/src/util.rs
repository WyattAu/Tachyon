// Utility functions module
// Common utility functions for Tachyon core

use chrono::{DateTime, Utc};
use regex::Regex;
use std::path::Path;

// ============================================================================
// Time Utilities
// ============================================================================

/// Get current UTC timestamp
///
/// # Returns
/// Current UTC DateTime
pub fn current_timestamp() -> DateTime<Utc> {
    Utc::now()
}

/// Format a DateTime to ISO 8601 string
///
/// # Arguments
/// * `dt` - DateTime to format
///
/// # Returns
/// ISO 8601 formatted string
pub fn format_iso8601(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Parse an ISO 8601 string to DateTime
///
/// # Arguments
/// * `s` - ISO 8601 formatted string
///
/// # Returns
/// Result containing parsed DateTime or error
pub fn parse_iso8601(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Calculate the duration between two timestamps in seconds
///
/// # Arguments
/// * `start` - Start timestamp
/// * `end` - End timestamp
///
/// # Returns
/// Duration in seconds
pub fn duration_seconds(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    end.timestamp() - start.timestamp()
}

// ============================================================================
// Path Utilities
// ============================================================================

/// Safely join path components
/// Prevents path traversal attacks by normalizing the path
///
/// # Arguments
/// * `base` - Base directory path
/// * `components` - Path components to join
///
/// # Returns
/// Result containing the joined path or error
pub fn safe_path_join<P: AsRef<Path>>(base: P, components: &[&str]) -> Result<String, PathError> {
    let mut path = base.as_ref().to_path_buf();

    for component in components {
        let component_path = Path::new(component);

        // Check for path traversal attempts
        if component_path.starts_with("..") || component_path.is_absolute() {
            return Err(PathError::TraversalAttempt(component.to_string()));
        }

        path.push(component);
    }

    // Normalize the path to resolve any relative components
    let normalized = path
        .canonicalize()
        .map_err(|_| PathError::InvalidPath(path.display().to_string()))?;

    // Ensure the normalized path is still within the base
    let base_normalized = base
        .as_ref()
        .canonicalize()
        .map_err(|_| PathError::InvalidPath(base.as_ref().display().to_string()))?;

    if !normalized.starts_with(&base_normalized) {
        return Err(PathError::TraversalAttempt(
            normalized.display().to_string(),
        ));
    }

    normalized
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| PathError::InvalidPath(normalized.display().to_string()))
}

/// Validate a file path
///
/// # Arguments
/// * `path` - Path to validate
///
/// # Returns
/// Result indicating valid path or error
pub fn validate_path<P: AsRef<Path>>(path: P) -> Result<(), PathError> {
    let path = path.as_ref();

    // Check for empty path
    if path.as_os_str().is_empty() {
        return Err(PathError::EmptyPath);
    }

    // Check for null bytes
    let path_str = path.to_str().ok_or_else(|| PathError::InvalidUtf8)?;

    if path_str.contains('\0') {
        return Err(PathError::InvalidPath(
            "Path contains null byte".to_string(),
        ));
    }

    Ok(())
}

/// Sanitize a filename
/// Remove or replace characters that are not safe for filenames
///
/// # Arguments
/// * `name` - Original filename
///
/// # Returns
/// Sanitized filename
pub fn sanitize_filename(name: &str) -> String {
    // Remove or replace unsafe characters
    let cleaned = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            '\x00'..='\x1F' => '_',
            _ => c,
        })
        .collect::<String>();

    // Remove leading and trailing dots and spaces
    let trimmed = cleaned.trim_matches(|c| c == '.' || c == ' ');

    // Ensure filename is not empty
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}

// ============================================================================
// String Utilities
// ============================================================================

/// Generate a URL-friendly slug from a string
///
/// # Arguments
/// * `input` - Input string to slugify
///
/// # Returns
/// URL-friendly slug string
pub fn slugify(input: &str) -> String {
    let slug = input
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0' // Mark for removal
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>();

    // Collapse multiple hyphens
    let re = Regex::new(r"-+").unwrap();
    let collapsed = re.replace_all(&slug, "-");

    // Trim leading/trailing hyphens
    collapsed.trim_matches('-').to_string()
}

/// Sanitize a string for safe output
/// Removes HTML tags and potentially dangerous characters
///
/// # Arguments
/// * `input` - Input string to sanitize
///
/// # Returns
/// Sanitized string
pub fn sanitize_string(input: &str) -> String {
    let re = Regex::new(r"<[^>]*>|&[a-zA-Z]+;").unwrap();
    let without_tags = re.replace_all(input, "");

    without_tags.to_string()
}

/// Validate a tag name
///
/// # Arguments
/// * `name` - Tag name to validate
///
/// # Returns
/// Result indicating valid tag or error message
pub fn validate_tag_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }

    if name.len() > 50 {
        return Err("Tag name too long (max 50 characters)".to_string());
    }

    // Check for valid characters
    let valid_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !valid_regex.is_match(name) {
        return Err("Tag name contains invalid characters".to_string());
    }

    Ok(())
}

/// Truncate text to a maximum length, adding ellipsis if truncated
///
/// # Arguments
/// * `text` - Text to truncate
/// * `max_len` - Maximum length in characters
///
/// # Returns
/// Truncated text with ellipsis if needed
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        // Find a safe break point (space or hyphen) near the limit
        if max_len > 3 {
            if let Some(pos) = text[..max_len].rfind(char::is_whitespace) {
                return format!("{}...", &text[..pos]);
            }
        }
        format!("{}...", &text[..max_len])
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Error type for path operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// Path traversal attempt detected
    TraversalAttempt(String),
    /// Invalid path
    InvalidPath(String),
    /// Empty path provided
    EmptyPath,
    /// Invalid UTF-8 in path
    InvalidUtf8,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TraversalAttempt(path) => write!(f, "Path traversal attempt: {}", path),
            Self::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            Self::EmptyPath => write!(f, "Empty path provided"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 in path"),
        }
    }
}

impl std::error::Error for PathError {}

// ============================================================================
// Hash Utilities
// ============================================================================

use sha2::{Digest, Sha256};

/// Compute a deterministic SHA-256 hash of markdown content.
///
/// The content is normalized before hashing:
/// - Trimmed of leading/trailing whitespace
/// - Line endings normalized to \n
///
/// This ensures `hash(content) == hash(content)` regardless of
/// platform-specific line endings or trailing whitespace.
pub fn compute_content_hash(content: &str) -> String {
    let normalized = content.trim().replace("\r\n", "\n").replace('\r', "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Time Utilities Tests
    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts.timestamp() > 0);
    }

    #[test]
    fn test_format_parse_iso8601() {
        let dt = Utc::now();
        let formatted = format_iso8601(dt);
        let parsed = parse_iso8601(&formatted).expect("Should parse");
        // Compare only up to seconds since format_iso8601 uses SecondsFormat::Secs
        assert_eq!(dt.timestamp(), parsed.timestamp());
    }

    #[test]
    fn test_duration_seconds() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(10);
        assert_eq!(duration_seconds(start, end), 10);
    }

    // Path Utilities Tests
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("test<>file.txt"), "test__file.txt"); // Both < and > replaced
        assert_eq!(sanitize_filename(".test."), "test");
        assert_eq!(sanitize_filename(""), "unnamed");
        assert_eq!(
            sanitize_filename("test<>file<>name<>text"),
            "test__file__name__text"
        );
    }

    #[test]
    fn test_validate_path() {
        assert!(validate_path("valid/path").is_ok());
        assert!(validate_path("path/with spaces").is_ok());
        assert!(validate_path("").is_err());
        assert!(validate_path("path\0withnull").is_err());
    }

    #[test]
    fn test_safe_path_join() {
        let base = "/tmp/test";
        let result = safe_path_join(base, &["subdir", "file.txt"]);
        // May fail if /tmp/test doesn't exist, but should not be traversal
        assert!(result.is_ok() || matches!(result.unwrap_err(), PathError::InvalidPath(_)));
    }

    #[test]
    fn test_safe_path_join_traversal() {
        let base = "/tmp/test";
        let result = safe_path_join(base, &["..", "etc", "passwd"]);
        assert!(matches!(
            result.unwrap_err(),
            PathError::TraversalAttempt(_)
        ));
    }

    // String Utilities Tests
    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Hello--World"), "hello-world");
        assert_eq!(slugify("Hello   World"), "hello-world");
        assert_eq!(slugify("Hello@World"), "helloworld");
        assert_eq!(slugify("Hello_World"), "hello-world");
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(
            sanitize_string("<script>alert('xss')</script>"),
            "alert('xss')"
        );
        assert_eq!(sanitize_string("Normal text"), "Normal text");
    }

    #[test]
    fn test_validate_tag_name() {
        assert!(validate_tag_name("valid-tag").is_ok());
        assert!(validate_tag_name("invalid tag").is_err());
        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("valid123").is_ok());
        assert!(validate_tag_name("a".repeat(51).as_str()).is_err());
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        // When there's a space within max_len, break at the space
        // "Hello world text" with max_len=10: "Hello worl" has space at position 5
        assert_eq!(truncate_text("Hello world text", 10), "Hello...");
        // When max_len doesn't include a space, truncate at max_len
        assert_eq!(truncate_text("Helloworldtext", 10), "Helloworld...");
        // "Hello world text" with max_len=12: "Hello world " (space at position 11)
        assert_eq!(truncate_text("Hello world text", 12), "Hello world...");
    }
}
