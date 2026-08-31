// Cache-Control and ETag middleware
// Adds appropriate caching headers to API responses

use axum::{
    extract::Request,
    http::{HeaderValue, Method, header},
    middleware::Next,
    response::Response,
};

/// Add Cache-Control and ETag headers to responses.
///
/// Strategy:
/// - GET requests: Cache-Control with short max-age for API (stale-while-revalidate)
/// - robots.txt / sitemap.xml: Longer cache (1 hour)
/// - Static HTML pages: Moderate cache (5 minutes)
/// - Non-GET / error responses: no-cache, no-store
pub async fn cache_control_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let status = response.status();

    // Only add caching headers to successful GET responses
    if method == Method::GET && status.is_success() {
        let cache_directive = determine_cache_directive(&path);
        if let Ok(value) = HeaderValue::from_str(&cache_directive) {
            response.headers_mut().insert(header::CACHE_CONTROL, value);
        }

        // Add a weak ETag based on the path and a content hash hint.
        // Full body-based ETags require buffering the entire response body,
        // which defeats the purpose of streaming. For API responses,
        // the combination of Cache-Control + short max-age is sufficient.
        // We add a path-based weak ETag as a hint for conditional requests.
        if !response.headers().contains_key(header::ETAG) {
            let etag_value = format!("W/\"{:016x}\"", simple_hash(&path));
            if let Ok(etag) = HeaderValue::from_str(&etag_value) {
                response.headers_mut().insert(header::ETAG, etag);
            }
        }
    } else {
        // Non-GET or error: prevent caching
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        );
    }

    response
}

/// Determine the appropriate Cache-Control directive based on path and file extension.
fn determine_cache_directive(path: &str) -> String {
    // HTML files and root: no cache (SPA shell must always be fresh)
    if path.ends_with(".html") || path == "/" {
        return "no-cache, no-store, must-revalidate".to_string();
    }

    // WASM: long cache (immutable, content-hashed by Trunk)
    if path.ends_with(".wasm") {
        return "public, max-age=31536000, immutable".to_string();
    }

    // JS/CSS: long cache (content-hashed by Trunk)
    if path.ends_with(".js") || path.ends_with(".css") {
        return "public, max-age=31536000, immutable".to_string();
    }

    // Images: long cache
    if path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".webp")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
    {
        return "public, max-age=31536000, immutable".to_string();
    }

    // Fonts: long cache
    if path.ends_with(".woff")
        || path.ends_with(".woff2")
        || path.ends_with(".ttf")
        || path.ends_with(".eot")
    {
        return "public, max-age=31536000, immutable".to_string();
    }

    // Source maps: long cache (paired with content-hashed assets)
    if path.ends_with(".map") {
        return "public, max-age=31536000, immutable".to_string();
    }

    // Manifest files: short cache (content may change)
    if path.ends_with(".webmanifest") || path == "/manifest.json" {
        return "public, max-age=3600".to_string();
    }

    // SEO files: cache for 1 hour
    if path == "/robots.txt" || path == "/sitemap.xml" {
        return "public, max-age=3600, stale-while-revalidate=86400".to_string();
    }

    // SSR document pages: cache for 5 minutes
    if path.starts_with("/docs/") {
        return "public, max-age=300, stale-while-revalidate=3600".to_string();
    }

    // API responses: short cache with SWR
    if path.starts_with("/api/") {
        // Health/metrics: very short
        if path.contains("/health") || path.contains("/metrics") {
            return "no-cache".to_string();
        }
        return "private, max-age=10, stale-while-revalidate=60".to_string();
    }

    // Default: conservative caching
    "public, max-age=60, stale-while-revalidate=300".to_string()
}

/// Simple FNV-1a hash for generating weak ETags from paths.
/// No external dependency needed.
fn simple_hash(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_cache_directive_seo() {
        let directive = determine_cache_directive("/robots.txt");
        assert!(directive.contains("max-age=3600"));

        let directive = determine_cache_directive("/sitemap.xml");
        assert!(directive.contains("max-age=3600"));
    }

    #[test]
    fn test_determine_cache_directive_docs() {
        let directive = determine_cache_directive("/docs/some-uuid");
        assert!(directive.contains("max-age=300"));
    }

    #[test]
    fn test_determine_cache_directive_api() {
        let directive = determine_cache_directive("/api/v1/documents");
        assert!(directive.contains("max-age=10"));

        let directive = determine_cache_directive("/api/v1/health");
        assert_eq!(directive, "no-cache");
    }

    #[test]
    fn test_determine_cache_directive_html() {
        let directive = determine_cache_directive("/index.html");
        assert_eq!(directive, "no-cache, no-store, must-revalidate");

        let directive = determine_cache_directive("/");
        assert_eq!(directive, "no-cache, no-store, must-revalidate");
    }

    #[test]
    fn test_determine_cache_directive_wasm() {
        let directive = determine_cache_directive("/tachyon_frontend_bg.wasm");
        assert!(directive.contains("max-age=31536000"));
        assert!(directive.contains("immutable"));
    }

    #[test]
    fn test_determine_cache_directive_js_css() {
        let directive = determine_cache_directive("/tachyon_frontend.js");
        assert!(directive.contains("immutable"));

        let directive = determine_cache_directive("/styles.css");
        assert!(directive.contains("immutable"));
    }

    #[test]
    fn test_determine_cache_directive_images() {
        for ext in &[".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".ico"] {
            let directive = determine_cache_directive(&format!("/assets/img{}", ext));
            assert!(
                directive.contains("immutable"),
                "Failed for extension {}",
                ext
            );
        }
    }

    #[test]
    fn test_determine_cache_directive_fonts() {
        for ext in &[".woff", ".woff2", ".ttf", ".eot"] {
            let directive = determine_cache_directive(&format!("/assets/font{}", ext));
            assert!(
                directive.contains("immutable"),
                "Failed for extension {}",
                ext
            );
        }
    }

    #[test]
    fn test_simple_hash_deterministic() {
        let h1 = simple_hash("/api/v1/documents");
        let h2 = simple_hash("/api/v1/documents");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_simple_hash_different_paths() {
        let h1 = simple_hash("/api/v1/documents");
        let h2 = simple_hash("/api/v1/users");
        assert_ne!(h1, h2);
    }
}
