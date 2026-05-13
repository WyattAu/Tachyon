// User API routes module
// Handles user CRUD operations and authentication

mod handlers;
#[cfg(test)]
mod tests;
mod types;

pub use handlers::*;
pub use types::*;
