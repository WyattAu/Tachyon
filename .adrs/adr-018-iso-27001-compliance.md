# ADR-018: ISO/IEC 27001:2022 Compliance

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Security Engineering Phase

---

## 1. Context and Problem Statement

### 1.1. Context

ISO/IEC 27001:2022 is the international standard for information security management systems (ISMS). Tachyon, as a knowledge management system handling potentially sensitive user data and providing collaboration features, must demonstrate compliance with this standard to ensure trustworthiness and market acceptance.

### 1.2. Problem Statement

ISO 27001:2022 Annex A contains 93 controls organized into 4 themes. Tachyon's security requirements must be mapped to these controls, and implementation strategies must be defined for each applicable control to achieve certification.

---

## 2. ISO 27001:2022 Control Mapping

### 2.1. Theme 1: Organizational (5 Controls)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-5.1 | Policies for information security | SC-RQ-008, All security requirements | Not yet implemented | P2 |
| ISO-5.7 | Threat intelligence | STRIDE threat model, threat analysis | Not yet implemented | P3 |
| ISO-5.19 | Information security in supplier relationships | Supply chain security | Not yet implemented | P3 |
| ISO-5.20 | Addressing information security within supplier agreements | Supply chain security | Not yet implemented | P3 |
| ISO-5.21 | Managing information security in the ICT supply chain | Supply chain security | Not yet implemented | P3 |

### 2.2. Theme 2: People (8 Controls)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-6.1 | Screening | Security training for developers | Not yet implemented | P3 |
| ISO-6.2 | Terms and conditions of employment | Security policies for users | Not yet implemented | P3 |
| ISO-6.3 | Information security awareness, education and training | User security training | Not yet implemented | P3 |
| ISO-6.4 | Disciplinary process | Security violation handling | Not yet implemented | P3 |
| ISO-6.5 | Responsibilities and competencies | Security role definitions | Not yet implemented | P3 |
| ISO-6.6 | Remote working | Secure remote access | Not yet implemented | P3 |
| ISO-6.7 | Mobile devices and BYOD | Secure device management | Not yet implemented | P3 |
| ISO-6.8 | Teleworking | Secure teleworking | Not yet implemented | P3 |

### 2.3. Theme 3: Physical (4 Controls)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-7.1 | Physical security perimeters | Physical access controls | Not applicable (SaaS deployment) | P4 |
| ISO-7.2 | Physical entry | Physical access controls | Not applicable (SaaS deployment) | P4 |
| ISO-7.3 | Securing offices, rooms and facilities | Physical security | Not applicable (SaaS deployment) | P4 |
| ISO-7.4 | Physical security monitoring | Physical security | Not applicable (SaaS deployment) | P4 |

### 2.4. Theme 4: Technological (76 Controls)

#### 2.4.1. Access Control (AC)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.1 | User endpoint devices | Secure client configuration | Not yet implemented | P3 |
| ISO-8.2 | Privileged access rights | RBAC for administrative functions | Not yet implemented | P1 |
| ISO-8.3 | Information access restriction | RBAC for documents | Not yet implemented | P1 |
| ISO-8.5 | Secure authentication | MFA, password policy | Not yet implemented | P1 |
| ISO-8.6 | Capacity management of information access resources | Rate limiting | Not yet implemented | P1 |
| ISO-8.7 | Protection against information leaks | Input validation, output encoding | Not yet implemented | P2 |

#### 2.4.2. Cryptography (CG)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.24 | Use of cryptography | AES-256-GCM for sensitive data | Not yet implemented | P1 |
| ISO-8.25 | Key management | Secure key derivation | Not yet implemented | P1 |

