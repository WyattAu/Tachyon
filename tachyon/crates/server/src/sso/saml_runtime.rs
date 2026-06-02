//! SAML 2.0 runtime flow.
//!
//! Provides SP-initiated SSO:
//! 1. `GET /auth/sso/saml/metadata` -- SP metadata XML for IdP configuration
//! 2. `POST /auth/sso/saml/acs` -- Assertion Consumer Service (ACS) endpoint

use axum::{extract::State, response::Json};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::io::Read;
use tracing::{debug, info, warn};

use super::saml::{SamlAttribute, SamlConfig};
use crate::error::ServerError;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct SamlState {
    pub config: SamlConfig,
    pub pool: tachyon_database::DatabasePool,
    pub jwt_secret: String,
}

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SamlAcsRequest {
    pub saml_response: String,
    pub relay_state: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SamlMetadataResponse {
    pub entity_id: String,
    pub acs_url: String,
    pub metadata_url: String,
    pub sp_metadata_xml: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SamlAcsResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub name_id: String,
    pub attributes: Vec<SamlAttribute>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Generate SP metadata XML for SAML IdP configuration.
///
/// `GET /api/v1/auth/sso/saml/metadata`
#[utoipa::path(
    get,
    path = "/api/v1/auth/sso/saml/metadata",
    responses(
        (status = 200, description = "SP metadata XML", body = SamlMetadataResponse),
    ),
    tag = "auth",
)]
pub async fn saml_metadata(
    State(state): State<SamlState>,
) -> Result<Json<SamlMetadataResponse>, ServerError> {
    info!("Generating SAML SP metadata");

    let xml = generate_sp_metadata_xml(&state.config);

    Ok(Json(SamlMetadataResponse {
        entity_id: state.config.entity_id.clone(),
        acs_url: state.config.acs_url.clone(),
        metadata_url: state.config.metadata_url.clone(),
        sp_metadata_xml: xml,
    }))
}

/// SAML Assertion Consumer Service endpoint.
///
/// Receives base64-encoded SAMLResponse from IdP, decodes, inflates, parses
/// the assertion, extracts NameID and attributes, creates/updates user, issues JWT.
///
/// `POST /api/v1/auth/sso/saml/acs`
#[utoipa::path(
    post,
    path = "/api/v1/auth/sso/saml/acs",
    responses(
        (status = 200, description = "SAML authentication successful", body = SamlAcsResponse),
        (status = 400, description = "Invalid SAML response"),
    ),
    tag = "auth",
)]
pub async fn saml_acs(
    State(state): State<SamlState>,
    axum::Form(form): axum::Form<SamlAcsRequest>,
) -> Result<Json<SamlAcsResponse>, ServerError> {
    info!("Received SAML ACS request");

    let decoded = decode_saml_response(&form.saml_response)?;
    let xml_str = inflate_if_needed(&decoded)?;

    let parsed = parse_saml_response(&xml_str)?;

    warn!(
        "SAML XML-DSig signature verification skipped -- \
         TODO: Full XML-DSig verification requires xmldsig-rs or similar crate. \
         Extracted certificate reference present: {}",
        parsed.signature_present
    );

    if let Some(ref cert_ref) = parsed.signature_cert_reference {
        debug!(certificate_reference = cert_ref, "SAML response signature reference");
    }

    debug!(
        name_id = &parsed.name_id,
        issuer = &parsed.issuer,
        num_attributes = parsed.attributes.len(),
        "Parsed SAML assertion"
    );

    upsert_saml_user(&state.pool, &parsed.name_id, &parsed.issuer, &parsed.attributes).await?;

    let now = jsonwebtoken::get_current_timestamp();
    let exp = now + 3600;

    let claims = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::HS256,
        ..Default::default()
    };

    let token_claims = serde_json::json!({
        "sub": parsed.name_id,
        "name": parsed.name_id.clone(),
        "role": "user",
        "iss": "tachyon",
        "aud": "tachyon-api",
        "exp": exp,
        "iat": now,
        "sso_provider": "saml",
        "sso_issuer": parsed.issuer,
    });

    let token = jsonwebtoken::encode(
        &claims,
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| ServerError::internal(format!("JWT creation failed: {}", e)))?;

    Ok(Json(SamlAcsResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        name_id: parsed.name_id,
        attributes: parsed.attributes,
    }))
}

// ============================================================================
// Internal helpers
// ============================================================================

struct ParsedSamlResponse {
    name_id: String,
    issuer: String,
    attributes: Vec<SamlAttribute>,
    signature_present: bool,
    signature_cert_reference: Option<String>,
}

fn decode_saml_response(encoded: &str) -> Result<Vec<u8>, ServerError> {
    let engine = base64::engine::general_purpose::STANDARD;
    engine
        .decode(encoded.trim())
        .map_err(|e| ServerError::bad_request(format!("Failed to base64-decode SAMLResponse: {}", e)))
}

fn inflate_if_needed(data: &[u8]) -> Result<String, ServerError> {
    if data.len() >= 2 && (data[0] == 0x78) {
        let mut decoder = flate2::read::DeflateDecoder::new(data);
        let mut decompressed = String::new();
        decoder
            .read_to_string(&mut decompressed)
            .map_err(|e| {
                ServerError::bad_request(format!("Failed to inflate SAMLResponse: {}", e))
            })?;
        Ok(decompressed)
    } else {
        String::from_utf8(data.to_vec()).map_err(|e| {
            ServerError::bad_request(format!("SAMLResponse is not valid UTF-8: {}", e))
        })
    }
}

