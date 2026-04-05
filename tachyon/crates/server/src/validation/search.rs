// Search query validation
// Validates search queries, filters, and pagination parameters

use super::common::*;
use super::ValidationResult;

pub const MAX_QUERY_LENGTH: usize = 500;
pub const MAX_PAGE_SIZE: usize = 100;
pub const DEFAULT_PAGE_SIZE: usize = 20;
pub const MIN_PAGE: usize = 1;

#[derive(Debug, Clone)]
pub struct ValidatedSearchQuery {
    query: String,
}

impl ValidatedSearchQuery {
    pub fn new(query: &str) -> ValidationResult<Self> {
        let query = query.trim();

        if query.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(query, 1, MAX_QUERY_LENGTH)?;

        validate_no_html(query)?;
        validate_no_scripts(query)?;
        validate_no_javascript_protocol(query)?;

        if contains_control_chars(query) {
            return Err(ValidationError::ContainsControlChars);
        }

        let sanitized = sanitize_string(query);

        let dangerous_patterns = [
            r"--",
            r";--",
            r"/*",
            r"*/",
            r"@@",
            r"char(",
            r"nchar(",
            r"varchar(",
            r"nvarchar(",
            r"alter(",
            r"begin(",
            r"cast(",
            r"create(",
            r"cursor(",
            r"declare(",
            r"delete(",
            r"drop(",
            r"exec(",
            r"execute(",
            r"fetch(",
            r"insert(",
            r"kill(",
            r"open(",
            r"select(",
            r"sys(",
            r"sysobjects(",
            r"syscolumns(",
            r"table(",
            r"update(",
            r"union(",
            r"or 1=1",
            r"and 1=1",
            r#"or '"#,
            r#"and '"#,
            r#"or ""#,
            r#"and ""#,
            r#"'='"#,
            r#"' or"#,
            r#"' and"#,
        ];

        let lower_query = sanitized.to_lowercase();
        for pattern in dangerous_patterns {
            if lower_query.contains(pattern) {
                return Err(ValidationError::ForbiddenContent {
                    reason: "Query contains potentially dangerous patterns".to_string(),
                });
            }
        }

        Ok(Self { query: sanitized })
    }

