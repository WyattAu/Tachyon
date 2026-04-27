// Security headers middleware
// Adds comprehensive security headers to all responses

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use crate::config::SecurityConfig as ServerSecurityConfig;

#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    pub content_security_policy: ContentSecurityPolicy,
    pub x_frame_options: XFrameOptions,
    pub x_content_type_options: bool,
    pub strict_transport_security: Option<StrictTransportSecurity>,
    pub x_xss_protection: bool,
    pub referrer_policy: ReferrerPolicy,
    pub permissions_policy: Option<PermissionsPolicy>,
    pub cross_origin_embedder_policy: Option<CrossOriginEmbedderPolicy>,
    pub cross_origin_opener_policy: Option<CrossOriginOpenerPolicy>,
    pub cross_origin_resource_policy: Option<CrossOriginResourcePolicy>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            content_security_policy: ContentSecurityPolicy::default(),
            x_frame_options: XFrameOptions::Deny,
            x_content_type_options: true,
            strict_transport_security: Some(StrictTransportSecurity {
                max_age: 31536000,
                include_subdomains: true,
                preload: true,
            }),
            x_xss_protection: true,
            referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            permissions_policy: Some(PermissionsPolicy::default()),
            cross_origin_embedder_policy: Some(CrossOriginEmbedderPolicy::RequireCorp),
            cross_origin_opener_policy: Some(CrossOriginOpenerPolicy::SameOrigin),
            cross_origin_resource_policy: Some(CrossOriginResourcePolicy::SameOrigin),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub media_src: Vec<String>,
    pub object_src: Vec<String>,
    pub frame_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub upgrade_insecure_requests: bool,
    pub block_all_mixed_content: bool,
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string(), "'wasm-unsafe-eval'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string(), "https://cdn.tailwindcss.com".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string(), "wss:".to_string()],
            media_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'self'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: true,
            block_all_mixed_content: true,
        }
    }
}

impl ContentSecurityPolicy {
    pub fn to_header_value(&self) -> String {
        let mut directives = Vec::new();
        
        if !self.default_src.is_empty() {
            directives.push(format!("default-src {}", self.default_src.join(" ")));
        }
        if !self.script_src.is_empty() {
            directives.push(format!("script-src {}", self.script_src.join(" ")));
        }
        if !self.style_src.is_empty() {
            directives.push(format!("style-src {}", self.style_src.join(" ")));
        }
        if !self.img_src.is_empty() {
            directives.push(format!("img-src {}", self.img_src.join(" ")));
        }
        if !self.font_src.is_empty() {
            directives.push(format!("font-src {}", self.font_src.join(" ")));
        }
        if !self.connect_src.is_empty() {
            directives.push(format!("connect-src {}", self.connect_src.join(" ")));
        }
        if !self.media_src.is_empty() {
            directives.push(format!("media-src {}", self.media_src.join(" ")));
        }
        if !self.object_src.is_empty() {
            directives.push(format!("object-src {}", self.object_src.join(" ")));
        }
        if !self.frame_src.is_empty() {
            directives.push(format!("frame-src {}", self.frame_src.join(" ")));
        }
        if !self.frame_ancestors.is_empty() {
            directives.push(format!("frame-ancestors {}", self.frame_ancestors.join(" ")));
        }
        if !self.base_uri.is_empty() {
            directives.push(format!("base-uri {}", self.base_uri.join(" ")));
        }
        if !self.form_action.is_empty() {
            directives.push(format!("form-action {}", self.form_action.join(" ")));
        }
        if self.upgrade_insecure_requests {
            directives.push("upgrade-insecure-requests".to_string());
        }
        if self.block_all_mixed_content {
            directives.push("block-all-mixed-content".to_string());
        }
        
        directives.join("; ")
    }
    
