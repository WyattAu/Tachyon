// Common validation utilities
// XSS prevention, HTML sanitization, and general validation helpers

use regex::Regex;
use std::sync::LazyLock;

static HTML_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());

static SCRIPT_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<\s*script[^>]*>.*?<\s*/\s*script\s*>").unwrap());

static JAVASCRIPT_PROTOCOL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*javascript:").unwrap());

static DATA_URI_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\s*data:").unwrap());

static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

static CONTROL_CHARS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]").unwrap());

static WHITESPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

static SLUG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    Required,
    TooShort { min: usize },
    TooLong { max: usize },
    InvalidFormat { message: String },
    InvalidEmail,
    InvalidUuid,
    InvalidUrl,
    ContainsHtml,
    ContainsScript,
    ContainsControlChars,
    InvalidLength { min: usize, max: usize },
    ForbiddenContent { reason: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "Field is required"),
            Self::TooShort { min } => write!(f, "Must be at least {} characters", min),
            Self::TooLong { max } => write!(f, "Must be at most {} characters", max),
            Self::InvalidFormat { message } => write!(f, "Invalid format: {}", message),
            Self::InvalidEmail => write!(f, "Invalid email address"),
            Self::InvalidUuid => write!(f, "Invalid UUID format"),
            Self::InvalidUrl => write!(f, "Invalid URL"),
            Self::ContainsHtml => write!(f, "HTML tags are not allowed"),
            Self::ContainsScript => write!(f, "Script content is not allowed"),
            Self::ContainsControlChars => write!(f, "Control characters are not allowed"),
            Self::InvalidLength { min, max } => {
                write!(f, "Must be between {} and {} characters", min, max)
            }
            Self::ForbiddenContent { reason } => write!(f, "Forbidden content: {}", reason),
        }
    }
}

impl std::error::Error for ValidationError {}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn sanitize_string(input: &str) -> String {
    let mut result = input.to_string();
    result = strip_html_tags(&result);
    result = remove_control_chars(&result);
    result = normalize_whitespace(&result);
    result
}

pub fn strip_html_tags(input: &str) -> String {
    HTML_TAG_REGEX.replace_all(input, "").to_string()
}

pub fn contains_html(input: &str) -> bool {
    HTML_TAG_REGEX.is_match(input)
}

pub fn contains_script_tags(input: &str) -> bool {
    SCRIPT_TAG_REGEX.is_match(input)
}

pub fn remove_control_chars(input: &str) -> String {
    CONTROL_CHARS_REGEX.replace_all(input, "").to_string()
}

pub fn contains_control_chars(input: &str) -> bool {
    CONTROL_CHARS_REGEX.is_match(input)
}

pub fn normalize_whitespace(input: &str) -> String {
    WHITESPACE_REGEX.replace_all(input.trim(), " ").to_string()
}

pub fn validate_required<T: AsRef<str>>(value: &Option<T>) -> ValidationResult<()> {
    match value {
        Some(v) if !v.as_ref().trim().is_empty() => Ok(()),
        _ => Err(ValidationError::Required),
    }
}

pub fn validate_length(value: &str, min: usize, max: usize) -> ValidationResult<()> {
    let len = value.chars().count();
    if len < min {
        Err(ValidationError::TooShort { min })
    } else if len > max {
        Err(ValidationError::TooLong { max })
    } else {
        Ok(())
    }
}

pub fn validate_email(email: &str) -> ValidationResult<()> {
    let email = email.trim();
    if email.is_empty() {
        return Err(ValidationError::Required);
    }
    if email.len() > 254 {
        return Err(ValidationError::TooLong { max: 254 });
    }
    if !validkit::is_valid_email(email) {
        return Err(ValidationError::InvalidEmail);
    }
    Ok(())
}

pub fn validate_uuid(uuid: &str) -> ValidationResult<()> {
    if uuid.is_empty() {
        return Err(ValidationError::Required);
    }
    if !UUID_REGEX.is_match(&uuid.to_lowercase()) {
        return Err(ValidationError::InvalidUuid);
    }
    Ok(())
}

