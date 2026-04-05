# ADR-020: FIPS 140-2/3 Cryptography

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Security Engineering Phase

---

## 1. Context and Problem Statement

### 1.1. Context

FIPS 140-2 and FIPS 140-3 are U.S. government standards for cryptographic modules. While full FIPS validation requires a formal certification process, Tachyon should implement cryptographic controls compliant with FIPS requirements to ensure regulatory acceptance and cryptographic best practices.

### 1.2. Problem Statement

Tachyon uses cryptography for multiple purposes including data encryption, secure communication, and authentication. A systematic approach to FIPS-compliant cryptography is required to ensure all cryptographic operations meet FIPS 140-2/3 standards.

---

## 2. FIPS 140-2/3 Requirements Analysis

### 2.1. Cryptographic Module Requirements

| Requirement | Description | Tachyon Implementation | Status |
|--------------|-------------|------------------------|--------|
| FIPS-140-AES | AES encryption | AES-256-GCM for data at rest | Not yet implemented |
| FIPS-140-RSA | RSA encryption | RSA-2048/3072 for key exchange | Not yet implemented |
| FIPS-140-SHA | SHA hashing | SHA-256/384 for hash verification | Not yet implemented |
| FIPS-140-HMAC | HMAC for authentication | HMAC-SHA256 for integrity | Not yet implemented |
| FIPS-140-DRBG | Deterministic RNG | Use CSPRNG or OS CSPRNG | Not yet implemented |
| FIPS-140-ECDH | Elliptic curve key exchange | ECDH P-384 for TLS | Not yet implemented |
| FIPS-140-ECDSA | Elliptic curve signatures | ECDSA P-384 for JWT | Not yet implemented |

### 2.2. Cryptographic Module Design

| Module | Algorithm | Key Size | Purpose | FIPS Status |
|--------|-----------|----------|---------|-------------|
| Data Encryption | AES-GCM | 256 bits | Data at rest | FIPS 197 |
| Data Integrity | HMAC-SHA256 | 256 bits | Integrity verification | FIPS 198-1 |
| Key Derivation | PBKDF2 | 256 bits | Password hashing | FIPS 198-1, SP 800-132 |
| TLS Transport | TLS 1.3 | RSA-2048/3072, ECDH P-384 | Secure communication | RFC 8446 |
| JWT Signing | ECDSA | P-384 | Token authentication | FIPS 186-4 |
| Hash Verification | SHA-256 | 256 bits | Hash verification | FIPS 180-4 |

---

## 3. Cryptographic Library Selection

### 3.1. Approved Cryptographic Libraries

| Library | FIPS Validation | Rust Support | Tachyon Usage |
|---------|----------------|--------------|--------------|
| OpenSSL | FIPS 140-2 validated | Yes (openssl crate) | Preferred for FIPS compliance |
| BoringSSL | Not FIPS validated | Yes (boring crate) | Alternative |
| libsodium | Not FIPS validated | Yes (sodium crate) | Non-critical operations |
| ring | Not FIPS validated | Yes | Non-critical operations |

### 3.2. Cryptographic Library Strategy

**Primary Library:** OpenSSL (FIPS 140-2 validated module)

**Rationale:**
- OpenSSL has FIPS 140-2 validated cryptographic module
- Widely supported and maintained
- Rust crate available (openssl crate)
- Supports all required algorithms

**Fallback for Non-Critical Operations:**
- Use ring or libsodium for non-regulated environments
- Use ring for performance-critical operations

---

## 4. Implementation Strategy

### 4.1. Phase 3.1: Core Cryptography (P1 Priority) - Week 1-2

1. **Data Encryption:**
   - Implement AES-256-GCM for sensitive data
   - Use OpenSSL FIPS module
   - Key derivation with PBKDF2-SHA256
   - IV/nonce management

2. **Data Integrity:**
   - Implement HMAC-SHA256 for integrity verification
   - Hash-based integrity for cache keys
   - Git commit hash verification

3. **TLS Configuration:**
   - TLS 1.3 only
   - FIPS-approved cipher suites
   - Certificate validation

### 4.2. Phase 3.2: Authentication and Authorization (P1 Priority) - Week 2-3

1. **JWT Implementation:**
   - ECDSA P-384 for JWT signing
   - Secure key storage
   - Token validation

