# ADR-108: Anti-Pattern Library Update

## Status

**Status:** Accepted
**Date:** 2026-02-12
**Decision Date:** 2026-02-12

## Context

The Tachyon project has identified 5 anti-patterns across multiple categories (concurrency, architecture, security, performance). Documenting these anti-patterns with consequences, prevention strategies, and related patterns helps other projects avoid common pitfalls and improve code quality.

## Problem

How do we update the global anti-pattern library with Tachyon project anti-patterns to make them available for cross-project sharing and future reference?

## Decision

### Anti-Pattern Library Update Strategy

Update the global anti-pattern library (`.patterns/global_anti_pattern_library.md`) with Tachyon project anti-patterns, organizing them by category with description, consequences, prevention strategies, and related patterns.

### Anti-Pattern Categories to Add

1. **Concurrency Anti-Patterns** (2 patterns)
   - Synchronous Blocking Operations
   - Mutex Contention

2. **Architecture Anti-Patterns** (1 pattern)
   - God Module

3. **Security Anti-Patterns** (1 pattern)
   - Implicit Authentication

4. **Performance Anti-Patterns** (1 pattern)
   - Cache Miss Storm

### Anti-Pattern Documentation Structure

For each anti-pattern:
- **Anti-Pattern ID:** Unique identifier (e.g., AP-SYNC-BLOCKING)
- **Anti-Pattern Name:** Clear, descriptive name
- **Category:** High-level category (e.g., Concurrency)
- **Severity:** Critical/High/Medium/Low
- **Description:** What the anti-pattern is
- **Consequences:** What happens if anti-pattern is used
- **Examples:** Code or scenario examples
- **Prevention Strategies:** How to avoid the anti-pattern
- **Related Pattern:** What pattern to use instead
- **Related Threats:** Security threats if applicable

### Global Anti-Pattern Library File

**Location:** `.patterns/global_anti_pattern_library.md`
**Format:** Markdown
**Access:** Read-only for consuming projects

### Update Process

1. **Extract:** Extract anti-patterns from project analysis
2. **Format:** Reformat to global library structure
3. **Add Context:** Include examples and consequences
4. **Review:** Knowledge Manager reviews for accuracy
5. **Publish:** Update global library with new anti-patterns
6. **Version:** Increment version number
7. **Notify:** Inform stakeholders of anti-pattern library updates

## Consequences

### Positive Consequences

- **Pitfall Avoidance:** Projects can avoid known mistakes
- **Improved Code Quality:** Prevention of anti-patterns leads to better code
- **Security Enhancement:** Security anti-patterns awareness prevents vulnerabilities
- **Performance Optimization:** Performance anti-patterns avoidance improves efficiency
- **Knowledge Sharing:** Centralized resource for common pitfalls

### Negative Consequences

- **Maintenance Overhead:** Anti-pattern library requires updates as projects evolve
- **Context Loss:** Anti-patterns may be applied inappropriately
- **False Positives:** Anti-pattern detection may flag acceptable code patterns

## Alternatives Considered

1. **No Anti-Pattern Documentation:** Rejected due to risk of repeated mistakes
2. **Project-Specific Only:** Rejected due to fragmentation
3. **Informal Warnings Only:** Rejected due to lack of structure
4. **Manual Detection Only:** Rejected due to time inefficiency

Rejected Reason: Centralized global anti-pattern library provides maximum value for cross-project learning while maintaining consistency and accessibility.

## Implementation

### Anti-Pattern Extraction

From project analysis, post-mortem reviews, and architecture decisions, extract anti-patterns and format for global library:

```markdown
## Anti-Pattern Category

### Anti-Pattern Name

**Anti-Pattern ID:** AP-SYNC-BLOCKING
**Category:** Concurrency
**Severity:** Critical
**Frequency:** Common in async Rust codebases

**Description:** [Full description]

**Consequences:**
- [List of consequences]

**Examples:**
```rust
// [Code example]
```

**Prevention Strategies:**
- [List of prevention strategies]

**Related Pattern:** [Related pattern ID]
**Related Threats:** [Related threat IDs]
```

### Global Anti-Pattern Library File

**Location:** `.patterns/global_anti_pattern_library.md`
**Format:** Markdown
**Access:** Read-only for consuming projects

### Update Process

1. **Extract:** Identify anti-patterns from project analysis
2. **Categorize:** Organize by category and severity
3. **Document:** Provide clear examples and prevention strategies
4. **Link:** Cross-reference to related patterns and threats
5. **Review:** Knowledge Manager validates accuracy
6. **Publish:** Update global library with new anti-patterns
7. **Version:** Increment version number

## Related Decisions

- [ADR-069](.adrs/adr-069-anti-pattern-library.md) - Anti-pattern library structure
- [ADR-090](.adrs/adr-090-lessons-learned-documentation-strategy.md) - Lessons learned documentation
- [ADR-104](.adrs/adr-104-knowledge-graph-finalization.md) - Knowledge graph finalization

## References

- [Tachyon Project Analysis](.specs/10_metrics/post_mortem.md)
- [Global Anti-Pattern Library](.patterns/global_anti_pattern_library.md)

---

**Document Status:** COMPLETE
**Owner:** Knowledge Manager
**Reviewers:** TBD
**Approved By:** TBD
