# ADR-071: Reusable Templates

**Status:** Accepted

**Date:** 2024-02-11

## Context

The Tachyon project developed numerous document structures, configurations, and workflows that can be reused in future projects. Creating reusable templates will:

- Accelerate project initialization for future projects
- Ensure consistency across projects
- Reduce learning curve for new team members
- Capture best practices in reusable form
- Improve documentation quality and completeness

Without reusable templates:
- Each project starts from scratch
- Inconsistent structures across projects
- Repeated effort to create similar artifacts
- Higher learning curve for new projects
- Quality varies based on individual knowledge

## Decision

We will establish a comprehensive Reusable Templates library with the following structure:

### Specification Document

**Reusable Templates Specification** (`.adrs/
- Project structure templates
- Configuration templates
- CI/CD pipeline templates
- Documentation templates
- ADR templates
- Test templates
- Deployment templates

### Template Categories

1. **Project Structure Templates**
   - Tachyon workspace structure
   - Documentation structure (Diataxis framework)
   - Specification directory structure
   - ADR directory structure
   - Pattern directory structure

2. **Configuration Templates**
   - Cargo.toml workspace template
   - flake.nix template
   - Dockerfile template
   - Environment configuration template

3. **CI/CD Pipeline Templates**
   - Multi-stage pipeline configuration
   - Quality gates configuration
   - Blue-green deployment template
   - Canary deployment template
   - SBOM generation template
   - Performance regression detection template

4. **Documentation Templates**
   - Requirements specification template
   - Architecture specification template
   - Security specification template
   - Performance specification template
   - API documentation template
   - User guide template

5. **ADR Template**
   - ADR header template
   - Context section template
   - Decision section template
   - Consequences section template
   - Implementation notes template

6. **Test Templates**
   - Unit test template
   - Integration test template
   - Fuzzing test template
   - Concurrency test template
   - Property-based test template

7. **Deployment Templates**
   - Blue-green deployment template
   - Canary deployment template
   - Health check template
   - Rollback procedure template

### Template Characteristics

Each template includes:
- Clear structure and organization
- Placeholders for project-specific content
- Comments explaining each section
- Examples of completed sections
- References to related documentation

## Consequences

### Positive

- Accelerated project initialization for future projects
- Consistent structures across projects
- Reduced learning curve for new team members
- Captured best practices in reusable form
- Improved documentation quality and completeness
- Reduced duplication of effort
- Easier onboarding for new projects

### Negative

- Additional documentation overhead during development
- Requires discipline to maintain and update templates
- May need periodic updates as practices evolve

### Neutral

- Reusable templates are a living document that will evolve with the project

## Alternatives Considered

1. **No formal reusable templates**
   - Rejected: Each project would start from scratch
   - Impact: Slower project initialization and inconsistent structures

2. **Only provide code templates**
   - Rejected: Configuration and documentation templates are equally valuable
   - Impact: Would miss opportunities for broader reuse

3. **Include templates in pattern library**
   - Rejected: Templates deserve separate documentation due to different focus (reuse vs. patterns)
   - Impact: Would dilute clarity of pattern library

## Implementation Notes

The reusable templates have been implemented in Phase 7.5: Knowledge Base Update. All templates include:
- Clear structure and organization
- Placeholders for project-specific content
- Comments explaining each section
- Examples of completed sections
- References to related documentation

Templates are organized by category and can be easily copied and modified for new projects. Each template includes a "How to Use" section with instructions for customization.

## Related ADRs

- [ADR-068: Pattern Library](adr-068-pattern-library.md)
- [ADR-069: Anti-Pattern Library](adr-069-anti-pattern-library.md)
- [ADR-070: Lessons Learned](adr-070-lessons-learned.md)
