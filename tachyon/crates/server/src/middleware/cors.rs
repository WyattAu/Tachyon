// CORS middleware
// Cross-Origin Resource Sharing configuration

use crate::config::ServerConfig;
use tower_http::cors::{Any, CorsLayer};

/// Create CORS layer from configuration
///
/// # Arguments
/// * `config` - Server configuration
///
/// # Returns
/// Configured CORS layer
pub fn create_cors_layer(config: &ServerConfig) -> CorsLayer {
    if !config.cors.enabled {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let mut cors = CorsLayer::new().allow_origin(Any);

    // Add allowed methods
    for method in &config.cors.allowed_methods {
        if let Ok(method_str) = method.parse::<axum::http::Method>() {
            cors = cors.allow_methods([method_str]);
        }
    }

    // Add allowed headers
    for header in &config.cors.allowed_headers {
        if let Ok(header_str) = header.parse::<axum::http::HeaderName>() {
            cors = cors.allow_headers([header_str]);
        }
    }

    // Add exposed headers
    for header in &config.cors.exposed_headers {
        if let Ok(header_str) = header.parse::<axum::http::HeaderName>() {
            cors = cors.expose_headers([header_str]);
        }
    }

    // Set credentials
    if config.cors.allow_credentials {
        cors = cors.allow_credentials(true);
    }

    // Set max age
    if let Some(max_age) = config.cors.max_age_secs {
        cors = cors.max_age(std::time::Duration::from_secs(max_age));
    }

    cors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cors_layer() {
        let config = ServerConfig::default();
        let cors_layer = create_cors_layer(&config);
        // If no panic occurs, the test passes
        let _ = cors_layer;
    }
}
