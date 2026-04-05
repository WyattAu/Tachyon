# ADR-105: Cross-Project Sharing

## Status

**Status:** Accepted
**Date:** 2026-02-12
**Decision Date:** 2026-02-12

## Context

The Tachyon project has accumulated valuable knowledge, patterns, and lessons learned. Sharing this knowledge with other projects can accelerate development, improve decision quality, and prevent repeated mistakes. However, sharing must be done systematically to ensure relevance, accessibility, and proper attribution.

## Problem

How do we share Tachyon project knowledge (patterns, anti-patterns, lessons learned) with other projects in a way that maximizes value while maintaining proper governance and security?

## Decision

### Cross-Project Sharing Framework

Adopt a structured cross-project sharing framework with the following components:

1. **Knowledge Categorization:** Organize by type (patterns, anti-patterns, lessons)
2. **Applicability Assessment:** Evaluate relevance for different project types
3. **Access Control:** Define who can access what knowledge
4. **Documentation Standards:** Consistent format and structure
5. **Version Management:** Track evolution of shared knowledge
6. **Feedback Loop:** Collect and incorporate feedback from consuming projects

### Sharing Channels

| Channel | Description | Target Audience | Frequency |
|---------|-------------|----------------|----------|
| Internal Wiki | Central knowledge base | All teams |
| Documentation Portal | Published documentation | External partners |
| Team Meetings | Presentations and demos | Development teams |
| Code Reviews | Pattern discussions | Engineering teams |
| Training Sessions | Formal knowledge transfer | All teams |
| Project Management System | Ticket tracking | Project managers |

### Access Levels

| Level | Description | Approval Required | Examples |
|-------|-------------|-----------------|----------|
| Full | Read, download, apply | Knowledge Manager | Core architecture teams |
| Read-Only | View, reference | All team members | Pattern consumers |
| Download | Download to local | Yes | External auditors |
| Modify | Suggest changes | Yes | Knowledge Manager | Senior architects |
| Admin | Manage sharing | Yes | Project Manager | Archive administrators |

## Consequences

### Positive Consequences

- **Accelerated Development:** New projects leverage existing patterns and lessons
- **Improved Quality:** Avoid known anti-patterns and pitfalls
- **Better Decisions:** Informed by proven experience
- **Knowledge Preservation:** Critical knowledge preserved for long-term reference
- **Consistency:** Standardized approaches across projects
- **Team Enablement:** Reduced onboarding time for new team members

### Negative Consequences

- **Time Investment:** Requires time to package and share knowledge
- **Maintenance Overhead:** Keeping shared knowledge up to date
- **Context Loss:** Knowledge may be applied inappropriately without context
- **Security Risk:** Sensitive patterns could be exposed to wrong audiences

## Alternatives Considered

1. **Informal Sharing:** Rejected due to lack of structure and accessibility
2. **Project-Specific Repositories:** Rejected due to fragmentation
3. **Ad-Hoc Sharing:** Rejected due to inconsistency and lack of traceability
4. **No Sharing:** Rejected due to loss of organizational knowledge

Rejected Reason: Structured framework with governance provides maximum value while maintaining security and consistency.

## Implementation

### Knowledge Packaging

**Format:** Markdown (.md files)
**Location:** `.patterns/` directory
**Structure:**
- `.patterns/global_pattern_library.md` - Design and implementation patterns
- `.patterns/global_anti_pattern_library.md` - Anti-patterns to avoid
- `.patterns/lessons_learned_database.md` - Lessons learned

Each document includes:
- Clear categorization
- Applicability context
- Implementation guidance
- Related resources

### Applicability Matrix

Include applicability matrix in each shared document:
- Pattern applicability by project type (Knowledge Management, Web Application, Mobile App, etc.)
- Anti-Pattern relevance by domain (Security, Performance, Architecture)
- Lesson applicability by phase (Requirements, Architecture, Implementation, Testing)

### Sharing Process

1. **Review:** Knowledge Manager reviews proposed sharing
2. **Classify:** Categorize knowledge by type and applicability
3. **Sanitize:** Remove project-specific sensitive information
4. **Document:** Add context and applicability guidance
5. **Approve:** Project Manager approves for publication
6. **Publish:** Publish to appropriate channels
7. **Notify:** Inform stakeholders of new shared knowledge

### Access Control

- RBAC for access levels
- Audit logging for all access
- Time-limited credentials for external access
- Regular access reviews

## Related Decisions

- [ADR-090](.adrs/adr-090-lessons-learned-documentation-strategy.md) - Lessons learned documentation framework
- [ADR-091](.adrs/adr-091-knowledge-transfer-strategy.md) - Knowledge transfer strategy
- [ADR-104](.adrs/adr-104-knowledge-graph-finalization.md) - Knowledge graph finalization

## References

- [Cross-Project Sharing Strategy](.knowledge_graph/cross_project_sharing.md)
- [Global Pattern Library](.patterns/global_pattern_library.md)
- [Global Anti-Pattern Library](.patterns/global_anti_pattern_library.md)
- [Lessons Learned Database](.patterns/lessons_learned_database.md)

---

**Document Status:** COMPLETE
**Owner:** Knowledge Manager
**Reviewers:** TBD
**Approved By:** TBD