fn parse_saml_response(xml_str: &str) -> Result<ParsedSamlResponse, ServerError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml_str);
    reader.config_mut().trim_text(true);

    let mut name_id: Option<String> = None;
    let mut issuer: Option<String> = None;
    let mut attributes: Vec<SamlAttribute> = Vec::new();
    let mut signature_present = false;
    let mut signature_cert_reference: Option<String> = None;

    let mut in_assertion = false;
    let mut in_name_id = false;
    let mut in_issuer = false;
    let mut in_attribute = false;
    let mut current_attr_name: Option<String> = None;
    let mut current_attr_friendly_name: Option<String> = None;
    let mut current_attr_values: Vec<String> = Vec::new();
    let mut in_attribute_value = false;
    let mut in_x509_cert = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let qname = e.local_name();
                let local: &[u8] = qname.as_ref();

                if local == b"Assertion" {
                    in_assertion = true;
                } else if local == b"NameID" && in_assertion {
                    in_name_id = true;
                } else if local == b"Issuer" && in_assertion && name_id.is_none() {
                    in_issuer = true;
                    issuer = Some(String::new());
                } else if local == b"Attribute" && in_assertion {
                    in_attribute = true;
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        if key == b"Name" {
                            current_attr_name = Some(val);
                        } else if key == b"FriendlyName" {
                            current_attr_friendly_name = Some(val);
                        }
                    }
                    current_attr_values = Vec::new();
                } else if local == b"AttributeValue" && in_attribute {
                    in_attribute_value = true;
                } else if local == b"Signature" && in_assertion {
                    signature_present = true;
                } else if local == b"X509Certificate" {
                    in_x509_cert = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if text.is_empty() {
                    continue;
                }

                if in_x509_cert && signature_present {
                    signature_cert_reference = Some(text.trim().to_string());
                    in_x509_cert = false;
                } else if in_name_id {
                    name_id = Some(text);
                    in_name_id = false;
                } else if let Some(ref mut issuer_val) = issuer {
                    if in_issuer && issuer_val.is_empty() {
                        *issuer_val = text;
                    }
                } else if in_attribute_value {
                    current_attr_values.push(text);
                    in_attribute_value = false;
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let local = local.as_ref();
                if local == b"Attribute" && in_attribute {
                    if let Some(attr_name) = current_attr_name.take() {
                        attributes.push(SamlAttribute {
                            name: attr_name,
                            friendly_name: current_attr_friendly_name.take(),
                            values: std::mem::take(&mut current_attr_values),
                        });
                    }
                    in_attribute = false;
                } else if local == b"NameID" {
                    in_name_id = false;
                } else if local == b"Issuer" {
                    in_issuer = false;
                } else if local == b"X509Certificate" {
                    in_x509_cert = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ServerError::bad_request(format!(
                    "Failed to parse SAML response XML: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    let name_id = name_id.ok_or_else(|| {
        ServerError::bad_request("SAML response contains no NameID in the assertion")
    })?;
    let issuer = issuer.unwrap_or_else(|| "unknown".to_string());

    Ok(ParsedSamlResponse {
        name_id,
        issuer,
        attributes,
        signature_present,
        signature_cert_reference,
    })
}

async fn upsert_saml_user(
    pool: &tachyon_database::DatabasePool,
    name_id: &str,
    issuer: &str,
    _attributes: &[SamlAttribute],
) -> Result<(), ServerError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| ServerError::internal(format!("Database connection failed: {}", e)))?;

    let sso_provider = "saml".to_string();
    let sso_subject = format!("{}:{}", issuer, name_id);

    let result = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM users WHERE sso_provider = $1 AND sso_subject = $2",
    )
    .bind(&sso_provider)
    .bind(&sso_subject)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| ServerError::internal(format!("User lookup failed: {}", e)))?;

    if result.is_some() {
        sqlx::query(
            "UPDATE users SET display_name = $1, updated_at = NOW() WHERE sso_provider = $2 AND sso_subject = $3",
        )
        .bind(name_id)
        .bind(&sso_provider)
        .bind(&sso_subject)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::internal(format!("User update failed: {}", e)))?;

        debug!("Updated existing SAML user: {} ({})", name_id, issuer);
    } else {
        let user_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO users (id, username, display_name, email, sso_provider, sso_subject, created_at, updated_at)
               VALUES ($1, $2, $3, NULL, $4, $5, NOW(), NOW())"#,
        )
        .bind(&user_id)
        .bind(name_id)
        .bind(name_id)
        .bind(&sso_provider)
        .bind(&sso_subject)
        .execute(&mut *conn)
        .await
        .map_err(|e| ServerError::internal(format!("User creation failed: {}", e)))?;

        info!("Created new SAML user: {} ({})", name_id, issuer);
    }

    Ok(())
}

fn generate_sp_metadata_xml(config: &SamlConfig) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                         entityID="{entity_id}"
                         validUntil="2027-12-31T23:59:59Z">
  <md:SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true"
                        protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>
      <saml:NameIDFormat xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
        urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress
      </saml:NameIDFormat>
      <saml:NameIDFormat xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
        urn:oasis:names:tc:SAML:2.0:nameid-format:transient
      </saml:NameIDFormat>
    </md:NameIDFormat>
    <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                                 Location="{acs_url}" />
    <md:SingleLogoutService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-REDIRECT"
                           Location="{acs_url}/logout" />
  </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
        entity_id = config.entity_id,
        acs_url = config.acs_url,
    )
}

// ============================================================================
// Router
// ============================================================================

pub fn create_saml_router() -> axum::Router<SamlState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/saml/metadata", get(saml_metadata))
        .route("/saml/acs", post(saml_acs))
}
