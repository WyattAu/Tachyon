# ADR-067: Consistency

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 7 - Narrative & Documentation
**Related ADRs:** ADR-062 (Brand Identity), ADR-063 (UX Philosophy), ADR-064 (Documentation Strategy)

---

## Context

Tachyon has multiple documentation artifacts (specifications, user guides, API references, FAQs, etc.) across different formats and audiences. Consistency across all documentation is critical for user experience, maintainability, and brand integrity. Inconsistent documentation leads to user confusion, increased support burden, and degraded trust in the product.

## Problem Statement

Without a defined consistency strategy, Tachyon faces several risks:
- Inconsistent terminology across documents
- Varying structure and formats
- Conflicting information
- Inconsistent visual presentation
- Difficulty maintaining documentation over time

## Decision

### Consistency Framework

#### 1. Terminology Consistency

**1.1 Glossary-Based Standardization**

All documentation must use terminology defined in the glossary ([`.docs/glossary.md`](../.docs/glossary.md)).

**Core Terminology:**

| Term | Definition | Usage Notes |
|------|-------------|-------------|
| **Tachyon** | The JIT-compiled documentation platform | Always capitalized, never "tachyon" |
| **JIT** | Just-In-Time compilation | Always use full term on first mention |
| **LRU Cache** | Least-Recently-Used cache eviction | Always use full term on first mention |
| **BM25** | Probabilistic relevance ranking algorithm | Always use full term on first mention |
| **RBAC** | Role-Based Access Control | Always use full term on first mention |
| **Desktop Mode** | Native application mode using Tauri | Always capitalized |
| **Server Mode** | HTTP/2 server mode with WebSocket | Always capitalized |
| **Static Export** | CLI tool for generating static HTML sites | Always capitalized |

**1.2 Acronym and Abbreviation Policy**

**First Mention Rule:**
```markdown
# Correct
Tachyon uses JIT (Just-In-Time) compilation for sub-15ms rendering.
The LRU (Least-Recently-Used) cache eviction strategy ensures efficiency.

# Incorrect
Tachyon uses jit compilation.
The lru cache eviction strategy ensures efficiency.
```

**Acronym Dictionary:**
```json
{
  "acronyms": {
    "JIT": {
      "full": "Just-In-Time",
      "category": "performance",
      "usage": "Always expand on first mention"
    },
    "LRU": {
      "full": "Least-Recently-Used",
      "category": "caching",
      "usage": "Always expand on first mention"
    },
    "BM25": {
      "full": "BM25 ranking function",
      "category": "search",
      "usage": "Technical term, no expansion needed"
    },
    "RBAC": {
      "full": "Role-Based Access Control",
      "category": "security",
      "usage": "Always expand on first mention"
    },
    "WCAG": {
      "full": "Web Content Accessibility Guidelines",
      "category": "accessibility",
      "usage": "Always expand on first mention"
    }
  }
}
```

#### 2. Structural Consistency

**2.1 Document Header Template**

All documentation must use the standard header:

```markdown
# [Document Title]

**Document ID:** TACHYON-[TYPE]-V[VERSION]
**Date:** YYYY-MM-DD
**Version:** X.Y.Z
**Status:** [Released/Draft/Deprecated]
**Accessibility:** WCAG 2.1 AA Compliant

---

## Table of Contents

[Auto-generated or manually maintained]

---

## [Section 1]

[Content]

---

## Getting Help

[Standard help section with links to all docs]

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | YYYY-MM-DD | [Author] | [Description of changes] |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant.
```

**2.2 Section Structure**

Every section must follow this structure:

1. **Overview** (Optional): Brief introduction to the section
2. **Core Content**: Main information, organized by topic
3. **Examples**: Practical code or workflow examples
4. **Caveats** (Optional): Limitations and considerations
5. **Related** (Optional): Links to related documentation

**Example:**

