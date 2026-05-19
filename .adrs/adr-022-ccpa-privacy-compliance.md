# ADR-022: CCPA Privacy Compliance

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Security Engineering Phase

---

## 1. Context and Problem Statement

### 1.1. Context

The California Consumer Privacy Act (CCPA) is a state statute intended to enhance privacy rights and consumer protection for residents of California, USA. Tachyon, as a knowledge management system that may process personal data of California residents, must comply with CCPA requirements.

### 1.2. Problem Statement

CCPA defines rights for California residents regarding their personal information, including the right to know what data is collected, right to delete data, right to opt-out of data sales, and right to non-discrimination. Tachyon must implement appropriate technical and organizational measures to ensure CCPA compliance.

---

## 2. CCPA Rights Analysis

### 2.1. Key CCPA Rights

| Right | Description | Tachyon Requirements | Implementation Status |
|-------|-------------|--------------------|----------------------|
| Right to Know (Section 1798.100) | Consumers have right to know what personal information is collected, used, shared, or sold | Not yet implemented | P1 |
| Right to Delete (Section 1798.105) | Consumers have right to request deletion of personal information | Not yet implemented | P1 |
| Right to Opt-Out (Section 1798.120) | Consumers have right to opt-out of data sales | Not yet implemented | P1 |
| Right to Non-Discrimination (Section 1798.125) | Consumers cannot be discriminated against for exercising rights | Not yet implemented | P2 |
| Right to Access (Section 1798.130) | Consumers have right to access their personal information | Not yet implemented | P1 |
| Right to Correct (Section 1798.130) | Consumers have right to correct inaccurate personal information | Not yet implemented | P1 |

---

## 3. Implementation Strategy

### 3.1. Phase 3.1: Privacy Policy and Notice (P1 Priority) - Week 1-2

1. **Privacy Policy:**
   - Implement comprehensive privacy policy
   - Include all required disclosures
   - Update policy as needed

2. **Notice at Collection:**
   - Implement notice at data collection points
   - Provide clear information about data collection
   - Allow consumers to make informed choices

3. **Do Not Sell My Information Link:**
   - Implement "Do Not Sell My Personal Information" link
   - Provide easy opt-out mechanism
   - Maintain opt-out requests

### 3.2. Phase 3.2: Consumer Rights (P1 Priority) - Week 2-3

1. **Right to Know (Section 1798.100):**
   - Implement data access functionality
   - Provide information about categories of personal information collected
   - Provide information about business purposes
   - Provide information about third-party sharing

2. **Right to Delete (Section 1798.105):**
   - Implement data deletion functionality
   - Verify requestor identity
   - Ensure deletion propagates to all systems
   - Maintain audit trail

3. **Right to Opt-Out (Section 1798.120):**
   - Implement opt-out mechanism
   - Verify requestor identity
   - Maintain opt-out requests
   - Honor opt-out preferences

### 3.3. Phase 3.3: Additional Rights (P2 Priority) - Week 3-4

1. **Right to Access (Section 1798.130):**
   - Implement data access functionality
   - Provide copies of personal information
   - Maintain access logs

2. **Right to Correct (Section 1798.130):**
   - Implement data correction functionality
   - Verify accuracy of corrections
   - Maintain correction logs

3. **Right to Non-Discrimination (Section 1798.125):**
   - Ensure no discrimination for exercising rights
   - Maintain equal service delivery
   - Document non-discrimination practices

### 3.4. Phase 3.4: Compliance and Verification (P2 Priority) - Week 4-5

1. **Verification of Requests:**
   - Implement identity verification
   - Verify requestor authorization
   - Maintain request documentation

2. **Response Timeframes:**
   - Implement response tracking
   - Respond within 45 days (extendable by 45 days)
   - Maintain response documentation

3. **Data Mapping:**
   - Implement comprehensive data mapping
   - Track data flow throughout system
   - Maintain data inventory

---

## 4. Technical Requirements

### 4.1. Data Categories

| Category | Description | Implementation Status |
|----------|-------------|----------------------|
| Identifiers | Name, postal address, email, phone number | Not yet implemented |
| Personal Information | Social security number, driver license number | Not yet implemented |
| Protected Characteristics | Age, race, religion | Not yet implemented |
| Commercial Information | Transaction history, spending habits | Not yet implemented |
| Biometric Information | Fingerprints, facial recognition | Not yet implemented |
| Internet Activity | Browsing history, search history | Not yet implemented |
| Geolocation | Precise location data | Not yet implemented |
| Audio/Electronic Information | Voice recordings, facial images | Not yet implemented |
| Professional Information | Employment history | Not yet implemented |
| Inferences | Profile characteristics, preferences | Not yet implemented |

### 4.2. Data Processing

| Processing Type | Description | Implementation Status |
|----------------|-------------|----------------------|
| Collection | Collection of personal information | Not yet implemented |
| Use | Use of personal information | Not yet implemented |
| Sharing | Sharing with third parties | Not yet implemented |
| Selling | Selling personal information | Not yet implemented |

---

## 5. Documentation Requirements

### 5.1. Privacy Policy

| Requirement | Content | Status |
|-------------|---------|--------|
| Categories of personal information collected | All categories | Not yet implemented |
| Purposes for collection | All purposes | Not yet implemented |
| Third parties sharing | All third parties | Not yet implemented |
| Data sales | Whether data is sold | Not yet implemented |
| Consumer rights | All consumer rights | Not yet implemented |

### 5.2. Request Documentation

| Requirement | Content | Status |
|-------------|---------|--------|
| Right to Know requests | All requests | Not yet implemented |
| Right to Delete requests | All requests | Not yet implemented |
| Right to Opt-Out requests | All requests | Not yet implemented |
| Response documentation | All responses | Not yet implemented |

---

## 6. Testing and Verification

### 6.1. Compliance Testing

| Test Type | Purpose | Coverage Goal |
|-----------|---------|-------------------|
| Right to Know Testing | Verify right to know works correctly | 100% of requests |
| Right to Delete Testing | Verify right to delete works correctly | 100% of requests |
| Right to Opt-Out Testing | Verify right to opt-out works correctly | 100% of requests |
| Non-Discrimination Testing | Verify no discrimination | All scenarios |

### 6.2. Privacy Policy Review

| Review Type | Purpose | Coverage Goal |
|-------------|---------|-------------------|
| Policy Content Review | Verify policy completeness | 100% of requirements |
| Policy Update Review | Verify policy is updated | All changes |
| Policy Accessibility Review | Verify policy is accessible | All platforms |

---

## 7. Status

**Status:** ACCEPTED
**Implementation:**
- CCPA rights analysis complete with all key rights identified
- Implementation timeline defined for all CCPA requirements
- Privacy policy and notice strategy defined

**Next Steps:**
1. Execute privacy policy and notice implementation (Week 1-2)
2. Execute consumer rights implementation (Week 2-3)
3. Execute additional rights implementation (Week 3-4)
4. Execute compliance and verification implementation (Week 4-5)
5. Conduct compliance audit and privacy policy review
6. Prepare documentation and training materials

---

## 8. References

- Tachyon Requirements: [`.adrs/
- Tachyon Architecture: [`.adrs/
- Threat Model: [`.adrs/
- CCPA Text: https://leginfo.legislature.ca.gov/faces/billTextClient.xhtml?bill_id=2019-2020ABCCPA
- GDPR ADR: [`.adrs/adr-021-gdpr-data-protection.md`](.adrs/adr-021-gdpr-data-protection.md)
- NIST SP 800-53 ADR: [`.adrs/adr-017-nist-800-53-controls.md`](.adrs/adr-017-nist-800-53-controls.md)
