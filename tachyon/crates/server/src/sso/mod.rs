//! Single Sign-On integration types and runtime flows.
//! Supports SAML 2.0, OpenID Connect, and LDAP providers.

pub mod ldap;
pub mod oidc;
pub mod oidc_runtime;
pub mod saml;

pub use oidc_runtime::{create_oidc_router, OidcState};
