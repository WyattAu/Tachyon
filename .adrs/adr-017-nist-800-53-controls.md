# ADR-017: NIST SP 800-53 Controls

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Security Engineering Phase

---

## 1. Context and Problem Statement

### 1.1. Context

NIST SP 800-53 Revision 5 provides a comprehensive catalog of security and privacy controls. Tachyon, as a knowledge management system handling sensitive user data and providing collaboration features, must implement the relevant controls to achieve compliance with this standard.

### 1.2. Problem Statement

The security requirements in Tachyon map to several NIST SP 800-53 control families. A systematic mapping is required to ensure each control is properly implemented with appropriate technical measures and audit capabilities.

---

## 2. NIST Control Family Mapping

| NIST Family | Tachyon Requirements | Implementation Status | Priority |
|--------------|--------------------|-------------|-----------|
| AC: Access Control | AC-RQ-001 through AC-RQ-006 | Not yet implemented | P1 |
| AU: Audit and Accountability | SC-RQ-008 | Not yet implemented | P2 |
| SC: System and Communications Protection | SC-RQ-001, SC-RQ-002, SC-RQ-003, SC-RQ-004 | Not yet implemented | P3 |
| SI: System and Information Integrity | IF-DB-001, CM-RQ-005 | Not yet implemented | P3 |
| CP: Contingency Planning | All high-severity threats have response plans | Not yet implemented | P3 |

---

## 3. Control Implementation

### 3.1. AC: Access Control (P1 Priority)

| Control ID | NIST Control | Implementation Strategy | Test Cases | Traceability |
|------------|--------------|----------------------------|--------------|
| NIST-AC-02 | AC-RQ-001: RBAC middleware | Implement role-based access control with deny-by-default | PT-PEN-004, PT-PEN-005 | AC-RQ-001 |
| NIST-AC-03 | AC-RQ-003: MFA | Implement multi-factor authentication for sensitive operations | PT-PEN-001 | AC-RQ-003 |
| NIST-AC-04 | AC-RQ-004: Secure session management | Implement JWT token storage, HttpOnly cookies, rotation | PT-PEN-003 | AC-RQ-004 |
| NIST-AC-07 | AC-RQ-005: Password policy | Implement minimum complexity, expiration, lockout | PT-PEN-001 | AC-RQ-005 |
| NIST-AC-08 | AC-RQ-006: Rate limiting | Rate limit login and authentication attempts | PT-PEN-001 | AC-RQ-006 |

### 3.2. AU: Audit and Accountability (P2 Priority)

| Control ID | NIST Control | Implementation Strategy | Test Cases | Traceability |
|------------|--------------|----------------------------|--------------|
| NIST-AU-002 | SC-RQ-008: Comprehensive audit logging | Implement security event logging with timestamps, user ID, action, outcome | ST-SAST-001, IF-SEC-001 | SC-RQ-008 |
| NIST-AU-003 | Audit trail integrity | Maintain immutable audit logs with tamper-evident protection | ST-SAST-001 | SC-RQ-008 |

### 3.3. SC: System and Communications Protection (P3 Priority)

| Control ID | NIST Control | Implementation Strategy | Test Cases | Traceability |
|------------|--------------|----------------------------|--------------|
| NIST-SC-001 | SC-RQ-001: Input validation | Implement strict input validation, sanitization | ST-SAST-001, PT-PEN-002 | SC-RQ-001 |
| NIST-SC-008 | SC-RQ-003: XSS prevention | Implement DOMPurify, Content-Security-Policy headers | ST-SAST-004, PT-PEN-006 | SC-RQ-003 |
| NIST-SC-008 | SC-RQ-004: CSRF protection | Implement CSRF tokens, SameSite cookies | ST-SAST-004, PT-PEN-012 | SC-RQ-004 |
| NIST-SC-002 | SC-RQ-002: Encryption at rest | Implement AES-256-GCM for sensitive data | ST-SAST-005, IF-DB-001 | SC-RQ-002 |
| NIST-SC-003 | SC-RQ-005: Output encoding | Implement DOMPurify for HTML output | ST-SAST-003 | SC-RQ-005 |

