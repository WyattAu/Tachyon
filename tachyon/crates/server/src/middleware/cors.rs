// CORS middleware
// Cross-Origin Resource Sharing configuration

use crate::config::ServerConfig;
use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn create_cors_layer(config: &ServerConfig) -> CorsLayer {
    if !config.cors.enabled {
        return CorsLayer::new();
    }

    let allow_origin = if config.cors.allowed_origins.contains(&"*".to_string()) {
        AllowOrigin::any()
    } else {
        let origins: Vec<HeaderValue> = config
            .cors
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        AllowOrigin::list(origins)
    };

    let allow_methods: Vec<Method> = config
        .cors
        .allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();

    let allow_headers: Vec<axum::http::HeaderName> = config
        .cors
        .allowed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();

    let expose_headers: Vec<axum::http::HeaderName> = config
        .cors
        .exposed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();

    let mut cors = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(allow_methods)
        .allow_headers(allow_headers);

    if !expose_headers.is_empty() {
        cors = cors.expose_headers(expose_headers);
    }

    if config.cors.allow_credentials {
        cors = cors.allow_credentials(true);
    }

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