2. **Password Storage:**
   - PBKDF2-SHA256 with proper salt
   - Minimum 100,000 iterations
   - Password complexity requirements

### 4.3. Phase 3.3: Random Number Generation (P2 Priority) - Week 3-4

1. **CSPRNG Implementation:**
   - Use OpenSSL RAND_bytes for cryptographic operations
   - Use OS CSPRNG for non-cryptographic operations
   - Seed management

2. **Key Management:**
   - Secure key storage
   - Key rotation policies
   - Key destruction

### 4.4. Phase 3.4: Cryptographic Module Validation (P3 Priority) - Week 5-6

1. **Module Testing:**
   - Cryptographic module self-test
   - Known answer tests
   - Algorithm validation

2. **Compliance Documentation:**
   - Document all cryptographic operations
   - Maintain cryptographic module inventory
   - Document key lifecycle management

---

## 5. FIPS Validation Path

### 5.1. Pre-Validation Activities

1. Gap Analysis: Complete assessment against FIPS 140-3 requirements
2. Implementation: Implement all FIPS-compliant algorithms
3. Testing: Complete cryptographic module testing
4. Documentation: Prepare validation documentation

### 5.2. Validation Process

1. Select FIPS-accredited laboratory
2. Submit cryptographic module for testing
3. Address any findings
4. Obtain FIPS 140-3 certificate

---

## 6. Testing and Verification

### 6.1. Cryptographic Testing

| Test Type | Purpose | Coverage Goal |
|-----------|---------|-------------------|
| Algorithm Testing | Verify correct implementation | All FIPS-approved algorithms |
| Key Generation Testing | Verify secure key generation | All key types and sizes |
| Encryption Testing | Verify correct encryption/decryption | All encryption operations |
| Integrity Testing | Verify integrity verification | All integrity operations |
| Performance Testing | Verify performance meets requirements | All cryptographic operations |

### 6.2. Compliance Verification

| Verification Type | Purpose | Coverage Goal |
|-------------------|---------|-------------------|
| FIPS Module Check | Verify FIPS module is active | All cryptographic operations |
| Algorithm Validation | Verify only approved algorithms used | All cryptographic operations |
| Key Management Verification | Verify secure key lifecycle | All key operations |
| Self-Test Verification | Verify module self-tests pass | All cryptographic modules |

---

## 7. Security Considerations

### 7.1. Side-Channel Attacks

**Mitigation Strategies:**
- Use constant-time implementations where possible
- Avoid timing-based attack vectors
- Implement cache timing attack mitigations

### 7.2. Key Management

**Key Storage:**
- Keys stored encrypted at rest
- Keys never exposed in logs or error messages
- Keys rotated regularly

**Key Destruction:**
- Secure memory zeroing after use
- Secure key deletion from storage
- Key destruction audit logging

---

## 8. Status

**Status:** ACCEPTED
**Implementation:**
- FIPS 140-2/3 requirements analysis complete
- Cryptographic library strategy defined (OpenSSL FIPS module)
- Implementation timeline defined for all cryptographic operations

**Next Steps:**
1. Execute core cryptography implementation (Week 1-2)
2. Execute authentication and authorization implementation (Week 2-3)
3. Execute random number generation implementation (Week 3-4)
4. Execute cryptographic module validation (Week 5-6)
5. Conduct gap analysis and prepare validation documentation
6. Select FIPS-accredited laboratory and initiate validation process

---

## 9. References

- Tachyon Requirements: [`.specs/00_requirements/requirements.md`](.specs/00_requirements/requirements.md)
- Tachyon Architecture: [`.specs/02_architecture/blue_paper.md`](.specs/02_architecture/blue_paper.md)
- Threat Model: [`.specs/03_security/threat_model.md`](.specs/03_security/threat_model.md)
- FIPS 140-2 Standard: https://csrc.nist.gov/publications/detail/fips/140/2/final
- FIPS 140-3 Standard: https://csrc.nist.gov/publications/detail/fips/140/3/final
- NIST SP 800-53 ADR: [`.adrs/adr-017-nist-800-53-controls.md`](.adrs/adr-017-nist-800-53-controls.md)
- ISO 27001 ADR: [`.adrs/adr-018-iso-27001-compliance.md`](.adrs/adr-018-iso-27001-compliance.md)