### 3.4. SI: System and Information Integrity (P3 Priority)

| Control ID | NIST Control | Implementation Strategy | Test Cases | Traceability |
|------------|--------------|----------------------------|--------------|
| NIST-SI-001 | IF-DB-001: Database integrity | Implement hash verification, audit trails | ST-SAST-005, IF-SEC-001 | IF-DB-001 |
| NIST-SI-007 | CM-RQ-005: Git integrity | Implement digital signatures for Git commits | ST-AUDIT-003 | CM-RQ-005 |

### 3.5. CP: Contingency Planning (P3 Priority)

| Control ID | NIST Control | Implementation Strategy | Test Cases | Traceability |
|------------|--------------|----------------------------|--------------|
| NIST-CP-002 | Incident response procedures | Document and test incident response plans | ST-AUDIT-001 | All high-severity threats |

---

## 4. Testing and Verification

### 4.1. Control Verification Testing

| Test Type | Purpose | Coverage Goal |
|-----------|---------|-------------------|
| Unit Tests | Verify each control implementation | 100% of NIST controls covered |
| Integration Tests | Verify controls work across components | All security requirements |
| Penetration Tests | Simulate attack scenarios | All OWASP Top 10 vulnerabilities |

### 4.2. Audit Logging Verification

| Metric | Target | Collection Method |
|---------|--------|-------------------|
| Security event logs | Detect all security events | Application logs |
| Audit completeness | All security events logged | ST-AUDIT-001 |
| Audit integrity | Immutable logs with tamper-evidence | NIST-AU-003 |

---

## 5. Implementation Timeline

### 5.1. Phase 3.1: Critical Controls (P1 Priority) - Week 1-2

1. Implement AC-RQ-001 RBAC middleware
2. Implement AC-RQ-003 MFA for sensitive operations
3. Implement AC-RQ-004 Secure session management
4. Implement AC-RQ-005 Password policy
5. Implement AC-RQ-006 Rate limiting

### 5.2. Phase 3.2: System and Communications (P2 Priority) - Week 3-4

1. Implement SC-RQ-001 Input validation
2. Implement SC-RQ-003 XSS prevention
3. Implement SC-RQ-002 Encryption at rest
4. Implement SC-RQ-004 CSRF protection
5. Implement SC-RQ-005 Output encoding

### 5.3. Phase 3.3: System and Information Integrity (P3 Priority) - Week 5-6

1. Implement IF-DB-001 Database integrity checks
2. Implement CM-RQ-005 Git commit verification

### 5.4. Phase 3.4: Audit and Logging (P3 Priority) - Week 7-8

1. Implement NIST-AU-002 Comprehensive audit logging
2. Set up security metrics and alerting

---

## 6. Status

**Status:** ACCEPTED
**Implementation:**
- NIST control family mapping complete with 16 controls identified
- Implementation timeline defined for all controls with priorities
- Testing and verification procedures defined

**Next Steps:**
1. Execute P1 implementation (Week 1-2)
2. Execute P2 implementation (Week 3-4)
3. Execute P3 implementation (Week 5-6)
4. Execute P4 implementation (Week 7-8)
5. Execute security test plan and verify control coverage

---

## 7. References

- Tachyon Requirements: [`.specs/00_requirements/requirements.md`](.specs/00_requirements/requirements.md)
- Tachyon Architecture: [`.specs/02_architecture/blue_paper.md`](.specs/02_architecture/blue_paper.md)
- Threat Model: [`.specs/03_security/threat_model.md`](.specs/03_security/threat_model.md)
- NIST SP 800-53 Revision 5: https://csrc.nist.gov/publications/detail/sp800-53/rev5/
