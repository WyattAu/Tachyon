# ADR-061: Code Example Validation in Documentation

## Status
**Accepted**

## Context
Documentation code examples are critical for user understanding but often become outdated or contain errors. Manual validation is impractical at scale. We need automated validation to ensure examples remain correct and executable.

## Decision
Implement automated code example validation that checks syntax, compilation, execution, and output verification for all examples in documentation.

## Consequences
### Positive
- Examples always compile and run correctly
- Early detection of breaking API changes
- Improved user experience with working examples
- Reduced support burden from broken examples
- Verification of documentation accuracy

### Negative
- Requires test harness infrastructure
- Potential execution environment issues
- Additional CI/CD pipeline time
- Need for sandboxing for security

## Alternatives Considered
1. **Manual Testing**
   - Pros: Human judgment, flexible
   - Cons: Time-consuming, inconsistent, not scalable
   
2. **Doctest Integration Only**
   - Pros: Native Rust tooling
   - Cons: Limited to Rust examples, no cross-language support

3. **User-Reported Issues**
   - Pros: Real user feedback
   - Cons: Reactive, delayed detection, inconsistent quality

## Implementation Details
The example validation system provides:

### Example Extraction
- Markdown code block parsing
- Language identification (Rust, TypeScript, Bash, JSON, YAML)
- Metadata extraction (ignore flags, compile-only, run-only)
- Dependency detection from code

### Validation Levels
| Category | Language | Validation Level | Auto-fixable |
|----------|----------|------------------|--------------|
| Rust Examples | Rust | Compilation + Execution | Partial |
| TypeScript Examples | TypeScript | Type checking | Yes |
| Shell Commands | Bash/Sh | Syntax check | No |
| JSON/YAML | JSON/YAML | Schema validation | Yes |
| HTTP Examples | HTTP | Live API testing | No |

### Metadata Markers
Supported metadata for code examples:
- `ignore`: Skip all validation for this example
- `no_compile`: Skip compilation, only check syntax
- `no_run`: Compile but do not execute
- `should_panic`: Expect and validate panic behavior
- `expected: <output>`: Verify output matches expected

### Security Considerations
- Sandbox execution environment
- Resource limits (memory, CPU, network)
- Hardcoded credential detection
- Dangerous command prevention

## Compliance
- IEEE 1016-2009: Recommended Practice for Software Design Descriptions
- ISO/IEC 25010: Software Product Quality Requirements
- Rust Documentation Testing Guidelines
- TypeScript Testing Best Practices

## References
- [.adrs/
- [ADR-058: Consistency Checks](./adr-058-consistency-checks.md)
