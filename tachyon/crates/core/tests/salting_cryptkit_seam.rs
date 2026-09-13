//! Integration tests for the `tachyon-core` estate-kit seams:
//!
//! - `types::user::User::{hash_password, verify_password}` → `salting`
//!   (Argon2id, `Argon2Params::low_memory()`)
//! - `util::compute_content_hash` → `cryptkit::hash::sha256`
//!
//! These lock the contracts the rest of Tachyon depends on: salted hashes
//! that never round-trip to plaintext, rejection of wrong passwords, and
//! platform-independent content addressing.

use tachyon_core::{User, compute_content_hash};

const PASSWORD: &str = "correct horse battery staple";

#[test]
fn password_hash_and_verify_round_trip() {
    let hash = User::hash_password(PASSWORD).expect("hashing must succeed");
    assert_ne!(hash, PASSWORD, "hash must never equal the plaintext");
    let ok = User::verify_password(PASSWORD, &hash).expect("verification must succeed");
    assert!(ok);
}

#[test]
fn password_verify_rejects_wrong_password() {
    let hash = User::hash_password(PASSWORD).expect("hashing must succeed");
    let ok = User::verify_password("wrong password", &hash).expect("verification must succeed");
    assert!(!ok, "wrong password must not verify");
}

#[test]
fn password_hashes_are_salted_per_call() {
    let a = User::hash_password(PASSWORD).expect("hash a");
    let b = User::hash_password(PASSWORD).expect("hash b");
    assert_ne!(a, b, "per-call salt must produce distinct hashes");
    assert!(User::verify_password(PASSWORD, &a).expect("verify a"));
    assert!(User::verify_password(PASSWORD, &b).expect("verify b"));
}

#[test]
fn password_hash_is_argon2_format() {
    let hash = User::hash_password(PASSWORD).expect("hash");
    assert!(
        hash.starts_with("$argon2"),
        "salting seam must produce PHC-format Argon2 hashes, got: {hash}"
    );
}

#[test]
fn content_hash_is_stable_across_line_endings_and_trim() {
    let content = "# Title\n\nSome body text.\n";
    let lf = compute_content_hash(content);
    let crlf = compute_content_hash("# Title\r\n\r\nSome body text.\r\n");
    let trailing_ws = compute_content_hash("  # Title\n\nSome body text.\n  \n");

    assert_eq!(lf, crlf, "CRLF must normalize to LF before hashing");
    assert_eq!(
        lf, trailing_ws,
        "outer whitespace must be trimmed before hashing"
    );
    assert_eq!(lf.len(), 64, "sha256 hex digest length");
    assert!(lf.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn content_hash_changes_when_content_changes() {
    let a = compute_content_hash("# Version one");
    let b = compute_content_hash("# Version two");
    assert_ne!(a, b, "different content must hash differently");
}

#[test]
fn content_hash_is_deterministic() {
    let content = "The same content, hashed twice.";
    assert_eq!(compute_content_hash(content), compute_content_hash(content));
}