    pub fn development() -> Self {
        Self {
            default_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string(), "'unsafe-eval'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "https:".to_string(), "http:".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string(), "ws:".to_string(), "wss:".to_string()],
            media_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'self'".to_string()],
            frame_ancestors: vec!["'self'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: false,
            block_all_mixed_content: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum XFrameOptions {
    Deny,
    SameOrigin,
    AllowFrom(String),
}

impl XFrameOptions {
    pub fn to_header_value(&self) -> &'static str {
        match self {
            XFrameOptions::Deny => "DENY",
            XFrameOptions::SameOrigin => "SAMEORIGIN",
            XFrameOptions::AllowFrom(_) => "SAMEORIGIN",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

impl ReferrerPolicy {
    pub fn to_header_value(&self) -> &'static str {
        match self {
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrictTransportSecurity {
    pub max_age: u64,
    pub include_subdomains: bool,
    pub preload: bool,
}

impl StrictTransportSecurity {
    pub fn to_header_value(&self) -> String {
        let mut value = format!("max-age={}", self.max_age);
        if self.include_subdomains {
            value.push_str("; includeSubDomains");
        }
        if self.preload {
            value.push_str("; preload");
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct PermissionsPolicy {
    pub accelerometer: Vec<String>,
    pub ambient_light_sensor: Vec<String>,
    pub autoplay: Vec<String>,
    pub battery: Vec<String>,
    pub camera: Vec<String>,
    pub display_capture: Vec<String>,
    pub document_domain: Vec<String>,
    pub encrypted_media: Vec<String>,
    pub execution_while_not_rendered: Vec<String>,
    pub execution_while_out_of_viewport: Vec<String>,
    pub fullscreen: Vec<String>,
    pub geolocation: Vec<String>,
    pub gyroscope: Vec<String>,
    pub magnetometer: Vec<String>,
    pub microphone: Vec<String>,
    pub midi: Vec<String>,
    pub navigation_override: Vec<String>,
    pub payment: Vec<String>,
    pub picture_in_picture: Vec<String>,
    pub publickey_credentials: Vec<String>,
    pub sync_xhr: Vec<String>,
    pub usb: Vec<String>,
    pub wake_lock: Vec<String>,
    pub xr_spatial_tracking: Vec<String>,
}

impl Default for PermissionsPolicy {
    fn default() -> Self {
        let none = vec!["()".to_string()];
        let self_only = vec!["(self)".to_string()];
        
        Self {
            accelerometer: none.clone(),
            ambient_light_sensor: none.clone(),
            autoplay: none.clone(),
            battery: none.clone(),
            camera: none.clone(),
            display_capture: none.clone(),
            document_domain: none.clone(),
            encrypted_media: none.clone(),
            execution_while_not_rendered: none.clone(),
            execution_while_out_of_viewport: none.clone(),
            fullscreen: self_only.clone(),
            geolocation: none.clone(),
            gyroscope: none.clone(),
            magnetometer: none.clone(),
            microphone: none.clone(),
            midi: none.clone(),
            navigation_override: none.clone(),
            payment: none.clone(),
            picture_in_picture: self_only,
            publickey_credentials: none.clone(),
            sync_xhr: vec!["*".to_string()],
            usb: none.clone(),
            wake_lock: none.clone(),
            xr_spatial_tracking: none,
        }
    }
}

impl PermissionsPolicy {
    pub fn to_header_value(&self) -> String {
        let mut directives = Vec::new();
        
        let add_directive = |name: &str, values: &[String]| -> String {
            format!("{}={}", name, values.join(" "))
        };
        
        directives.push(add_directive("accelerometer", &self.accelerometer));
        directives.push(add_directive("ambient-light-sensor", &self.ambient_light_sensor));
        directives.push(add_directive("autoplay", &self.autoplay));
        directives.push(add_directive("battery", &self.battery));
        directives.push(add_directive("camera", &self.camera));
        directives.push(add_directive("display-capture", &self.display_capture));
        directives.push(add_directive("document-domain", &self.document_domain));
        directives.push(add_directive("encrypted-media", &self.encrypted_media));
        directives.push(add_directive("execution-while-not-rendered", &self.execution_while_not_rendered));
        directives.push(add_directive("execution-while-out-of-viewport", &self.execution_while_out_of_viewport));
        directives.push(add_directive("fullscreen", &self.fullscreen));
        directives.push(add_directive("geolocation", &self.geolocation));
        directives.push(add_directive("gyroscope", &self.gyroscope));
        directives.push(add_directive("magnetometer", &self.magnetometer));
        directives.push(add_directive("microphone", &self.microphone));
        directives.push(add_directive("midi", &self.midi));
        directives.push(add_directive("navigation-override", &self.navigation_override));
        directives.push(add_directive("payment", &self.payment));
        directives.push(add_directive("picture-in-picture", &self.picture_in_picture));
        directives.push(add_directive("publickey-credentials", &self.publickey_credentials));
        directives.push(add_directive("sync-xhr", &self.sync_xhr));
        directives.push(add_directive("usb", &self.usb));
        directives.push(add_directive("wake-lock", &self.wake_lock));
        directives.push(add_directive("xr-spatial-tracking", &self.xr_spatial_tracking));
        
        directives.join(", ")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CrossOriginEmbedderPolicy {
    UnsafeNone,
    RequireCorp,
    Credentialless,
}

impl CrossOriginEmbedderPolicy {
    pub fn to_header_value(&self) -> &'static str {
        match self {
            CrossOriginEmbedderPolicy::UnsafeNone => "unsafe-none",
            CrossOriginEmbedderPolicy::RequireCorp => "require-corp",
            CrossOriginEmbedderPolicy::Credentialless => "credentialless",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CrossOriginOpenerPolicy {
    UnsafeNone,
    SameOriginAllowPopups,
    SameOrigin,
}

impl CrossOriginOpenerPolicy {
    pub fn to_header_value(&self) -> &'static str {
        match self {
            CrossOriginOpenerPolicy::UnsafeNone => "unsafe-none",
            CrossOriginOpenerPolicy::SameOriginAllowPopups => "same-origin-allow-popups",
            CrossOriginOpenerPolicy::SameOrigin => "same-origin",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CrossOriginResourcePolicy {
    SameSite,
    SameOrigin,
    CrossOrigin,
}

impl CrossOriginResourcePolicy {
    pub fn to_header_value(&self) -> &'static str {
        match self {
            CrossOriginResourcePolicy::SameSite => "same-site",
            CrossOriginResourcePolicy::SameOrigin => "same-origin",
            CrossOriginResourcePolicy::CrossOrigin => "cross-origin",
        }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersState {
    #[cfg(feature = "staging")]
    config: Arc<SecurityHeadersConfig>,
}

impl SecurityHeadersState {
    pub fn new(_config: SecurityHeadersConfig) -> Self {
        Self {
            #[cfg(feature = "staging")]
            config: Arc::new(config),
        }
    }
    
    pub fn production() -> Self {
        Self::new(SecurityHeadersConfig::default())
    }
    
    pub fn development() -> Self {
        Self::new(SecurityHeadersConfig {
            content_security_policy: ContentSecurityPolicy::development(),
            x_frame_options: XFrameOptions::SameOrigin,
            x_content_type_options: true,
            strict_transport_security: None,
            x_xss_protection: true,
            referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            permissions_policy: None,
            cross_origin_embedder_policy: None,
            cross_origin_opener_policy: None,
            cross_origin_resource_policy: None,
        })
    }
}

pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;
    add_security_headers(response)
}

pub fn add_security_headers(response: Response) -> Response {
    add_security_headers_with_config_opts(response, &Default::default())
}

pub fn add_security_headers_from_config(response: Response, config: &ServerSecurityConfig) -> Response {
    add_security_headers_with_config_opts(response, config)
}

fn add_security_headers_with_config_opts(mut response: Response, config: &ServerSecurityConfig) -> Response {
    let headers = response.headers_mut();
    let is_dev = config.is_development();
    
    if config.csp_enabled {
        let mut csp = if is_dev {
            ContentSecurityPolicy::development()
        } else {
            ContentSecurityPolicy::default()
        };
        
        if !config.frame_ancestors.is_empty() {
            csp.frame_ancestors = vec![config.frame_ancestors.clone()];
        }
        
        let csp_value = config.csp_custom.as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| csp.to_header_value());
        
        if config.csp_report_only {
            if let Ok(value) = HeaderValue::from_str(&csp_value) {
                headers.insert(
                    "Content-Security-Policy-Report-Only",
                    value,
                );
            }
        } else if let Ok(value) = HeaderValue::from_str(&csp_value) {
            headers.insert(
                header::CONTENT_SECURITY_POLICY,
                value,
            );
        }
    }
    
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    if config.is_hsts_enabled() && !is_dev {
        let sts = StrictTransportSecurity {
            max_age: config.hsts_max_age,
            include_subdomains: config.hsts_include_subdomains,
            preload: config.hsts_preload,
        };
        if let Ok(value) = HeaderValue::from_str(&sts.to_header_value()) {
            headers.insert(header::STRICT_TRANSPORT_SECURITY, value);
        }
    }
    
    if config.permissions_policy {
        let permissions = PermissionsPolicy::default();
        if let Ok(value) = HeaderValue::from_str(&permissions.to_header_value()) {
            headers.insert("Permissions-Policy", value);
        }
    }
    
    if config.coep_enabled {
        let coep_value = if is_dev { "unsafe-none" } else { "credentialless" };
        headers.insert(
            "Cross-Origin-Embedder-Policy",
            HeaderValue::from_static(coep_value),
        );
    }
    
    if !is_dev {
        headers.insert(
            "Cross-Origin-Opener-Policy",
            HeaderValue::from_static("same-origin"),
        );
        
        headers.insert(
            "Cross-Origin-Resource-Policy",
            HeaderValue::from_static("same-origin"),
        );
    }
    
    response
}

pub fn add_security_headers_with_config(mut response: Response, config: &SecurityHeadersConfig) -> Response {
    let headers = response.headers_mut();
    
    if let Ok(value) = HeaderValue::from_str(&config.content_security_policy.to_header_value()) {
        headers.insert(header::CONTENT_SECURITY_POLICY, value);
    }
    
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static(config.x_frame_options.to_header_value()),
    );
    
    if config.x_content_type_options {
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
    }
    
    if config.x_xss_protection {
        headers.insert(
            "X-XSS-Protection",
            HeaderValue::from_static("1; mode=block"),
        );
    }
    
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static(config.referrer_policy.to_header_value()),
    );
    
    if let Some(ref sts) = config.strict_transport_security {
        if let Ok(value) = HeaderValue::from_str(&sts.to_header_value()) {
            headers.insert(header::STRICT_TRANSPORT_SECURITY, value);
        }
    }
    
    if let Some(ref permissions) = config.permissions_policy {
        if let Ok(value) = HeaderValue::from_str(&permissions.to_header_value()) {
            headers.insert("Permissions-Policy", value);
        }
    }
    
    if let Some(coep) = config.cross_origin_embedder_policy {
        headers.insert(
            "Cross-Origin-Embedder-Policy",
            HeaderValue::from_static(coep.to_header_value()),
        );
    }
    
    if let Some(coop) = config.cross_origin_opener_policy {
        headers.insert(
            "Cross-Origin-Opener-Policy",
            HeaderValue::from_static(coop.to_header_value()),
        );
    }
    
    if let Some(corp) = config.cross_origin_resource_policy {
        headers.insert(
            "Cross-Origin-Resource-Policy",
            HeaderValue::from_static(corp.to_header_value()),
        );
    }
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_csp_header_generation() {
        let csp = ContentSecurityPolicy::default();
        let header = csp.to_header_value();
        
        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'self'"));
        assert!(header.contains("object-src 'none'"));
        assert!(header.contains("upgrade-insecure-requests"));
    }
    
    #[test]
    fn test_sts_header_generation() {
        let sts = StrictTransportSecurity {
            max_age: 31536000,
            include_subdomains: true,
            preload: true,
        };
        
        let header = sts.to_header_value();
        assert!(header.contains("max-age=31536000"));
        assert!(header.contains("includeSubDomains"));
        assert!(header.contains("preload"));
    }
    
    #[test]
    fn test_x_frame_options() {
        assert_eq!(XFrameOptions::Deny.to_header_value(), "DENY");
        assert_eq!(XFrameOptions::SameOrigin.to_header_value(), "SAMEORIGIN");
    }
    
    #[test]
    fn test_referrer_policy() {
        assert_eq!(
            ReferrerPolicy::StrictOriginWhenCrossOrigin.to_header_value(),
            "strict-origin-when-cross-origin"
        );
    }
}