    pub fn new_optional(query: Option<&str>) -> ValidationResult<Option<Self>> {
        match query {
            Some(q) if !q.trim().is_empty() => Ok(Some(Self::new(q)?)),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.query
    }

    pub fn into_inner(self) -> String {
        self.query
    }
}

impl AsRef<str> for ValidatedSearchQuery {
    fn as_ref(&self) -> &str {
        &self.query
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedPage {
    value: usize,
}

impl ValidatedPage {
    pub fn new(page: Option<usize>) -> Self {
        Self {
            value: page.unwrap_or(1).max(MIN_PAGE),
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn offset(&self, page_size: usize) -> usize {
        (self.value.saturating_sub(1)) * page_size
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedPageSize {
    value: usize,
}

impl ValidatedPageSize {
    pub fn new(page_size: Option<usize>) -> Self {
        Self {
            value: page_size
                .unwrap_or(DEFAULT_PAGE_SIZE)
                .min(MAX_PAGE_SIZE)
                .max(1),
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedPagination {
    page: ValidatedPage,
    page_size: ValidatedPageSize,
}

impl ValidatedPagination {
    pub fn new(page: Option<usize>, page_size: Option<usize>) -> Self {
        Self {
            page: ValidatedPage::new(page),
            page_size: ValidatedPageSize::new(page_size),
        }
    }

    pub fn page(&self) -> usize {
        self.page.value()
    }

    pub fn page_size(&self) -> usize {
        self.page_size.value()
    }

    pub fn offset(&self) -> usize {
        self.page.offset(self.page_size.value())
    }

    pub fn limit(&self) -> i64 {
        self.page_size.value() as i64
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedTagFilter {
    tags: Vec<String>,
}

impl ValidatedTagFilter {
    pub fn new(tags: Option<&str>) -> ValidationResult<Self> {
        let tags = match tags {
            Some(t) if !t.trim().is_empty() => t,
            _ => return Ok(Self { tags: vec![] }),
        };

        let parsed: Vec<String> = tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if parsed.len() > 20 {
            return Err(ValidationError::TooLong { max: 20 });
        }

        for tag in &parsed {
            validate_length(tag, 1, 100)?;
            validate_no_html(tag)?;
        }

        Ok(Self { tags: parsed })
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedDateFilter {
    date: chrono::DateTime<chrono::Utc>,
}

impl ValidatedDateFilter {
    pub fn new(date: Option<&str>) -> ValidationResult<Option<Self>> {
        match date {
            Some(d) if !d.trim().is_empty() => {
                let parsed = chrono::DateTime::parse_from_rfc3339(d)
                    .map_err(|_| ValidationError::InvalidFormat {
                        message: "Invalid date format. Use ISO 8601 (RFC 3339)".to_string(),
                    })?
                    .with_timezone(&chrono::Utc);

                Ok(Some(Self { date: parsed }))
            }
            _ => Ok(None),
        }
    }

    pub fn date(&self) -> chrono::DateTime<chrono::Utc> {
        self.date
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedContentType {
    content_type: String,
}

impl ValidatedContentType {
    pub fn new(content_type: Option<&str>) -> ValidationResult<Option<Self>> {
        match content_type {
            Some(ct) if !ct.trim().is_empty() => {
                let ct = ct.trim().to_lowercase();

                let valid_types = ["markdown", "text", "html"];
                if !valid_types.contains(&ct.as_str()) {
                    return Err(ValidationError::InvalidFormat {
                        message: format!("Content type must be one of: {}", valid_types.join(", ")),
                    });
                }

                Ok(Some(Self { content_type: ct }))
            }
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.content_type
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedSortField {
    field: String,
    descending: bool,
}

impl ValidatedSortField {
    pub fn new(field: Option<&str>, descending: bool) -> ValidationResult<Option<Self>> {
        match field {
            Some(f) if !f.trim().is_empty() => {
                let f = f.trim().to_lowercase();

                let valid_fields = [
                    "title",
                    "created_at",
                    "updated_at",
                    "published_at",
                    "word_count",
                    "status",
                    "visibility",
                    "author_id",
                ];

                if !valid_fields.contains(&f.as_str()) {
                    return Err(ValidationError::InvalidFormat {
                        message: format!("Sort field must be one of: {}", valid_fields.join(", ")),
                    });
                }

                validate_ascii_alphanumeric(&f)?;

                Ok(Some(Self {
                    field: f,
                    descending,
                }))
            }
            _ => Ok(None),
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn is_descending(&self) -> bool {
        self.descending
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedSearchFilters {
    pub query: Option<ValidatedSearchQuery>,
    pub pagination: ValidatedPagination,
    pub content_type: Option<ValidatedContentType>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub project_id: Option<String>,
    pub author_id: Option<String>,
    pub tags: ValidatedTagFilter,
    pub date_from: Option<ValidatedDateFilter>,
    pub date_to: Option<ValidatedDateFilter>,
    pub sort: Option<ValidatedSortField>,
}

impl ValidatedSearchFilters {
    pub fn validate(
        query: Option<&str>,
        page: Option<usize>,
        page_size: Option<usize>,
        content_type: Option<&str>,
        status: Option<&str>,
        visibility: Option<&str>,
        project_id: Option<&str>,
        author_id: Option<&str>,
        tags: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        sort_field: Option<&str>,
        sort_desc: bool,
    ) -> ValidationResult<Self> {
        let validated_query = ValidatedSearchQuery::new_optional(query)?;

        let validated_status = if let Some(s) = status {
            let valid_statuses = ["draft", "published", "archived", "deleted"];
            let s = s.trim().to_lowercase();
            if !s.is_empty() && !valid_statuses.contains(&s.as_str()) {
                return Err(ValidationError::InvalidFormat {
                    message: format!("Status must be one of: {}", valid_statuses.join(", ")),
                });
            }
            Some(s)
        } else {
            None
        };

        let validated_visibility = if let Some(v) = visibility {
            let valid_visibilities = ["public", "private", "restricted"];
            let v = v.trim().to_lowercase();
            if !v.is_empty() && !valid_visibilities.contains(&v.as_str()) {
                return Err(ValidationError::InvalidFormat {
                    message: format!(
                        "Visibility must be one of: {}",
                        valid_visibilities.join(", ")
                    ),
                });
            }
            Some(v)
        } else {
            None
        };

        Ok(Self {
            query: validated_query,
            pagination: ValidatedPagination::new(page, page_size),
            content_type: ValidatedContentType::new(content_type)?,
            status: validated_status,
            visibility: validated_visibility,
            project_id: project_id
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            author_id: author_id
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            tags: ValidatedTagFilter::new(tags)?,
            date_from: ValidatedDateFilter::new(date_from)?,
            date_to: ValidatedDateFilter::new(date_to)?,
            sort: ValidatedSortField::new(sort_field, sort_desc)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_search_query() {
        assert!(ValidatedSearchQuery::new("hello world").is_ok());
        assert!(ValidatedSearchQuery::new("rust programming").is_ok());
        assert!(ValidatedSearchQuery::new("").is_err());
        assert!(ValidatedSearchQuery::new("<script>alert('xss')</script>").is_err());
        assert!(ValidatedSearchQuery::new("'; DROP TABLE users;--").is_err());
    }

    #[test]
    fn test_validated_pagination() {
        let pagination = ValidatedPagination::new(Some(2), Some(50));
        assert_eq!(pagination.page(), 2);
        assert_eq!(pagination.page_size(), 50);
        assert_eq!(pagination.offset(), 50);

        let pagination = ValidatedPagination::new(None, None);
        assert_eq!(pagination.page(), 1);
        assert_eq!(pagination.page_size(), 20);

        let pagination = ValidatedPagination::new(Some(0), Some(200));
        assert_eq!(pagination.page(), 1);
        assert_eq!(pagination.page_size(), 100);
    }

    #[test]
    fn test_validated_tag_filter() {
        let filter = ValidatedTagFilter::new(Some("rust, web, api")).unwrap();
        assert_eq!(filter.tags(), &["rust", "web", "api"]);

        let filter = ValidatedTagFilter::new(None).unwrap();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_sql_injection_prevention() {
        assert!(ValidatedSearchQuery::new("'; DROP TABLE users;--").is_err());
        assert!(ValidatedSearchQuery::new("1' OR '1'='1").is_err());
        assert!(ValidatedSearchQuery::new("admin'--").is_err());
    }
}
