# ADR-054: SBOM Automation

**Status:** ACCEPTED
**Date:** 2026-02-11
**Context:** Phase 6 - CI/CD Engineering

## Context

This ADR documents the SBOM automation decision for the Tachyon project, which implements automated Software Bill of Materials generation for all components on every build.

## Decision

We have chosen to implement automated SBOM generation with the following characteristics:

1. SBOM generation on every build for all components (Rust, Node.js, Docker)
2. Multiple SBOM formats (SPDX 2.3, CycloneDX 1.5)
3. Automated SBOM verification and validation
4. SBOM signing for integrity
5. SBOM upload to repository and release artifacts

## Drivers

### Compliance
- SPDX 2.3 compliance with NIST SP 800-161 and Executive Order 14028
- CycloneDX 1.5 for vulnerability management
- License compliance verification

### Supply Chain Transparency
- Complete dependency visibility for all components
- Automated vulnerability detection through SBOM scanning

### Automation
- SBOM generated automatically in CI/CD pipeline
- No manual SBOM generation required
- Consistent SBOM format across components

## Alternatives Considered

### Alternative 1: Manual SBOM Generation
SBOMs generated manually by developers.

**Pros:**
- No automation overhead
- Full control over SBOM content

**Cons:**
- Time-consuming for developers
- Inconsistent SBOM generation
- Risk of missing or outdated SBOMs
- Not generated on every build

**Rejected:** Automation required for compliance.

### Alternative 2: Single SBOM for All Components
Single combined SBOM for all project components.

**Pros:**
- Single SBOM to manage
- Simpler release process

**Cons:**
- Loss of component-level granularity
- Difficult to track component-specific issues
- Less useful for security scanning

**Rejected:** Need component-level SBOMs for effective vulnerability management.

### Alternative 3: Vendor-Managed SBOM Tools
Using external SBOM generation services.

**Pros:**
- No tool maintenance
- Quick setup time
- Professional tooling

**Cons:**
- Vendor lock-in
- Data privacy concerns
- Limited customization options
- Potential cost implications

**Rejected:** Need full control over SBOM data and generation process.

## Consequences

### Positive Consequences
- Complete supply chain transparency
- Automated compliance verification
- Consistent SBOM generation across all builds
- Enhanced security through vulnerability detection

### Negative Consequences
- Increased build time for SBOM generation
- Additional infrastructure complexity for SBOM tools
- Learning curve for team on SBOM management

## Implementation Notes

- SBOM automation documented in .specs/07_ci_cd/sbom_automation.md
- GitHub Actions workflow in .github/workflows/sbom_generation.yml
- SBOM tools: cargo-bom, cyclonedx-npm, syft

## References

- .specs/07_ci_cd/sbom_automation.md
- .specs/01_5_supply_chain/sbom.spdx
- .specs/01_5_supply_chain/license_compliance.md
- NIST SP 800-161: https://nvlpubs.nist.gov/nistpubs/Specialpublications/NIST.SP.800-161/
- Executive Order 14028: https://www.whitehouse.gov/briefings-and-statements/presidential-actions-on-improving-the-nations-cybersecurity/

---

**Approval:**

| Role | Name | Date |
|------|------|------|
| DevOps Lead | TBD | TBD |
| Security Lead | TBD | TBD |
| Compliance Lead | TBD | TBD |
