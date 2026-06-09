# Data Retention and Disposal Policies

**Policy ID:** DR-001
**Version:** 1.0
**Effective Date:** 2026-06-09
**Review Cycle:** Annual
**Classification:** Internal

## 1. Purpose

This document defines the data retention and secure disposal procedures for the Tachyon platform to ensure compliance with regulatory requirements and protection of customer data.

## 2. Scope

Applies to all data processed by Tachyon:

- Customer documents and knowledge base content
- User account information
- Authentication credentials and tokens
- System logs and audit trails
- Configuration data
- Temporary and cached data

## 3. Data Classification

### 3.1 Data Categories

| Category | Description | Retention Period | Disposal Method |
|----------|-------------|------------------|-----------------|
| Customer Content | Documents, notes, knowledge base | Account lifetime + 30 days | Cryptographic erasure |
| User Identity | Email, name, profile | Account lifetime + 30 days | Database deletion |
| Authentication | Passwords, tokens, API keys | Active account lifetime | Hash invalidation |
| Audit Logs | Access logs, changes, events | 1 year | Automated purge |
| System Logs | Application logs, errors | 90 days | Log rotation |
| Temporary Files | Cache, sessions, temp data | 24 hours | Automatic cleanup |
| Backups | Database backups | 30 days | Secure overwrite |

### 3.2 Regulatory Requirements

- **SOC 2:** Data retention and disposal controls required
- **GDPR:** Right to erasure within 30 days of request
- **CCPA:** Consumer deletion requests honored
- **HIPAA:** If applicable, 6-year retention minimum

## 4. Retention Procedures

### 4.1 Customer Content

```
Storage: PostgreSQL/SQLite database
Configuration: Database connection settings in config.rs
```

**Retention Rules:**
- Content retained while account is active
- Content preserved for 30 days after account deletion
- Content permanently deleted after grace period
- Backup copies purged within 48 hours of deletion

**Implementation:**
- Soft delete with timestamp
- Automated cleanup job runs daily
- Hard delete after retention period

### 4.2 User Account Data

```
Storage: PostgreSQL/SQLite database
Configuration: Database URL in config.rs:988-989
```

**Retention Rules:**
- Account data retained while account is active
- Anonymized data may be retained for analytics
- PII permanently deleted upon request
- Authentication credentials invalidated immediately

**Deletion Process:**
1. User requests account deletion
2. Identity verification completed
3. Account marked for deletion (soft delete)
4. 30-day grace period for recovery
5. Permanent deletion executed
6. Confirmation sent to user

### 4.3 Authentication Credentials

```
Implementation: tachyon/crates/server/src/config.rs:124-170
```

**JWT Tokens:**
- Expire after configurable period (default 24 hours)
- Cannot be revoked before expiration (stateless)
- Key rotation invalidates all tokens signed with old key

**API Keys:**
- Retained until explicitly revoked
- Revoked keys immediately invalidated
- Key history not retained

**OAuth2 Tokens:**
- Provider tokens expire per provider policy
- Refresh tokens expire after configurable period
- Revocation propagated to provider

### 4.4 System Logs

```
Implementation: tachyon/crates/server/src/config.rs:364-372
Configuration: TACHYON_LOG_FORMAT, TACHYON_LOG_LEVEL
```

**Retention Rules:**
- Access logs: 1 year retention
- Application logs: 90 days retention
- Error logs: 90 days retention
- Audit logs: 1 year retention

**Log Management:**
- Structured JSON format for machine processing
- Automated rotation and compression
- Secure storage with access controls
- Encrypted at rest

### 4.5 Temporary Data

**Session Data:**
- In-memory session storage
- Cleared on server restart
- Max concurrent sessions configurable (`config.rs:308`)

**Cache Data:**
- Configurable cache size (`config.rs:34`)
- TTL-based expiration
- Cleared on deployment

**Upload Temp Files:**
- Stored in temporary directory
- Cleaned up after processing
- Maximum size limits enforced (`config.rs:301-302`)

## 5. Disposal Procedures

### 5.1 Secure Deletion Methods

| Data Type | Method | Verification |
|-----------|--------|-------------|
| Database Records | SQL DELETE with overwrite | Row count verification |
| File System | Secure file deletion | File system check |
| Backups | Cryptographic erasure | Decryption key destruction |
| Logs | Automated purge | Log rotation verification |
| Cache | Memory deallocation | Cache size monitoring |

### 5.2 Cryptographic Erasure

For encrypted data:

1. Destroy encryption keys
2. Verify data is unreadable
3. Document key destruction
4. Confirm disposal with certificate

### 5.3 Backup Disposal

- Backup retention: 30 days maximum
- Disposal via secure overwrite
- Disposal log maintained
- Verification of complete deletion

## 6. Data Subject Requests

### 6.1 Right to Access

- Provide data export within 30 days
- Format: Machine-readable (JSON, CSV)
- Include all user-generated content
- Exclude other users' data

### 6.2 Right to Erasure

- Process deletion within 30 days
- Verify complete deletion
- Provide confirmation to requestor
- Document deletion for compliance

### 6.3 Right to Portability

- Export in standard formats
- Include structured data
- Exclude system-generated data
- Provide via secure download

## 7. Configuration

### 7.1 Retention Configuration

Retention periods can be configured via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `TACHYON_LOG_RETENTION_DAYS` | 90 | Application log retention |
| `TACHYON_AUDIT_RETENTION_DAYS` | 365 | Audit log retention |
| `TACHYON_BACKUP_RETENTION_DAYS` | 30 | Backup retention |
| `TACHYON_SESSION_EXPIRY_HOURS` | 24 | Session expiration |

### 7.2 Cleanup Schedules

- Daily: Temporary files, expired sessions
- Weekly: Log rotation, cache cleanup
- Monthly: Audit log archiving
- Quarterly: Backup verification and disposal

## 8. Compliance Evidence

| Control | Evidence |
|---------|----------|
| Retention Periods | Policy documentation, configuration |
| Disposal Procedures | Deletion logs, verification records |
| Data Subject Requests | Request handling logs, confirmation |
| Backup Management | Backup inventory, disposal logs |
| Audit Trail | Log retention, rotation records |

## 9. Related Documents

- [Security Policy](security-policy.md)
- [Access Control Procedures](access-control.md)
- [Incident Response Plan](incident-response.md)