```markdown
## Configuration

### Overview

Tachyon uses a configuration file (`tachyon.toml`) in the repository root to customize behavior.

### Core Content

[Configuration options organized by category]

### Examples

```toml
[server]
port = 8080
host = "0.0.0.0"

[cache]
max_entries = 1000
max_size_bytes = 104857600  # 100 MB
```

### Caveats

- Configuration changes require restarting Tachyon
- Some options are mode-specific (Desktop vs Server vs Static)

### Related

- [Configuration Guide](./configuration_guide.md)
- [Installation Guide](./installation_guide.md)
```

#### 3. Visual Consistency

**3.1 Typography**

| Element | Font | Size | Weight | Usage |
|---------|------|------|--------|--------|
| **Heading 1** | Inter | 36px | 700 | Document title |
| **Heading 2** | Inter | 28px | 600 | Section titles |
| **Heading 3** | Inter | 24px | 600 | Subsection titles |
| **Body Text** | Inter | 16px | 400 | Primary content |
| **Code** | JetBrains Mono | 14px | 400 | Code snippets |
| **Code Block** | JetBrains Mono | 14px | 400 | Multi-line code |

**3.2 Color Usage**

| Purpose | Color | Hex | Usage |
|---------|-------|-----|-------|
| **Primary** | Blue | #0066CC | Links, primary actions |
| **Secondary** | Teal | #00A0A0 | Secondary actions |
| **Accent** | Orange | #FF8C00 | Call-to-actions |
| **Success** | Green | #28A745 | Success messages |
| **Error** | Red | #DC3545 | Error messages |
| **Warning** | Yellow | #FFC107 | Warning messages |
| **Code Background** | Gray | #F8F9FA | Code blocks |

**3.3 Code Block Styling**

All code blocks must include:

```markdown
```rust
// Example Rust code
pub fn compile_document(doc: &Document) -> Result<Html> {
    // JIT compilation logic
}
```

```bash
# Example shell command
tachyon serve --port 8080
```

```toml
# Example configuration
[server]
port = 8080
```

```json
// Example JSON
{
  "name": "Tachyon",
  "version": "0.2.0"
}
```
```

**Language Identification:**
- Always specify language for syntax highlighting
- Use lowercase language identifiers

#### 4. Content Consistency

**4.1 Code Examples**

All code examples must:

1. **Be Executable**: Code should run without modifications
2. **Include Comments**: Explain complex logic
3. **Use Realistic Values**: Don't use placeholder data
4. **Follow Style Guide**: Follow project coding standards

**Example:**

```markdown
### Creating a Document

Use the `tachyon create` command to create a new document:

```bash
tachyon create --title "API Reference" --tags "api,reference"
```

This command:
- Creates a new Markdown document in the repository
- Adds the specified title and tags
- Opens the document in your default editor

**Options:**
- `--title`: Document title (required)
- `--tags`: Comma-separated tags (optional)
- `--author`: Document author (optional)
```

**4.2 Cross-References**

All cross-references must use relative paths and include context:

```markdown
For more information on configuration, see the [Configuration Guide](./configuration_guide.md#server-configuration).

See the API Reference for details on the [Document API](./api_reference.md#document-api).
```

**Link Format Rules:**

| Rule | Example |
|------|---------|
| **Relative paths** | `./configuration_guide.md` |
| **Section anchors** | `./configuration_guide.md#server-configuration` |
| **Context in text** | "See the Configuration Guide for more information" |
| **Descriptive text** | "Configuration Guide" (not "click here") |

**4.3 Version-Specific Content**

When documenting version-specific features:

```markdown
## JIT Compilation

**Introduced:** Version 0.2.0

Tachyon uses JIT (Just-In-Time) compilation for sub-15ms rendering.

### Version 0.2.0

- Three-tier compilation strategy
- LRU cache with >80% hit rate
- BM25 search with <100ms query time

### Version 0.3.0 (Planned)

- Advanced caching strategies
- Improved search relevance
```

#### 5. Style Guidelines

**5.1 Voice and Tone**

**Tone:**
- Professional and approachable
- Technical but accessible
- Direct and concise
- Evidence-based

