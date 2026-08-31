// Document validation rules
// Validates document title, content, and metadata

use super::ValidationResult;
use super::common::*;
use regex::Regex;
use std::sync::LazyLock;

static SLUG_HYPHEN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-+").unwrap());

pub const MAX_TITLE_LENGTH: usize = 200;
pub const MIN_TITLE_LENGTH: usize = 1;
pub const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 2000;

#[derive(Debug, Clone)]
pub struct ValidatedDocumentTitle {
    value: String,
}

impl ValidatedDocumentTitle {
    pub fn new(title: &str) -> ValidationResult<Self> {
        let title = title.trim();

        if title.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(title, MIN_TITLE_LENGTH, MAX_TITLE_LENGTH)?;
        validate_no_html(title)?;
        validate_no_scripts(title)?;

        let sanitized = sanitize_string(title);

        Ok(Self { value: sanitized })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

impl AsRef<str> for ValidatedDocumentTitle {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedDocumentContent {
    content: String,
    size: usize,
}

impl ValidatedDocumentContent {
    pub fn new(content: &str) -> ValidationResult<Self> {
        let size = content.len();

        if size > MAX_CONTENT_SIZE {
            return Err(ValidationError::TooLong {
                max: MAX_CONTENT_SIZE,
            });
        }

        if contains_control_chars(content) {
            return Err(ValidationError::ContainsControlChars);
        }

        Ok(Self {
            content: content.to_string(),
            size,
        })
    }

    pub fn from_bytes(content: Vec<u8>) -> ValidationResult<Self> {
        let size = content.len();

        if size > MAX_CONTENT_SIZE {
            return Err(ValidationError::TooLong {
                max: MAX_CONTENT_SIZE,
            });
        }

        let content = String::from_utf8(content).map_err(|_| ValidationError::InvalidFormat {
            message: "Invalid UTF-8 content".to_string(),
        })?;

        Ok(Self { content, size })
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn into_inner(self) -> String {
        self.content
    }
}

impl AsRef<str> for ValidatedDocumentContent {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedTag {
    value: String,
}

impl ValidatedTag {
    pub fn new(tag: &str) -> ValidationResult<Self> {
        let tag = tag.trim();

        if tag.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(tag, 1, MAX_TAG_LENGTH)?;
        validate_no_html(tag)?;

        let tag = normalize_whitespace(tag);

        if tag.contains(',') {
            return Err(ValidationError::InvalidFormat {
                message: "Tags cannot contain commas".to_string(),
            });
        }

        Ok(Self { value: tag })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

impl AsRef<str> for ValidatedTag {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedTagList {
    tags: Vec<ValidatedTag>,
}

impl ValidatedTagList {
    pub fn new(tags: &[String]) -> ValidationResult<Self> {
        if tags.len() > MAX_TAGS {
            return Err(ValidationError::TooLong { max: MAX_TAGS });
        }

        let validated: Result<Vec<_>, _> = tags.iter().map(|t| ValidatedTag::new(t)).collect();

        Ok(Self { tags: validated? })
    }

    pub fn as_strings(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.as_str().to_string()).collect()
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidatedTag> {
        self.tags.iter()
    }
}

#[derive(Debug, Clone)]
pub enum DocumentVisibilityValue {
    Public,
    Private,
    Restricted,
}

impl DocumentVisibilityValue {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> ValidationResult<Self> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            "restricted" => Ok(Self::Restricted),
            _ => Err(ValidationError::InvalidFormat {
                message: "Visibility must be 'public', 'private', or 'restricted'".to_string(),
            }),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Restricted => "restricted",
        }
    }
}

#[derive(Debug, Clone)]
pub enum DocumentStatusValue {
    Draft,
    Published,
    Archived,
    Deleted,
}

impl DocumentStatusValue {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> ValidationResult<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "archived" => Ok(Self::Archived),
            "deleted" => Ok(Self::Deleted),
            _ => Err(ValidationError::InvalidFormat {
                message: "Status must be 'draft', 'published', 'archived', or 'deleted'"
                    .to_string(),
            }),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedDescription {
    value: String,
}

impl ValidatedDescription {
    pub fn new(description: &str) -> ValidationResult<Self> {
        let description = description.trim();

        if description.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(description, 1, MAX_DESCRIPTION_LENGTH)?;
        validate_no_html(description)?;

        let sanitized = sanitize_string(description);

        Ok(Self { value: sanitized })
    }

    pub fn new_optional(description: Option<&str>) -> ValidationResult<Option<Self>> {
        match description {
            Some(d) if !d.trim().is_empty() => Ok(Some(Self::new(d)?)),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedDocumentId {
    value: String,
}

impl ValidatedDocumentId {
    pub fn new(id: &str) -> ValidationResult<Self> {
        let id = id.trim();

        if id.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_uuid(id)?;

        Ok(Self {
            value: id.to_lowercase(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedSlug {
    value: String,
}

impl ValidatedSlug {
    pub fn new(slug: &str) -> ValidationResult<Self> {
        validate_slug(slug)?;
        Ok(Self {
            value: slug.to_lowercase(),
        })
    }

    pub fn generate_from_title(title: &str) -> Self {
        let slug = title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|&c| c != '\0')
            .collect::<String>();

        let slug = SLUG_HYPHEN_RE
            .replace_all(&slug, "-")
            .trim_matches('-')
            .to_string();

        let slug = if slug.is_empty() {
            format!("doc-{}", uuid::Uuid::new_v4())
        } else {
            slug
        };

        Self { value: slug }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_document_title() {
        assert!(ValidatedDocumentTitle::new("My Document").is_ok());
        assert!(ValidatedDocumentTitle::new("").is_err());
        assert!(ValidatedDocumentTitle::new("<script>alert('xss')</script>").is_err());
        assert!(ValidatedDocumentTitle::new(&"x".repeat(201)).is_err());
    }

    #[test]
    fn test_validated_document_content() {
        assert!(ValidatedDocumentContent::new("Some content").is_ok());
        assert!(ValidatedDocumentContent::new("").is_ok());
    }

    #[test]
    fn test_validated_tag() {
        assert!(ValidatedTag::new("rust").is_ok());
        assert!(ValidatedTag::new("web development").is_ok());
        assert!(ValidatedTag::new("").is_err());
        assert!(ValidatedTag::new("<script>").is_err());
    }

    #[test]
    fn test_validated_tag_list() {
        let tags = vec!["rust".to_string(), "web".to_string()];
        assert!(ValidatedTagList::new(&tags).is_ok());

        let too_many: Vec<String> = (0..25).map(|i| format!("tag{}", i)).collect();
        assert!(ValidatedTagList::new(&too_many).is_err());
    }

    #[test]
    fn test_validated_slug() {
        let slug = ValidatedSlug::generate_from_title("My Document Title!");
        assert_eq!(slug.as_str(), "my-document-title");

        let slug = ValidatedSlug::generate_from_title("Hello    World");
        assert_eq!(slug.as_str(), "hello-world");
    }
}
