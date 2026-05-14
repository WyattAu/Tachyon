# Security Incident Runbook

## Severity: Critical

A security incident includes unauthorized access, data breach, compromised credentials, injection attacks, or any threat to system confidentiality, integrity, or availability.

## Detection Methods

1. **Authentication anomalies**: Spike in `AUTH_ERROR` responses, unusual login patterns
2. **Audit logs**: `audit.log` shows unauthorized access attempts or privilege escalation
3. **Rate limit triggers**: Sustained `RATE_LIMITED` responses from specific IPs
4. **Webhook failures**: Unexpected `INVALID_SIGNATURE` on billing webhooks
5. **External reports**: User reports, vulnerability disclosures, bug bounty submissions
6. **Dependency scanning**: CVE alerts in `cargo audit` or Dependabot
7. **Unusual activity**: Unexpected `DELETE` operations, mass data exports, or admin role changes

## Response Procedure

### 1. Assess and Contain (immediate)

```bash
# Enable audit logging if not already active (check middleware/audit.rs)
# Review recent authentication events
grep "AUTH_ERROR\|UNAUTHORIZED" /var/log/tachyon/app.log | tail -100

# Check for brute force attempts
grep "rate_limited\|RATE_LIMITED" /var/log/tachyon/app.log | tail -100

# Block malicious IPs at the firewall if identified
sudo iptables -A INPUT -s <malicious_ip> -j DROP

# Revoke all sessions if credentials are compromised
curl -X DELETE http://localhost:8080/api/v1/users/{user_id}/sessions \
  -H "Authorization: Bearer <admin_token>"
```

### 2. Investigate

- **Identify the scope**: Which users, data, or systems are affected?
- **Determine the vector**: How did the attacker gain access?
  - Stolen credentials (check password reset logs)
  - SQL injection (check query logs)
  - XSS (check document content for injected scripts)
  - JWT compromise (check `TACHYON_JWT_SECRETS` exposure)
  - OAuth2 CSRF bypass (check state validation logs)
- **Timeline**: When did the incident start? What actions were taken?
- **Data impact**: Which records were accessed, modified, or deleted?

```bash
# Check for unusual admin actions
grep "role.*admin\|delete\|update.*user" /var/log/tachyon/audit.log | tail -200

# Check database for unauthorized changes
psql -c "SELECT * FROM audit_log WHERE action IN ('delete', 'update_role', 'create_user') ORDER BY created_at DESC LIMIT 50;"
```

### 3. Remediate

- **Rotate secrets immediately**:
  ```bash
  # Generate new JWT secrets
  export TACHYON_JWT_SECRETS="new_secret_1,new_secret_2"

  # Rotate database password
  psql -c "ALTER USER tachyon WITH PASSWORD 'new_password';"

  # Rotate OAuth2 secrets (Google, GitHub)
  # Rotate TrueLayer webhook secret
  # Rotate webhook URL secrets
  ```

- **Force password reset for affected users**:
  ```bash
  psql -c "UPDATE users SET password_hash = NULL WHERE id IN (<affected_ids>);"
  ```

- **Disable compromised accounts**:
  ```bash
  psql -c "UPDATE users SET is_active = false WHERE id = '<compromised_user_id>';"
  ```

- **Revoke all active sessions**:
  ```bash
  psql -c "UPDATE sessions SET revoked_at = NOW() WHERE revoked_at IS NULL;"
  ```

### 4. Recover

- Restore any deleted or modified data from backups
- Rebuild search index: `POST /api/v1/search/reindex`
- Verify all endpoints return expected data
- Run full test suite

### 5. Communicate

- Notify affected users within 72 hours (GDPR requirement)
- Document the incident timeline and impact
- Report to relevant authorities if personal data was breached

## Prevention Measures

- Enable rate limiting on all authentication endpoints
- Enforce MFA for admin accounts
- Rotate JWT secrets regularly (support multiple via comma-separated `TACHYON_JWT_SECRETS`)
- Run `cargo audit` in CI pipeline
- Enable CSP headers via security middleware
- Implement IP allowlisting for admin endpoints
- Regular penetration testing
- Monitor audit logs with automated anomaly detection
- Keep dependencies up to date