**Writing Guidelines:**

1. **Use Active Voice**
   ```markdown
   # Correct
   Tachyon compiles documents in sub-15ms.

   # Incorrect
   Documents are compiled by Tachyon in sub-15ms.
   ```

2. **Be Specific**
   ```markdown
   # Correct
   Tachyon compiles documents in sub-15ms with 80% cache hit rate.

   # Incorrect
   Tachyon compiles documents quickly with high cache hit rate.
   ```

3. **Address the User Directly**
   ```markdown
   # Correct
   You can configure Tachyon using the `tachyon.toml` file.

   # Incorrect
   Tachyon can be configured using the `tachyon.toml` file.
   ```

4. **Keep Sentences Short**
   ```markdown
   # Correct
   Tachyon compiles documents in sub-15ms. The LRU cache achieves an 80% hit rate.

   # Incorrect
   Tachyon compiles documents in sub-15ms and the LRU cache achieves an 80% hit rate.
   ```

**5.2 Formatting**

**Lists:**
- Use numbered lists for sequential steps
- Use bulleted lists for non-sequential items
- Keep list items parallel in structure

**Emphasis:**
- Use bold for key terms and important concepts
- Use italics for emphasis (rarely)
- Never use all caps (except for acronyms)

**Headings:**
- Use sentence case for headings (Capitalize first word only)
- Use descriptive headings that summarize content
- Avoid "Introduction" or "Overview" as section titles

**5.3 Accessibility**

All documentation must be WCAG 2.1 AA compliant:

- Proper heading hierarchy (H1, H2, H3, etc.)
- Alt text for all images
- Color-independent information conveyance
- Minimum 4.5:1 contrast ratio for text
- Keyboard navigation support

#### 6. Consistency Enforcement

**6.1 Automated Checks**

**Linting Tools:**

```bash
# markdownlint for Markdown linting
npx markdownlint .docs/

# vale for prose linting
vale .docs/

# cspell for spell checking
npx cspell .docs/
```

**CI/CD Integration:**

```yaml
# .github/workflows/doc-lint.yml
name: Documentation Linting
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Lint Markdown
        run: npx markdownlint .docs/
      - name: Lint Prose
        run: vale .docs/
      - name: Spell Check
        run: npx cspell .docs/
```

**6.2 Manual Review Process**

**Review Checklist:**

| Category | Items |
|----------|-------|
| **Terminology** | Terms match glossary, acronyms expanded on first use |
| **Structure** | Header present, sections organized, cross-references correct |
| **Formatting** | Code blocks have language, links use relative paths |
| **Style** | Active voice, specific language, user-centered |
| **Accessibility** | Heading hierarchy, alt text, color contrast |
| **Accuracy** | Content matches implementation, examples work |

**Review Workflow:**

1. **Self-Review**: Author reviews against checklist
2. **Peer Review**: Another team member reviews
3. **Tech Review**: Subject matter expert reviews technical accuracy
4. **Final Approval**: Doc team lead approves

**6.3 Drift Detection**

**Automated Drift Detection:**

```bash
# Check for code-doc consistency
# (from .adrs/
tachyon verify-doc-consistency
```

**Drift Detection Triggers:**

| Trigger | Action |
|---------|--------|
| **API Change** | API documentation must be updated |
| **Configuration Change** | Configuration guide must be updated |
| **Feature Addition** | User guide must be updated |
| **Deprecation** | Migration guide must be updated |

#### 7. Consistency Metrics

**7.1 Key Metrics**

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Terminology Consistency** | 100% | Automated check |
| **Structure Consistency** | 100% | Manual review |
| **Style Consistency** | 100% | Automated + manual |
| **Accessibility Compliance** | 100% | Automated + manual |
| **Code-Doc Consistency** | 100% | Automated check |

**7.2 Quality Gates**

