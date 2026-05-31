//! GraphQL API layer for Tachyon.
//!
//! Provides a GraphQL schema alongside the existing REST API.
//! All GraphQL queries/mutations go through the same AppState and middleware.
//!
//! Note: `build_schema()` is only available in tests because it creates a schema
//! without a `DatabasePool`. Production code must use `build_schema_with_data(pool)`
//! to ensure the schema has access to the database.

pub mod schema;
pub mod types;

#[cfg(test)]
pub use schema::build_schema;
pub use schema::build_schema_with_data;
pub use schema::GraphqlAuthContext;
pub use schema::TachyonSchema;
pub use types::MutationRoot;
pub use types::QueryRoot;
