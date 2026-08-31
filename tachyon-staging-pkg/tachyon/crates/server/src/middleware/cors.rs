// CORS middleware
// Cross-Origin Resource Sharing configuration
//
// The canonical implementation lives in crate::build_cors_layer() in lib.rs.
// This module is retained for backwards-compatible re-export.

use crate::config::ServerConfig;

/// Create a CORS layer from server configuration.
///
/// Delegates to `crate::build_cors_layer`. This wrapper exists for
/// backwards compatibility with `pub use middleware::cors::*`.
pub fn create_cors_layer(config: &ServerConfig) -> tower_http::cors::CorsLayer {
    crate::build_cors_layer(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cors_layer() {
        let config = ServerConfig::default();
        let cors_layer = create_cors_layer(&config);
        let _ = cors_layer;
    }
}
