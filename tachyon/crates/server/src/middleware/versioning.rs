//! API version negotiation middleware.
//! Routes prefixed with /api/v2/ receive v2 behavior.
//! Routes prefixed with /api/v1/ receive v1 behavior with deprecation headers.

use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

const DEPRECATION: &str = "Deprecation";

#[derive(Debug, Clone)]
pub struct ApiVersion {
    pub major: u32,
}

impl ApiVersion {
    pub const V1: ApiVersion = ApiVersion { major: 1 };
    pub const V2: ApiVersion = ApiVersion { major: 2 };

    pub fn from_path(path: &str) -> Option<Self> {
        let path = path.trim_start_matches('/');
        let mut parts = path.split('/');
        if parts.next()? != "api" {
            return None;
        }
        let version_str = parts.next()?;
        let major = version_str.strip_prefix('v')?.parse().ok()?;
        Some(ApiVersion { major })
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.major)
    }
}

/// Middleware function that adds API version and deprecation headers to responses.
pub async fn api_version_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let path = request.uri().path();
    let version = ApiVersion::from_path(path);

    let mut response = next.run(request).await;

    if let Some(ref ver) = version {
        let headers = response.headers_mut();
        headers.insert(
            "X-API-Version",
            axum::http::HeaderValue::from_str(&ver.to_string())
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("unknown")),
        );

        if ver.major == 1 {
            headers.insert(DEPRECATION, header::HeaderValue::from_static("true"));
            headers.insert(
                header::LINK,
                header::HeaderValue::from_static("</api/v2>; rel=\"successor-version\""),
            );
            headers.insert(
                "Sunset",
                header::HeaderValue::from_static("Sat, 01 Jan 2028 00:00:00 GMT"),
            );
        }
    }

    response
}

/// Returns a response with deprecation headers (usable as a handler or in tests).
pub fn deprecation_response() -> Response {
    let mut resp = StatusCode::OK.into_response();
    let headers = resp.headers_mut();
    headers.insert(DEPRECATION, header::HeaderValue::from_static("true"));
    headers.insert(
        header::LINK,
        header::HeaderValue::from_static("</api/v2>; rel=\"successor-version\""),
    );
    headers.insert(
        "Sunset",
        header::HeaderValue::from_static("Sat, 01 Jan 2028 00:00:00 GMT"),
    );
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_from_path_v1() {
        let version = ApiVersion::from_path("/api/v1/documents").unwrap();
        assert_eq!(version.major, 1);
    }

    #[test]
    fn test_api_version_from_path_v2() {
        let version = ApiVersion::from_path("/api/v2/health").unwrap();
        assert_eq!(version.major, 2);
    }

    #[test]
    fn test_api_version_from_non_api_path() {
        assert!(ApiVersion::from_path("/health").is_none());
        assert!(ApiVersion::from_path("/graphql").is_none());
    }

    #[test]
    fn test_api_version_from_invalid_version() {
        assert!(ApiVersion::from_path("/api/vx/documents").is_none());
    }

    #[test]
    fn test_api_version_display() {
        assert_eq!(ApiVersion::V1.to_string(), "v1");
        assert_eq!(ApiVersion::V2.to_string(), "v2");
    }

    #[test]
    fn test_deprecation_response_headers() {
        let response = deprecation_response();
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers.get(DEPRECATION).unwrap(), "true");
        assert_eq!(
            headers.get(header::LINK).unwrap(),
            "</api/v2>; rel=\"successor-version\""
        );
        assert_eq!(
            headers.get("Sunset").unwrap(),
            "Sat, 01 Jan 2028 00:00:00 GMT"
        );
    }
}
