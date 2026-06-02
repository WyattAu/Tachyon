//! Single Sign-On integration types and runtime flows.
//! Supports SAML 2.0, OpenID Connect, and LDAP providers.

pub mod ldap;
pub mod ldap_runtime;
pub mod oidc;
pub mod oidc_runtime;
pub mod saml;
pub mod saml_runtime;

pub use ldap::LdapConfig;
pub use ldap_runtime::LdapState;
pub use ldap_runtime::create_ldap_router;
pub use oidc::OidcConfig;
pub use oidc_runtime::OidcState;
pub use oidc_runtime::create_oidc_router;
pub use saml::SamlConfig;
pub use saml_runtime::SamlState;
pub use saml_runtime::create_saml_router;