```yaml
quality_gates:
  consistency:
    - terminology_consistency: 100%
    - structure_consistency: 100%
    - style_consistency: 100%
    - accessibility_compliance: wcag_aa
    - code_doc_consistency: 100%
```

#### 8. Maintenance Strategy

**8.1 Regular Audits**

| Audit Type | Frequency | Responsibility |
|------------|-----------|----------------|
| **Terminology Review** | Monthly | Doc Team |
| **Structure Review** | Quarterly | Doc Team |
| **Style Review** | Monthly | Doc Team |
| **Accessibility Audit** | Quarterly | Accessibility Specialist |
| **Code-Doc Consistency** | Weekly | Automated |

**8.2 Update Process**

When updating documentation:

1. **Check Consistency**: Review against style guide
2. **Update Glossary**: Add new terms, update existing terms
3. **Update Cross-References**: Verify all links still work
4. **Run Linters**: Fix all automated issues
5. **Request Review**: Submit for peer review
6. **Update Version**: Increment document version number

**8.3 Continuous Improvement**

- Collect user feedback on documentation
- Analyze frequently asked questions
- Identify areas needing clarification
- Update style guide based on lessons learned
- Recognize contributors who maintain consistency

## Consequences

### Positive Consequences

1. **Consistent User Experience**
   - Users have predictable documentation across all materials
   - Reduced confusion and support burden

2. **Improved Maintainability**
   - Clear guidelines for creating and updating docs
   - Automated checks catch issues early

3. **Better Quality**
   - Consistent style and structure improve readability
   - Accessibility compliance built into workflow

4. **Easier Onboarding**
   - New contributors have clear guidelines to follow
   - Reduced learning curve for documentation

### Negative Consequences

1. **Initial Overhead**
   - Setting up consistency tools takes time
   - Learning style guide requires investment

2. **Process Rigidity**
   - Guidelines may feel restrictive for some content
   - May require exceptions for special cases

3. **Ongoing Maintenance**
   - Consistency checks require continuous attention
   - Style guide needs periodic updates

## Alternatives Considered

1. **No Consistency Guidelines**
   - Rejected: Would lead to inconsistent, confusing documentation

2. **Minimal Guidelines**
   - Rejected: Insufficient for professional documentation

3. **Strict Enforcement**
   - Rejected: Too rigid, would stifle creativity

## References

- [White Paper](../.adrs/
- [ADR-062: Brand Identity](./adr-062-brand-identity.md)
- [ADR-063: UX Philosophy](./adr-063-ux-philosophy.md)
- [ADR-064: Documentation Strategy](./adr-064-documentation-strategy.md)
- [ADR-066: Multi-lingual](./adr-066-multi-lingual.md)
- [Glossary](../.docs/glossary.md)
- [Doc Verification Plan](../.adrs/
- [Google Developer Documentation Style Guide](https://developers.google.com/tech-writing/one-pagers)
- [Microsoft Style Guide](https://docs.microsoft.com/en-us/style-guide/)

## Implementation

### Phase 1: Foundation (Week 1-2)
- [ ] Finalize consistency framework
- [ ] Create style guide
- [ ] Set up linting tools
- [ ] Define review process

### Phase 2: Tooling (Week 3-4)
- [ ] Configure markdownlint
- [ ] Configure vale
- [ ] Configure cspell
- [ ] Integrate with CI/CD

### Phase 3: Enforcement (Week 5-6)
- [ ] Implement automated checks
- [ ] Set up manual review process
- [ ] Train contributors on style guide
- [ ] Create review checklist

### Phase 4: Validation (Week 7-8)
- [ ] Conduct consistency audit
- [ ] Fix consistency issues
- [ ] Gather feedback on process
- [ ] Refine based on feedback

### Phase 5: Maintenance (Ongoing)
- [ ] Monitor consistency metrics
- [ ] Conduct regular audits
- [ ] Update style guide as needed
- [ ] Recognize contributors

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | 2026-02-11 | Brand Strategist | Initial consistency framework |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant with proper heading structure, sufficient color contrast, and keyboard navigation support.
