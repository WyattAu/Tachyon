//! GraphQL API layer for Tachyon.
//!
//! Provides a GraphQL schema alongside the existing REST API.
//! All GraphQL queries/mutations go through the same AppState and middleware.

pub mod schema;
pub mod types;

pub use schema::build_schema;
pub use types::MutationRoot;
pub use types::QueryRoot;