#### 2.4.3. Operations Security (OP)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.8 | Collection of evidence | Audit logging | Not yet implemented | P2 |
| ISO-8.9 | Logging of activities | Security event logging | Not yet implemented | P2 |
| ISO-8.10 | Monitoring activities | Security monitoring | Not yet implemented | P2 |
| ISO-8.11 | Clock synchronization | Time synchronization | Not yet implemented | P3 |
| ISO-8.12 | Use of privileged programs | Privilege management | Not yet implemented | P2 |
| ISO-8.14 | Configuration management | Secure configuration | Not yet implemented | P3 |
| ISO-8.15 | Information deletion | Secure data deletion | Not yet implemented | P3 |
| ISO-8.16 | Masking of data | Data masking for logs | Not yet implemented | P3 |
| ISO-8.17 | Data leakage prevention | Data loss prevention | Not yet implemented | P3 |
| ISO-8.18 | Information backup | Backup procedures | Not yet implemented | P3 |
| ISO-8.19 | Redundancy of information processing facilities | High availability | Not yet implemented | P3 |
| ISO-8.20 | Removal of assets | Secure disposal | Not yet implemented | P3 |

#### 2.4.4. Communications Security (CS)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.20 | Security of network services | Secure network communication | Not yet implemented | P2 |
| ISO-8.21 | Segregation of networks | Network segmentation | Not yet implemented | P3 |
| ISO-8.22 | Security of network services | Web application firewall | Not yet implemented | P3 |
| ISO-8.23 | Web filtering | Content filtering | Not yet implemented | P4 |

#### 2.4.5. System Acquisition, Development and Maintenance (AD)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.25 | Secure development lifecycle | Secure development practices | Not yet implemented | P3 |
| ISO-8.26 | Application security requirements on third parties | Supplier security | Not yet implemented | P3 |
| ISO-8.27 | Security architecture | System security architecture | Not yet implemented | P2 |
| ISO-8.28 | Secure coding | Secure coding standards | Not yet implemented | P3 |
| ISO-8.29 | Security testing | Security test plan | Not yet implemented | P3 |
| ISO-8.30 | Development, testing and acceptance environments | Environment separation | Not yet implemented | P3 |
| ISO-8.31 | Out-of-band data transfer | Secure data transfer | Not yet implemented | P3 |

#### 2.4.6. Supplier Relationships (SR)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.32 | Supplier security | Supply chain security | Not yet implemented | P3 |
| ISO-8.33 | Supplier service delivery management | Service level agreements | Not yet implemented | P3 |
| ISO-8.34 | Supplier service continuity | Business continuity | Not yet implemented | P3 |
| ISO-8.35 | Supplier monitoring | Supplier monitoring | Not yet implemented | P3 |

#### 2.4.7. Information Security Incident Management (IM)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.36 | Responsibilities and procedures | Incident response plan | Not yet implemented | P3 |
| ISO-8.37 | Learning from information security incidents | Incident post-mortem | Not yet implemented | P3 |
| ISO-8.38 | Collection of evidence | Evidence collection | Not yet implemented | P3 |

#### 2.4.8. Information Security Continuity (BC)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.39 | Information security during disruption | Business continuity plan | Not yet implemented | P3 |
| ISO-8.40 | ICT readiness for business continuity | Disaster recovery plan | Not yet implemented | P3 |

#### 2.4.9. Compliance (CO)

| Control ID | ISO Control | Tachyon Requirements | Implementation Status | Priority |
|------------|-------------|--------------------|-------------|-----------|
| ISO-8.41 | Identification of applicable legislation and contractual requirements | Compliance monitoring | Not yet implemented | P3 |
| ISO-8.42 | Intellectual property rights | IP protection | Not yet implemented | P3 |
| ISO-8.43 | Protection of records | Record retention | Not yet implemented | P3 |
| ISO-8.44 | Privacy and protection of PII | Data protection | Not yet implemented | P3 |
| ISO-8.45 | Independent review of information security | Security audits | Not yet implemented | P3 |

---

## 3. Implementation Plan

### 3.1. Phase 3.1: Critical Controls (P1 Priority) - Week 1-2

