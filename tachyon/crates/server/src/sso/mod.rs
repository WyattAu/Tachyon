//! Single Sign-On integration types and runtime flows.
//! Supports SAML 2.0, OpenID Connect, and LDAP providers.

pub mod ldap;
pub mod ldap_runtime;
pub mod oidc;
pub mod oidc_runtime;
pub mod saml;
pub mod saml_runtime;

pub use ldap_runtime::LdapState;
pub use oidc_runtime::OidcState;
pub use saml_runtime::SamlState;
