# ADR-095: Post-Mortem Process for Level 4+ Errors

## Status

**Status:** Accepted  
**Date:** 2026-02-12  
**Decision Date:** 2026-02-12  

## Context

Level 4+ errors require structured post-mortem analysis to understand root causes, develop corrective actions, and prevent recurrence. This ADR defines the post-mortem process for such errors.

## Problem

How do we conduct effective post-mortem analyses for Level 4+ errors while maintaining a blameless culture and focusing on learning and improvement?

## Decision

### Post-Mortem Framework

The Tachyon project adopts a structured post-mortem framework for Level 4+ errors:

1. **Immediate Response:** Containment and assessment
2. **Investigation:** Root cause analysis
3. **Documentation:** Comprehensive incident documentation
4. **Corrective Actions:** Actionable improvement items
5. **Follow-Up:** Verification of effectiveness
6. **Knowledge Integration:** Lessons captured in knowledge base

### Post-Mortem Requirements

| Requirement | Description | Status |
|-------------|-------------|--------|
| Conducted within 14 days | Post-mortem completed promptly | TBD |
| Root cause identified | Root cause clearly identified | TBD |
| Action items defined | Corrective actions are actionable | TBD |
| Lessons documented | Lessons captured in knowledge base | TBD |
| Follow-up scheduled | Follow-up for effectiveness verification | TBD |

## Consequences

### Positive Consequences

- Improved understanding of root causes
- Reduced risk of recurrence
- Better incident response procedures
- Enhanced knowledge base
- Improved team processes

### Negative Consequences

- Time required for post-mortem process
- Potential for defensive behavior
- Risk of incomplete analysis
- Need for facilitation skills

## Alternatives Considered

1. **Informal Review Only:** Rejected due to lack of structure
2. **Blame-Focused Review:** Rejected due to culture impact
3. **Delayed Analysis:** Rejected due to need for timely learning

## Implementation

### Post-Mortem Process

1. **Incident Detection:** Level 4+ error identified
2. **Immediate Response:** Containment actions taken
3. **Post-Mortem Triggered:** Decision to conduct analysis
4. **Team Assembly:** Assemble post-mortem team
5. **Investigation:** Root cause analysis
6. **Documentation:** Post-mortem report created
7. **Action Planning:** Corrective actions defined
8. **Implementation:** Actions implemented
9. **Follow-Up:** Effectiveness verified

### Post-Mortem Team Composition

| Role | Responsibilities |
|-------|-----------------|
| Incident Lead | Overall coordination, report authorship |
| Technical Lead | Technical investigation, root cause analysis |
| Quality Lead | Process review, gap identification |
| Security Lead | Security review (if applicable) |
| Project Manager | Action item tracking, stakeholder communication |

### Post-Mortem Timeline

| Phase | Timeframe | Activities |
|-------|------------|-------------|
| Immediate | 0-24 hours | Initial assessment, containment |
| Analysis | 1-7 days | Investigation, root cause analysis |
| Documentation | 7-14 days | Report writing, review |
| Actions | 14-30 days | Implementation of corrective actions |
| Follow-Up | 30+ days | Effectiveness verification |

### Post-Mortem Report Template

See [`.adrs/ for detailed post-mortem template and process.

### Knowledge Base Integration

All Level 4+ error post-mortems are integrated into:
- [`.adrs/ - Lessons Learned
- [`.adrs/ - Anti-Patterns to avoid
- Post-mortem archive for historical reference

### Post-Mortem Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Post-Mortems Completed | TBD | TBD | TBD |
| Root Cause Identified Rate | 100% | TBD | TBD |
| Actions Implemented Rate | >= 90% | TBD | TBD |
| Recurrence Rate | < 5% | TBD | TBD |
| Follow-Up Completion Rate | >= 95% | TBD | TBD |

## Related Decisions

- [ADR-090](.adrs/adr-090-lessons-learned.md) - Lessons Learned Documentation
- [`.adrs/ - Post-Mortem Specification
- [`.adrs/ - Risk Dashboard
- [`.adrs/ - Recovery Time Analysis

## References

- [`.adrs/ - Post-Mortem Specification
- Post-Mortem Analysis Best Practices
- Blameless Post-Mortem Culture

---

**Document Status:** COMPLETE  
**Owner:** Incident Response Lead  
**Reviewers:** TBD  
**Approved By:** TBD