pub fn validate_url(url: &str) -> ValidationResult<()> {
    if url.is_empty() {
        return Err(ValidationError::Required);
    }
    match url::Url::parse(url) {
        Ok(parsed) => match parsed.scheme() {
            "http" | "https" => Ok(()),
            _ => Err(ValidationError::InvalidUrl),
        },
        Err(_) => Err(ValidationError::InvalidUrl),
    }
}

pub fn validate_no_html(value: &str) -> ValidationResult<()> {
    if contains_html(value) {
        Err(ValidationError::ContainsHtml)
    } else {
        Ok(())
    }
}

pub fn validate_no_scripts(value: &str) -> ValidationResult<()> {
    if contains_script_tags(value) {
        Err(ValidationError::ContainsScript)
    } else {
        Ok(())
    }
}

pub fn validate_no_javascript_protocol(value: &str) -> ValidationResult<()> {
    if JAVASCRIPT_PROTOCOL_REGEX.is_match(value) {
        Err(ValidationError::ForbiddenContent {
            reason: "JavaScript protocol is not allowed".to_string(),
        })
    } else {
        Ok(())
    }
}

pub fn validate_no_data_uri(value: &str) -> ValidationResult<()> {
    if DATA_URI_REGEX.is_match(value) {
        Err(ValidationError::ForbiddenContent {
            reason: "Data URI is not allowed".to_string(),
        })
    } else {
        Ok(())
    }
}

pub fn validate_text_input(
    value: &str,
    min: usize,
    max: usize,
    allow_html: bool,
) -> ValidationResult<String> {
    if value.is_empty() && min > 0 {
        return Err(ValidationError::Required);
    }

    validate_length(value, min, max)?;

    if !allow_html {
        validate_no_html(value)?;
        validate_no_scripts(value)?;
    }

    validate_no_javascript_protocol(value)?;
    validate_no_data_uri(value)?;

    if contains_control_chars(value) {
        return Err(ValidationError::ContainsControlChars);
    }

    Ok(normalize_whitespace(value))
}

pub fn validate_ascii_alphanumeric(value: &str) -> ValidationResult<()> {
    if value.is_empty() {
        return Err(ValidationError::Required);
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ValidationError::InvalidFormat {
            message: "Only alphanumeric characters, hyphens, and underscores are allowed"
                .to_string(),
        });
    }
    Ok(())
}

pub fn validate_slug(value: &str) -> ValidationResult<()> {
    if value.is_empty() {
        return Err(ValidationError::Required);
    }

    if !SLUG_REGEX.is_match(value) {
        return Err(ValidationError::InvalidFormat {
            message: "Slug must contain only lowercase letters, numbers, and hyphens".to_string(),
        });
    }

    validate_length(value, 1, 200)?;
    Ok(())
}

pub fn escape_html(value: &str) -> String {
    html_escape::encode_text(value).to_string()
}

pub fn escape_json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[derive(Debug, Clone)]
pub struct ValidatedString {
    value: String,
}

impl ValidatedString {
    pub fn new(value: String, min: usize, max: usize, allow_html: bool) -> ValidationResult<Self> {
        let validated = validate_text_input(&value, min, max, allow_html)?;
        Ok(Self { value: validated })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

impl AsRef<str> for ValidatedString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for ValidatedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
        assert_eq!(
            strip_html_tags("<script>alert('xss')</script>"),
            "alert('xss')"
        );
        assert_eq!(strip_html_tags("No HTML here"), "No HTML here");
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@example.org").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", 1, 10).is_ok());
        assert!(validate_length("", 1, 10).is_err());
        assert!(validate_length("very long string", 1, 5).is_err());
    }

    #[test]
    fn test_validate_no_html() {
        assert!(validate_no_html("plain text").is_ok());
        assert!(validate_no_html("<p>html</p>").is_err());
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("my-document").is_ok());
        assert!(validate_slug("my-document-123").is_ok());
        assert!(validate_slug("My-Document").is_err());
        assert!(validate_slug("my document").is_err());
        assert!(validate_slug("-start").is_err());
        assert!(validate_slug("end-").is_err());
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(
            normalize_whitespace("\t\nhello\n\tworld\t\n"),
            "hello world"
        );
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }
}
