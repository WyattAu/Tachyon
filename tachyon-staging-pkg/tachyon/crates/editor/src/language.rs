//! File-type detection via extension mapping.
//!
//! Maps file extensions (e.g. "rs", "py", "js") to language identifiers
//! used by the editor's highlight provider selection.

/// Supported languages for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Json,
    Yaml,
    Css,
    Bash,
    Html,
    Markdown,
    PlainText,
    Unknown,
}

static EXT_MAP: &[(&str, Language)] = &[
    ("rs", Language::Rust),
    ("py", Language::Python),
    ("pyw", Language::Python),
    ("js", Language::JavaScript),
    ("jsx", Language::JavaScript),
    ("mjs", Language::JavaScript),
    ("cjs", Language::JavaScript),
    ("ts", Language::TypeScript),
    ("tsx", Language::TypeScript),
    ("mts", Language::TypeScript),
    ("cts", Language::TypeScript),
    ("json", Language::Json),
    ("yml", Language::Yaml),
    ("yaml", Language::Yaml),
    ("css", Language::Css),
    ("scss", Language::Css),
    ("sass", Language::Css),
    ("less", Language::Css),
    ("sh", Language::Bash),
    ("bash", Language::Bash),
    ("zsh", Language::Bash),
    ("html", Language::Html),
    ("htm", Language::Html),
    ("md", Language::Markdown),
    ("markdown", Language::Markdown),
    ("mdx", Language::Markdown),
];

impl Language {
    /// Detect language from file extension (case-insensitive).
    pub fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.to_ascii_lowercase();

        // Use a simple linear scan — the table is small (~27 entries) and
        // the lookup runs at most once per file open, so no need for a
        // HashMap build.
        for &(e, lang) in EXT_MAP {
            if e == ext_lower {
                return lang;
            }
        }

        Language::PlainText
    }

    /// Canonical name string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Css => "css",
            Language::Bash => "bash",
            Language::Html => "html",
            Language::Markdown => "markdown",
            Language::PlainText => "plain_text",
            Language::Unknown => "unknown",
        }
    }

    /// Common file extensions for this language.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::Python => &["py", "pyw"],
            Language::JavaScript => &["js", "jsx", "mjs", "cjs"],
            Language::TypeScript => &["ts", "tsx", "mts", "cts"],
            Language::Json => &["json"],
            Language::Yaml => &["yml", "yaml"],
            Language::Css => &["css", "scss", "sass", "less"],
            Language::Bash => &["sh", "bash", "zsh"],
            Language::Html => &["html", "htm"],
            Language::Markdown => &["md", "markdown", "mdx"],
            Language::PlainText => &[],
            Language::Unknown => &[],
        }
    }
}

/// Extract file extension from a filename or path.
/// Returns the extension without the dot (e.g. "rs", "py").
pub fn extract_extension(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() => ext,
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_extension_matches() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("pyw"), Language::Python);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("jsx"), Language::JavaScript);
        assert_eq!(Language::from_extension("mjs"), Language::JavaScript);
        assert_eq!(Language::from_extension("cjs"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("tsx"), Language::TypeScript);
        assert_eq!(Language::from_extension("mts"), Language::TypeScript);
        assert_eq!(Language::from_extension("cts"), Language::TypeScript);
        assert_eq!(Language::from_extension("json"), Language::Json);
        assert_eq!(Language::from_extension("yml"), Language::Yaml);
        assert_eq!(Language::from_extension("yaml"), Language::Yaml);
        assert_eq!(Language::from_extension("css"), Language::Css);
        assert_eq!(Language::from_extension("scss"), Language::Css);
        assert_eq!(Language::from_extension("sass"), Language::Css);
        assert_eq!(Language::from_extension("less"), Language::Css);
        assert_eq!(Language::from_extension("sh"), Language::Bash);
        assert_eq!(Language::from_extension("bash"), Language::Bash);
        assert_eq!(Language::from_extension("zsh"), Language::Bash);
        assert_eq!(Language::from_extension("html"), Language::Html);
        assert_eq!(Language::from_extension("htm"), Language::Html);
        assert_eq!(Language::from_extension("md"), Language::Markdown);
        assert_eq!(Language::from_extension("markdown"), Language::Markdown);
        assert_eq!(Language::from_extension("mdx"), Language::Markdown);
    }

    #[test]
    fn from_extension_case_insensitive() {
        assert_eq!(Language::from_extension("RS"), Language::Rust);
        assert_eq!(Language::from_extension("Py"), Language::Python);
        assert_eq!(Language::from_extension("JSX"), Language::JavaScript);
    }

    #[test]
    fn from_extension_unknown_is_plain_text() {
        assert_eq!(Language::from_extension("xyz"), Language::PlainText);
        assert_eq!(Language::from_extension(""), Language::PlainText);
    }

    #[test]
    fn as_str_roundtrip() {
        for &(ext, lang) in EXT_MAP {
            assert_eq!(Language::from_extension(ext), lang);
            assert!(!lang.as_str().is_empty());
        }
    }

    #[test]
    fn extract_extension_paths() {
        assert_eq!(extract_extension("main.rs"), "rs");
        assert_eq!(extract_extension("src/main.rs"), "rs");
        assert_eq!(extract_extension("/home/user/project/README.md"), "md");
        assert_eq!(extract_extension("Makefile"), "");
        assert_eq!(extract_extension(".hidden"), "hidden");
        assert_eq!(extract_extension("archive.tar.gz"), "gz");
    }

    #[test]
    fn full_detection_pipeline() {
        assert_eq!(
            Language::from_extension(extract_extension("src/lib.rs")),
            Language::Rust,
        );
        assert_eq!(
            Language::from_extension(extract_extension("/tmp/script.sh")),
            Language::Bash,
        );
        assert_eq!(
            Language::from_extension(extract_extension("Makefile")),
            Language::PlainText,
        );
    }
}
