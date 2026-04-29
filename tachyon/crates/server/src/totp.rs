use base32::{encode, Alphabet};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::time::{Duration, SystemTime};

type HmacSha1 = Hmac<Sha1>;

pub fn generate_secret() -> String {
    let mut rng = rand::thread_rng();
    let secret: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
    encode(Alphabet::Rfc4648 { padding: false }, &secret)
}

pub fn generate_totp(secret: &str) -> Result<u32, TotpError> {
    generate_totp_at(secret, SystemTime::now())
}

pub fn generate_totp_at(secret: &str, time: SystemTime) -> Result<u32, TotpError> {
    let bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
        .ok_or(TotpError::InvalidSecret)?;

    let time_step = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| TotpError::InvalidTime)?
        .as_secs()
        / 30;

    let time_bytes = time_step.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(&bytes).map_err(|_| TotpError::InvalidSecret)?;
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0x0f) as usize;
    let code = ((result[offset] & 0x7f) as u32) << 24
        | (result[offset + 1] as u32) << 16
        | (result[offset + 2] as u32) << 8
        | (result[offset + 3] as u32);

    Ok(code % 1_000_000)
}

pub fn verify_totp(secret: &str, code: u32) -> Result<bool, TotpError> {
    let now = SystemTime::now();

    if generate_totp_at(secret, now)? == code {
        return Ok(true);
    }

    if generate_totp_at(secret, now - Duration::from_secs(30))? == code {
        return Ok(true);
    }

    if generate_totp_at(secret, now + Duration::from_secs(30))? == code {
        return Ok(true);
    }

    Ok(false)
}

pub fn generate_backup_codes(count: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let code: u32 = rng.gen_range(10000000..99999999);
            format!("{:08}", code)
        })
        .collect()
}

pub fn generate_otpauth_uri(secret: &str, email: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        urlencoding::encode(issuer),
        urlencoding::encode(email),
        urlencoding::encode(secret),
        urlencoding::encode(issuer),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum TotpError {
    #[error("Invalid TOTP secret")]
    InvalidSecret,
    #[error("Invalid time")]
    InvalidTime,
    #[error("TOTP verification failed")]
    VerificationFailed,
    #[error("TOTP not enabled")]
    NotEnabled,
    #[error("Invalid backup code")]
    InvalidBackupCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnableTotpResponse {
    pub secret: String,
    pub qr_code_uri: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTotpRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct DisableTotpRequest {
    pub code: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = generate_secret();
        assert!(secret.len() >= 20);
        assert!(secret
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_and_verify_totp() {
        let secret = generate_secret();
        let code = generate_totp(&secret).unwrap();
        assert!(verify_totp(&secret, code).unwrap());
    }

    #[test]
    fn test_verify_wrong_code() {
        let secret = generate_secret();
        assert!(!verify_totp(&secret, 0).unwrap());
    }

    #[test]
    fn test_generate_backup_codes() {
        let codes = generate_backup_codes(10);
        assert_eq!(codes.len(), 10);
        assert!(codes
            .iter()
            .all(|c| c.len() == 8 && c.parse::<u32>().is_ok()));
    }

    #[test]
    fn test_otpauth_uri() {
        let uri = generate_otpauth_uri("SECRET", "user@example.com", "Tachyon");
        assert!(uri.contains("otpauth://totp"));
        assert!(uri.contains("Tachyon"));
        assert!(uri.contains("SECRET"));
    }
}
