---
title: Post-Mortem Template
description: Template for conducting post-incident reviews and documenting lessons learned
date: 2026-05-23
category: operations
order: 4
---

# Post-Mortem Template

Use this template after any P1/P2 incident. Complete within 3 business days of resolution.

## Incident Summary

| Field | Value |
|-------|-------|
| **Incident ID** | INC-YYYY-NNN |
| **Date/Time (UTC)** | Start: YYYY-MM-DD HH:MM / End: YYYY-MM-DD HH:MM |
| **Duration** | X hours Y minutes |
| **Severity** | P1 / P2 |
| **Impact** | X users affected, Y% error rate, Z minutes downtime |
| **Resolved by** | Name / Team |
| **Post-mortem owner** | Name |
| **Review date** | YYYY-MM-DD |

## Timeline (UTC)

| Time | Event |
|------|-------|
| HH:MM | Alert triggered / Incident detected |
| HH:MM | First responder acknowledged |
| HH:MM | Root cause identified |
| HH:MM | Mitigation applied |
| HH:MM | Full resolution confirmed |
| HH:MM | Stakeholders notified |

## Root Cause Analysis

### What happened

[Detailed technical description of the failure]

### Why it happened

[Underlying cause — use 5-Why analysis]

### Why it was not caught earlier

[Gap in testing, monitoring, or process]

## Impact Assessment

- **Users affected**: [Number or percentage]
- **Data loss**: [Yes/No — if yes, describe scope]
- **Revenue impact**: [Estimated if applicable]
- **Reputation impact**: [External-facing or internal-only]

## What Went Well

- [Good practices that helped resolve quickly]

## What Could Be Improved

- [Process gaps, tooling limitations, communication issues]

## Action Items

| ID | Action | Owner | Priority | Due Date | Status |
|----|--------|-------|----------|----------|--------|
| A1 | [Specific actionable item] | Name | High | YYYY-MM-DD | Open |
| A2 | [Specific actionable item] | Name | Medium | YYYY-MM-DD | Open |

## Lessons Learned

1. [Key takeaway for the team]
2. [Process improvement recommendation]
3. [Technical improvement recommendation]

## Appendix

- Monitoring screenshots / dashboards
- Relevant log excerpts (sanitized)
- Link to incident Slack channel
- Link to deployment diff (if applicable)