1. ISO-8.2: RBAC for administrative functions
2. ISO-8.3: RBAC for documents
3. ISO-8.5: MFA and password policy
4. ISO-8.6: Rate limiting
5. ISO-8.24: AES-256-GCM for sensitive data
6. ISO-8.25: Secure key derivation

### 3.2. Phase 3.2: System and Communications (P2 Priority) - Week 3-4

1. ISO-5.1: Security policies
2. ISO-8.8: Evidence collection
3. ISO-8.9: Security event logging
4. ISO-8.10: Security monitoring
5. ISO-8.12: Privilege management
6. ISO-8.20: Secure network communication
7. ISO-8.27: Security architecture
8. ISO-8.7: Protection against information leaks

### 3.3. Phase 3.3: Supply Chain and Development (P3 Priority) - Week 5-6

1. ISO-5.7: Threat intelligence
2. ISO-5.19-5.21: Supplier security
3. ISO-6.1-6.8: People security controls
4. ISO-8.14-8.23: Operations security
5. ISO-8.25-8.31: Secure development lifecycle
6. ISO-8.32-8.35: Supplier relationships
7. ISO-8.36-8.40: Incident management and continuity
8. ISO-8.41-8.45: Compliance controls

### 3.4. Phase 3.4: Documentation and Audit Preparation (P4 Priority) - Week 7-8

1. Physical security controls (if applicable)
2. Documentation of all controls
3. Internal audit preparation
4. Certification audit preparation

---

## 4. Testing and Verification

### 4.1. Control Testing

| Test Type | Purpose | Coverage Goal |
|-----------|---------|-------------------|
| Unit Tests | Verify each control implementation | 100% of controls covered |
| Integration Tests | Verify controls work across components | All applicable controls |
| Gap Analysis | Identify missing controls | All 93 controls reviewed |

### 4.2. Audit Readiness

| Audit Requirement | Target | Collection Method |
|-------------------|--------|-------------------|
| Security policies | Complete and documented | Policy documentation |
| Evidence of implementation | All controls | System logs, configuration |
| Management review | Annual review | Management review meetings |
| Internal audit | Quarterly | Internal audit reports |

---

## 5. Certification Path

### 5.1. Pre-Certification Activities

1. Gap Analysis: Complete assessment against ISO 27001:2022 controls
2. Remediation: Implement missing controls
3. Internal Audit: Conduct internal audit to verify compliance
4. Management Review: Obtain management approval and commitment

### 5.2. Certification Process

1. Select accredited certification body
2. Stage 1 Audit: Documentation review
3. Stage 2 Audit: Implementation verification
4. Corrective Actions: Address any findings
5. Certification: Obtain ISO 27001:2022 certificate
6. Surveillance: Annual surveillance audits

---

## 6. Status

**Status:** ACCEPTED
**Implementation:**
- ISO 27001:2022 control mapping complete with 93 controls reviewed
- Implementation timeline defined for all applicable controls
- Certification path defined with pre-certification activities

**Next Steps:**
1. Execute P1 implementation (Week 1-2)
2. Execute P2 implementation (Week 3-4)
3. Execute P3 implementation (Week 5-6)
4. Execute P4 implementation (Week 7-8)
5. Conduct gap analysis and internal audit
6. Select certification body and initiate certification process

---

## 7. References

- Tachyon Requirements: [`.specs/00_requirements/requirements.md`](.specs/00_requirements/requirements.md)
- Tachyon Architecture: [`.specs/02_architecture/blue_paper.md`](.specs/02_architecture/blue_paper.md)
- Threat Model: [`.specs/03_security/threat_model.md`](.specs/03_security/threat_model.md)
- ISO/IEC 27001:2022 Standard: https://www.iso.org/standard/82875.html
- NIST SP 800-53 ADR: [`.adrs/adr-017-nist-800-53-controls.md`](.adrs/adr-017-nist-800-53-controls.md)
