//! Error types for import/export operations.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for import/export operations.
pub type ImportExportResult<T> = Result<T, ImportExportError>;

/// Import/export error types.
#[derive(Error, Debug)]
pub enum ImportExportError {
    /// IO errors during file reading/writing.
    #[error("IO error: {path:?}: {message}")]
    Io { path: PathBuf, message: String },

    /// Error reading or writing a ZIP archive.
    #[error("ZIP error: {0}")]
    Zip(String),

    /// Error parsing YAML frontmatter.
    #[error("Frontmatter parse error in {path:?}: {message}")]
    FrontmatterParse { path: PathBuf, message: String },

    /// Error during import operations.
    #[error("Import error: {0}")]
    Import(String),

    /// Error during export operations.
    #[error("Export error: {0}")]
    Export(String),

    /// A file was not recognized as a supported format.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// A required field was missing.
    #[error("Missing field: {0}")]
    MissingField(String),

    /// Rendering error (from tachyon-renderer).
    #[error("Render error: {0}")]
    Render(String),

    /// Generic error.
    #[error("{0}")]
    Generic(String),
}

impl ImportExportError {
    pub fn io(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn zip(msg: impl Into<String>) -> Self {
        Self::Zip(msg.into())
    }

    pub fn frontmatter(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::FrontmatterParse {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn import(msg: impl Into<String>) -> Self {
        Self::Import(msg.into())
    }

    pub fn export(msg: impl Into<String>) -> Self {
        Self::Export(msg.into())
    }
}

impl From<std::io::Error> for ImportExportError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            path: PathBuf::new(),
            message: err.to_string(),
        }
    }
}

impl From<zip::result::ZipError> for ImportExportError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::Zip(err.to_string())
    }
}
