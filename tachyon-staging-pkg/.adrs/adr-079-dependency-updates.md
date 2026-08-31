# ADR-079: Dependency Updates

**Status:** Accepted
**Date:** 2026-02-12
**Decision Type:** Technical
**Context:** Phase 8.5 - Supply Chain Monitoring

---

## Context and Problem Statement

### Context
The Tachyon project has dependencies across multiple ecosystems (Rust/Cargo, Node.js/npm) that require regular updates to:
1. Address security vulnerabilities
2. Incorporate new features and bug fixes
3. Maintain compatibility with ecosystem updates
4. Ensure license compliance
5. Prevent dependency rot

The vulnerability report ([`.adrs/ identifies a response SLA of <24 hours for Critical vulnerabilities, requiring automated update mechanisms.

### Problem Statement
We lack an automated dependency update strategy that provides:
1. Timely detection of available updates
2. Automated PR creation for security patches
3. Automated testing of updates
4. Controlled rollout of updates
5. Rollback capability for problematic updates

Current challenges:
- Manual dependency updates are slow and error-prone
- No automated testing of updated dependencies
- Risk of breaking changes going unnoticed
- Difficult to track dependency freshness
- No standardized update process

---

## Decision Drivers

### Security Requirements
From [`.adrs/
- **NIST-SI-07:** Software and Information Integrity
- **OWASP-A06:** Vulnerable and Outdated Components

### Business Requirements
- **Response SLA:** <24 hours for Critical vulnerability patches
- **Breaking Change Detection:** Must identify API changes before merge
- **Automated Testing:** All updates must pass test suite
- **Rollback Capability:** Must be able to revert problematic updates

### Operational Constraints
- **Developer Availability:** Minimize manual review overhead
- **CI/CD Integration:** Automated where possible
- **False Positive Rate:** <5% for automated updates
- **Review Turnaround:** <7 days for non-critical updates

---

## Considered Alternatives

### Alternative 1: Dependabot-Only Updates
**Description:** Rely solely on GitHub Dependabot for dependency updates

**Pros:**
- Free service
- Native GitHub integration
- Automated PR creation
- Good coverage for popular packages
- Minimal configuration

**Cons:**
- Limited update categorization (no security vs feature)
- No custom update policies
- Limited to GitHub-hosted packages
- No support for private packages
- No container image updates
- Inconsistent coverage across ecosystems

**Rejection:** Insufficient coverage and control

### Alternative 2: SaaS-Based Updates (Renovate, Snyk)
**Description:** Use commercial SaaS platforms for automated dependency updates

**Pros:**
- Comprehensive coverage across ecosystems
- Rich UI for update management
- Automated security patch PRs
- Grouped updates for efficiency
- Advanced scheduling and batch PRs
- Automated testing integration

**Cons:**
- Recurring licensing costs ($5K-$20K/year)
- Limited control over update rules
- Dependency on external service
- Learning curve for team
- Potential vendor lock-in

**Rejection:** Cost-prohibitive for current scale

### Alternative 3: Manual Updates Only
**Description:** Continue with manual, ad-hoc dependency updates

**Pros:**
- Zero additional cost
- Full control over update process
- Can use any tools
- No vendor lock-in

**Cons:**
- High human error and omission risk
- Slow update cycle
- Inconsistent coverage
- High developer overhead
- Difficult to track compliance

**Rejection:** Does not meet security response SLA

### Alternative 4: Multi-Tool Automated Updates (Chosen)
**Description:** Combine multiple tools for comprehensive automated dependency updates

**Pros:**
- Zero licensing cost for open-source tools
- Full control over update policies
- Comprehensive coverage (Rust, NPM, Docker)
- Native CI/CD integration
- Custom update rules and categorization
- Automated testing and validation
- Support for private packages

**Cons:**
- Requires tool installation and configuration
- Multiple tools to maintain and update
- Complex workflow orchestration
- Requires script development

**Acceptance:** Best balance of automation, control, and cost

---

## Decision

### Chosen Approach: Multi-Tool Automated Updates

We will implement automated dependency updates using:

**Rust/Cargo:**
- **cargo-outdated:** Dependency freshness detection
- **Dependabot:** Automated PR creation for GitHub-hosted packages

**Node.js/NPM:**
- **npm outdated:** Dependency freshness detection
- **Dependabot:** Automated PR creation for GitHub-hosted packages

**Custom Automation:**
- Python scripts for update policy enforcement
- Automated PR creation for private packages
- Update categorization (Security, Feature, Patch)
- Automated testing integration

### Update Architecture

```
┌───────────────────────────────────────────────────────────────────────────┐
│                    Dependency Update System                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Detection Layer                                                   │
│  - Freshness Scanning (cargo-outdated, npm outdated)               │
│  - Update Availability Detection                                      │
│  - Security Advisory Monitoring                                        │
│  - Breaking Change Analysis                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Policy Layer                                                      │
│  - Update Categorization (Security, Feature, Patch, Deprecated)         │
│  - Version Pinning Rules                                         │
│  - License Compliance Checks                                        │
│  - Review Requirements (Security: Auto, Feature: Manual)          │
├─────────────────────────────────────────────────────────────────────────────┤
│  Automation Layer                                                    │
│  - PR Generation (Dependabot, Custom Scripts)                     │
│  - Automated Testing (CI/CD)                                     │
│  - Dependency Pinning                                              │
│  - Rollback Automation                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  Reporting Layer                                                    │
│  - Update Dashboard                                                 │
│  - Dependency Health Metrics                                         │
│  - Update Success Metrics                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### Update Categories

**Security Updates (Priority 1 - Automatic):**
- Trigger: Critical or High severity vulnerability detected
- Action: Automatic PR creation
- Version strategy: Patch version only
- Testing: Full test suite required
- Review: Required within 24 hours
- Deployment: Automatic after approval

**Feature Updates (Priority 2 - Automated):**
- Trigger: Minor version with important features
- Action: Automated PR creation with label "feature-update"
- Version strategy: Allow minor version bump
- Testing: Full test suite required
- Review: Required within 7 days
- Deployment: Manual after approval

**Patch Updates (Priority 3 - Batched):**
- Trigger: Patch version available
- Action: Weekly batched PR creation
- Version strategy: Allow patch updates
- Testing: Automated test suite
- Review: Required within 14 days
- Deployment: Manual after approval

**Development Dependencies (Priority 4 - Monthly):**
- Trigger: Monthly schedule
- Action: Separate PR for dev dependencies
- Version strategy: Allow patch and minor updates
- Testing: Relevant dev tests only
- Review: Required within 30 days
- Deployment: No deployment impact

### Version Pinning Strategy

**Production Dependencies:**
```toml
# Cargo.toml - Use exact versions for critical deps
[dependencies]
tokio = { version = "1.49.0", features = ["full"] }
serde = { version = "1.0.218", default-features = false }
```

**Development Dependencies:**
```toml
# Cargo.toml - Use caret ranges for non-critical deps
[dev-dependencies]
criterion = "0.5.0"
proptest = "1.0.0"
```

### Automated Testing

**Pre-Merge Testing:**
```bash
#!/bin/bash
# test-dependency-update.sh

DEPENDENCY="${1:-tokio}"
NEW_VERSION="${2:-1.50.0}"

echo "Testing ${DEPENDENCY} update to ${NEW_VERSION}"

# Update dependency
cargo update ${DEPENDENCY} --precise ${NEW_VERSION}

# Run full test suite
cargo test --workspace

# Run integration tests
cargo test --workspace --test '*'

# Check for breaking changes
cargo check --workspace
```

**CI/CD Integration:**
```yaml
# .github/workflows/dependency-update.yml
name: Dependency Update Test
on:
  pull_request:
    types: [opened, synchronize]

jobs:
  test-update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: |
          cargo test --workspace
          cargo clippy -- -D warnings
```

### Update Policy Configuration

**Security Updates:**
```yaml
# config/dependency-updates.yml
security_updates:
  auto_create_pr: true
  version_strategy: "patch_only"
  require_tests: true
  auto_merge: false
  review_deadline_hours: 24
```

**Feature Updates:**
```yaml
feature_updates:
  auto_create_pr: true
  version_strategy: "minor_allowed"
  require_tests: true
  auto_merge: false
  review_deadline_hours: 168
  pr_label: "feature-update"
```

---

## Rollback Strategy

### Triggers for Rollback
- Security vulnerability in new version
- Breaking API changes detected
- Performance regression detected
- License violation in new version
- Test suite failures

### Rollback Process
1. **Detection:** Monitor for issues within 24 hours of deployment
2. **Analysis:** Determine if rollback is necessary
3. **Rollback:** Revert to previous stable version
4. **Investigation:** Root cause analysis of problematic update
5. **Remediation:** Address issue before attempting update again
6. **Documentation:** Record rollback details in incident log

### Rollback Automation
```bash
#!/bin/bash
# rollback-dependency.sh

DEPENDENCY="${1}"
PREVIOUS_VERSION="${2}"

echo "Rolling back ${DEPENDENCY} to ${PREVIOUS_VERSION}"

# Revert to previous version
cargo update ${DEPENDENCY} --precise ${PREVIOUS_VERSION}

# Verify build
cargo build --release

# Run tests
cargo test --workspace

echo "Rollback complete. Build verified."
```

---

## Consequences

### Positive Consequences
1. **Improved Security:** Automated security patching reduces vulnerability exposure
2. **Reduced Maintenance:** Automated updates reduce manual overhead
3. **Better Coverage:** Multi-tool approach covers all dependency types
4. **Controlled Rollout:** Automated testing prevents breaking changes
5. **Compliance:** Meets NIST SP 800-53 SI-07 requirements
6. **Cost Efficiency:** Open-source tools avoid SaaS licensing costs
7. **Dependency Freshness:** Continuous monitoring prevents dependency rot

### Negative Consequences
1. **Automation Overhead:** Initial setup and ongoing maintenance
2. **Complexity:** Multiple tools and policies to manage
3. **False Positives:** Automated PRs may require manual review
4. **Breaking Changes:** Even with testing, some updates may break compatibility
5. **Review Burden:** Automated PRs create review workload

---

## Compliance and Standards Alignment

### NIST SP 800-53 Controls
- **SI-07:** Software and Information Integrity - Addressed by automated updates
- **SA-12:** Supply Chain Protection - Enhanced by update monitoring

### OWASP Top 10
- **A06:** Vulnerable and Outdated Components - Addressed by automated patching
- **A08:** Software and Data Integrity Failures - Addressed by testing

### Executive Order 14028 Requirements
- **Automated Testing:** CI/CD integration for update testing
- **SBOM Generation:** SBOM updated with each dependency change

---

## Related Decisions

- **ADR-077:** Supply Chain Monitoring ([`.adrs/adr-077-supply-chain-monitoring.md`](.adrs/adr-077-supply-chain-monitoring.md))
- **ADR-078:** Vulnerability Scanning ([`.adrs/adr-078-vulnerability-scanning.md`](.adrs/adr-078-vulnerability-scanning.md))

---

## References

**Internal Documents:**
- [`.adrs/ - Monitoring Strategy
- [`.adrs/ - Alerting Rules
- [`.adrs/ - Vulnerability Report
- [`.adrs/ - Compliance Matrix
- [`.adrs/ - CI/CD Pipeline

**External Tools:**
- Dependabot: https://docs.github.com/en/dependabot
- cargo-outdated: https://github.com/kbknapp/cargo-outdated
- npm outdated: https://docs.npmjs.com/cli/v6/commands/npm-outdated

**External Standards:**
- NIST SP 800-53: Security and Privacy Controls
- Executive Order 14028: Improving Software Supply Chain Security
- OWASP Top 10: https://owasp.org/Top10

---

## Approval

**Approved By:** DevOps Team Lead
**Approval Date:** 2026-02-12
**Reviewers:** DevOps Team, Security Team
**Implementation Status:** Approved for Implementation
