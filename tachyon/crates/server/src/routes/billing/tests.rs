#[cfg(test)]
mod tests {
    use crate::routes::billing::*;
    use axum::http::StatusCode;
    use chrono::{Duration, Utc};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type TestHmacSha256 = Hmac<Sha256>;

    fn generate_hmac_sha256(secret: &str, payload: &[u8]) -> String {
        let mut mac = TestHmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn test_validate_plan_name_valid() {
        for plan in &["free", "pro", "team", "enterprise"] {
            assert!(
                validate_plan_name(plan).is_ok(),
                "plan '{}' should be valid",
                plan
            );
        }
    }

    #[test]
    fn test_validate_plan_name_invalid() {
        assert!(validate_plan_name("premium").is_err());
        assert!(validate_plan_name("basic").is_err());
        assert!(validate_plan_name("").is_err());
    }

    #[test]
    fn test_validate_plan_transition_upgrades() {
        assert_eq!(
            validate_plan_transition("free", "pro").unwrap(),
            TransitionType::Upgrade
        );
        assert_eq!(
            validate_plan_transition("free", "team").unwrap(),
            TransitionType::Upgrade
        );
        assert_eq!(
            validate_plan_transition("free", "enterprise").unwrap(),
            TransitionType::Upgrade
        );
        assert_eq!(
            validate_plan_transition("pro", "team").unwrap(),
            TransitionType::Upgrade
        );
        assert_eq!(
            validate_plan_transition("pro", "enterprise").unwrap(),
            TransitionType::Upgrade
        );
        assert_eq!(
            validate_plan_transition("team", "enterprise").unwrap(),
            TransitionType::Upgrade
        );
    }

    #[test]
    fn test_validate_plan_transition_downgrades() {
        assert_eq!(
            validate_plan_transition("pro", "free").unwrap(),
            TransitionType::Downgrade
        );
        assert_eq!(
            validate_plan_transition("team", "pro").unwrap(),
            TransitionType::Downgrade
        );
        assert_eq!(
            validate_plan_transition("team", "free").unwrap(),
            TransitionType::Downgrade
        );
    }

    #[test]
    fn test_validate_plan_transition_same_plan() {
        let result = validate_plan_transition("pro", "pro");
        assert!(result.is_err());
        let (status, body) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body.code, "SAME_PLAN");
    }

