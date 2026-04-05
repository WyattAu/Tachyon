# ADR-069: Anti-Pattern Library

**Status:** Accepted

**Date:** 2024-02-11

## Context

During the Tachyon project development, we encountered numerous common mistakes and anti-patterns that should be documented to avoid repetition in future projects. Anti-patterns are particularly valuable because they:
- Help identify and prevent common mistakes
- Provide clear examples of what NOT to do
- Include consequences to emphasize the importance of avoidance
- Offer solutions to correct or prevent the anti-pattern

Without a structured anti-pattern library:
- Teams repeat the same mistakes
- New developers don't learn from past errors
- Debugging time increases due to preventable issues
- Code quality suffers from repeated anti-patterns

## Decision

We will establish a comprehensive Anti-Pattern Library with the following structure:

### Specification Document

**Anti-Pattern Library Specification** (`.specs/08_5_knowledge_base/anti_patterns.md`)
- 67 anti-patterns across 7 categories
- Each anti-pattern includes: ID, name, category, severity, problem, consequences, solution, traceability

### Pattern Files

**Rust Anti-Patterns** (`.patterns/rust_anti_patterns.md`)
- Concurrency anti-patterns (blocking async operations, unbounded channels)
- I/O anti-patterns (synchronous file operations in async context)
- Security anti-patterns (security through obscurity, hardcoded credentials)
- Performance anti-patterns (cache without monitoring, cloning large data structures)

### Anti-Pattern Categories

1. **Rust Language Anti-Patterns** (12 patterns)
   - Blocking async operations in async context
   - Unbounded channel buffers
   - Panic on expected errors
   - Silent error swallowing
   - Unintentional reference cycles
   - Cloning large data structures

2. **Architecture Anti-Patterns** (10 patterns)
   - Cache stampede (multiple requests for same uncached data)
   - Cache without invalidation (serving stale data)
   - N+1 query problem
   - Mutex lock contention
   - Big bang deployment

3. **CI/CD Anti-Patterns** (8 patterns)
   - Big bang deployment
   - Ignoring security scan failures
   - Drift without detection
   - Flaky tests in pipeline
   - Ignoring quality gate failures

4. **Security Anti-Patterns** (10 patterns)
   - Security through obscurity
   - Incomplete input sanitization
   - Hardcoded credentials
   - Weak authentication
   - Missing validation at trust boundaries

5. **Performance Anti-Patterns** (9 patterns)
   - Cache without hit rate monitoring
   - Ignoring performance benchmarks
   - Unnecessary cloning
   - Blocking operations in async context

6. **Documentation Anti-Patterns** (6 patterns)
   - Outdated API documentation
   - Missing examples
   - Drift from code without detection

7. **Project Management Anti-Patterns** (5 patterns)
   - Skipping ADRs for decisions
   - Undefined success criteria
   - Vague requirements

### Severity Classification

- **Critical:** Must fix immediately (security vulnerabilities, data corruption)
- **High:** Should fix soon (performance issues, stability problems)
- **Medium:** Should address (usability issues, maintainability concerns)
- **Low:** Nice to have (style issues, minor optimizations)

## Consequences

### Positive

- Centralized anti-pattern repository for all categories
- Clear severity classification for prioritization
- Easy reference for code reviews and quality assurance
- Reduced debugging time
- Fewer repeated mistakes
- Better code quality
- Improved team learning

### Negative

- Additional documentation overhead during development
- Requires discipline to maintain anti-pattern library
- May need periodic updates as new anti-patterns are discovered

### Neutral

- Anti-pattern library is a living document that will evolve with the project

## Alternatives Considered

1. **No formal anti-pattern library**
   - Rejected: Teams would repeat the same mistakes
   - Impact: Higher debugging time and lower code quality

2. **Only document patterns (not anti-patterns)**
   - Rejected: Learning from mistakes is as important as learning from successes
   - Impact: Would miss valuable learning opportunities

3. **Include anti-patterns in pattern library**
   - Rejected: Anti-patterns deserve separate documentation due to different focus (prevention vs. adoption)
   - Impact: Would dilute clarity of pattern library

## Implementation Notes

The anti-pattern library has been implemented in Phase 7.5: Knowledge Base Update. All anti-pattern files include:
- Clear anti-pattern categorization
- Severity classification for prioritization
- Problem-consequences-solution format for easy reference
- Traceability to project artifacts (ADRs, specs, reports)
- Implementation examples where applicable
- References to related documentation

## Related ADRs

- [ADR-068: Pattern Library](adr-068-pattern-library.md)
- [ADR-070: Lessons Learned](adr-070-lessons-learned.md)
- [ADR-071: Reusable Templates](adr-071-reusable-templates.md)
