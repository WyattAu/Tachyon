//! Integration tests pinning the validation security contracts the server
//! delegates to the `validkit` estate crate:
//!
//! - `validation::common::validate_email` → `validkit::is_valid_email`
//! - `routes::files::handlers` object-key safety → `validkit::ObjectKey`
//!   (path-traversal rejection is a threat-model requirement)
//!
//! The `ObjectKey` cases pin the behavior the file handlers rely on at the
//! seam; if `validkit` changes its rules, these tests catch it here first.

use tachyon_server::validation::common::{ValidationError, validate_email};
use validkit::ObjectKey;

#[test]
fn accepts_ordinary_addresses() {
    for ok in [
        "user@example.com",
        "first.last@sub.domain.org",
        "a+b@host.io",
    ] {
        assert!(validate_email(ok).is_ok(), "{ok} should be valid");
    }
}

#[test]
fn empty_email_is_required_error() {
    assert!(matches!(validate_email(""), Err(ValidationError::Required)));
    assert!(matches!(
        validate_email("   "),
        Err(ValidationError::Required)
    ));
}

#[test]
fn overlong_email_is_too_long_error() {
    let local = "a".repeat(250);
    let email = format!("{local}@example.com");
    assert!(email.len() > 254);
    assert!(matches!(
        validate_email(&email),
        Err(ValidationError::TooLong { .. })
    ));
}

#[test]
fn malformed_email_is_invalid_error() {
    for bad in ["no-at-sign", "two@@ats.com", "@leading.com", "trailing@"] {
        assert!(validate_email(bad).is_err(), "{bad} should be rejected");
    }
}

#[test]
fn object_key_rejects_path_traversal() {
    for evil in ["../secret", "docs/../../../etc/passwd", "a/../..", ".."] {
        assert!(ObjectKey::parse(evil).is_err(), "{evil} must be rejected");
    }
}

#[test]
fn object_key_rejects_absolute_and_empty_and_control_chars() {
    for evil in ["/abs", "", "line1\nline2", "cr\r"] {
        assert!(ObjectKey::parse(evil).is_err(), "{evil:?} must be rejected");
    }
}

#[test]
fn object_key_accepts_normal_doc_paths() {
    for fine in ["docs/readme.md", "nested/dir/file.tar.gz", "a-b_c.9"] {
        let key = ObjectKey::parse(fine).unwrap_or_else(|e| panic!("{fine} must parse: {e:?}"));
        assert_eq!(key.as_str(), fine);
    }
}

#[test]
fn object_key_rejects_over_1024_bytes() {
    let long = "a".repeat(1025);
    assert!(ObjectKey::parse(&long).is_err());
    let max = "a".repeat(1024);
    assert!(ObjectKey::parse(&max).is_ok());
}