    #[test]
    fn test_validate_plan_transition_enterprise_blocked() {
        let result = validate_plan_transition("enterprise", "free");
        assert!(result.is_err());
        let (status, body) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body.code, "ENTERPRISE_CHANGE_REQUIRES_ADMIN");
    }

    #[test]
    fn test_validate_plan_transition_enterprise_upgrade_blocked() {
        let result = validate_plan_transition("enterprise", "team");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_calculate_proration_mid_period_upgrade() {
        let start = Utc::now() - Duration::days(15);
        let end = Utc::now() + Duration::days(15);
        let now = Utc::now();

        let result = calculate_proration(12.0, 29.0, start, end, now);

        assert!(result.prorated_amount > 0.0);
        assert!(result.charge > result.credit);
        assert_eq!(result.days_remaining, 15);
    }

    #[test]
    fn test_calculate_proration_downgrade() {
        let start = Utc::now() - Duration::days(15);
        let end = Utc::now() + Duration::days(15);
        let now = Utc::now();

        let result = calculate_proration(29.0, 12.0, start, end, now);

        assert!(result.prorated_amount < 0.0);
        assert!(result.charge < result.credit);
        assert_eq!(result.days_remaining, 15);
    }

    #[test]
    fn test_calculate_proration_free_upgrade() {
        let start = Utc::now() - Duration::days(15);
        let end = Utc::now() + Duration::days(15);
        let now = Utc::now();

        let result = calculate_proration(0.0, 12.0, start, end, now);

        assert_eq!(result.credit, 0.0);
        assert!(result.charge > 0.0);
        assert!(result.prorated_amount > 0.0);
    }

    #[test]
    fn test_calculate_proration_downgrade_to_free() {
        let start = Utc::now() - Duration::days(15);
        let end = Utc::now() + Duration::days(15);
        let now = Utc::now();

        let result = calculate_proration(12.0, 0.0, start, end, now);

        assert_eq!(result.charge, 0.0);
        assert!(result.credit > 0.0);
        assert!(result.prorated_amount < 0.0);
    }

    #[test]
    fn test_calculate_proration_at_period_end() {
        let start = Utc::now() - Duration::days(30);
        let end = Utc::now();
        let now = Utc::now();

        let result = calculate_proration(12.0, 29.0, start, end, now);

        assert_eq!(result.days_remaining, 0);
        assert!(result.prorated_amount.abs() < 0.01);
    }

    #[test]
    fn test_plan_limits() {
        assert_eq!(Plan::Free.max_documents(), 100);
        assert_eq!(Plan::Free.max_members(), 1);

        assert_eq!(Plan::Pro.max_documents(), 10_000);
        assert_eq!(Plan::Pro.max_members(), 5);

        assert_eq!(Plan::Team.max_documents(), 100_000);
        assert_eq!(Plan::Team.max_members(), 50);

        assert_eq!(Plan::Enterprise.max_documents(), usize::MAX);
        assert_eq!(Plan::Enterprise.max_members(), usize::MAX);
    }

    #[test]
    fn test_plan_prices() {
        assert_eq!(Plan::Free.price_monthly(), 0);
        assert_eq!(Plan::Pro.price_monthly(), 12_00);
        assert_eq!(Plan::Team.price_monthly(), 29_00);
        assert_eq!(Plan::Enterprise.price_monthly(), 0);
    }

    #[test]
    fn test_plan_price_f64() {
        assert!((plan_price_f64("free") - 0.0).abs() < 0.001);
        assert!((plan_price_f64("pro") - 12.0).abs() < 0.001);
        assert!((plan_price_f64("team") - 29.0).abs() < 0.001);
    }

    #[test]
    fn test_change_plan_request_deserialize() {
        let json = r#"{"organization_id":"org-1","new_plan":"pro"}"#;
        let req: ChangePlanRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.organization_id, "org-1");
        assert_eq!(req.new_plan, "pro");
    }

    #[test]
    fn test_change_plan_response_serialize() {
        let resp = ChangePlanResponse {
            subscription_id: "sub-1".to_string(),
            old_plan: "free".to_string(),
            new_plan: "pro".to_string(),
            status: "immediate".to_string(),
            effective_at: "2025-01-01T00:00:00+00:00".to_string(),
            prorated_amount: Some(6.50),
            next_billing_date: Some("2025-02-01T00:00:00+00:00".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("immediate"));
        assert!(json.contains("6.5"));
    }

    #[test]
    fn test_verify_valid_signature() {
        let secret = "test_secret";
        let payload = r#"{"event": "payment.created"}"#;
        let signature = generate_hmac_sha256(secret, payload.as_bytes());
        let header = format!("v1={}", signature);
        assert!(verify_webhook_signature(
            payload.as_bytes(),
            &header,
            secret
        ));
    }

    #[test]
    fn test_verify_valid_signature_without_prefix() {
        let secret = "test_secret";
        let payload = r#"{"event": "payment.created"}"#;
        let signature = generate_hmac_sha256(secret, payload.as_bytes());
        assert!(verify_webhook_signature(
            payload.as_bytes(),
            &signature,
            secret
        ));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let secret = "test_secret";
        let payload = r#"{"event": "payment.created"}"#;
        assert!(!verify_webhook_signature(
            payload.as_bytes(),
            "v1=completely_wrong_signature",
            secret
        ));
    }

    #[test]
    fn test_verify_different_secret_fails() {
        let secret = "correct_secret";
        let wrong_secret = "wrong_secret";
        let payload = r#"{"event": "payment.created"}"#;
        let signature = generate_hmac_sha256(wrong_secret, payload.as_bytes());
        let header = format!("v1={}", signature);
        assert!(!verify_webhook_signature(
            payload.as_bytes(),
            &header,
            secret
        ));
    }

    #[test]
    fn test_verify_empty_signature_fails() {
        let secret = "test_secret";
        let payload = r#"{"event": "payment.created"}"#;
        assert!(!verify_webhook_signature(payload.as_bytes(), "", secret));
    }

    #[test]
    fn test_verify_empty_payload() {
        let secret = "test_secret";
        let payload = b"";
        let signature = generate_hmac_sha256(secret, payload);
        let header = format!("v1={}", signature);
        assert!(verify_webhook_signature(payload, &header, secret));
    }

    #[test]
    fn test_verify_empty_payload_wrong_signature() {
        let secret = "test_secret";
        let payload = b"";
        assert!(!verify_webhook_signature(
            payload,
            "v1=0000000000000000",
            secret
        ));
    }

    #[test]
    fn test_verify_different_payload_fails() {
        let secret = "test_secret";
        let payload_a = r#"{"event": "payment.created"}"#;
        let payload_b = r#"{"event": "payment.failed"}"#;
        let signature = generate_hmac_sha256(secret, payload_a.as_bytes());
        let header = format!("v1={}", signature);
        assert!(!verify_webhook_signature(
            payload_b.as_bytes(),
            &header,
            secret
        ));
    }

    #[test]
    fn test_verify_empty_secret() {
        let secret = "";
        let payload = r#"{"event": "payment.created"}"#;
        let signature = generate_hmac_sha256(secret, payload.as_bytes());
        let header = format!("v1={}", signature);
        assert!(verify_webhook_signature(
            payload.as_bytes(),
            &header,
            secret
        ));
    }

    #[test]
    fn test_signature_is_constant_time() {
        let secret = "test_secret";
        let payload = r#"{"event": "payment.created"}"#;
        let signature = generate_hmac_sha256(secret, payload.as_bytes());
        let header = format!("v1={}", signature);
        assert!(verify_webhook_signature(
            payload.as_bytes(),
            &header,
            secret
        ));
    }
}
