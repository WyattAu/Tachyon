# ADR-072: Execution Graph Architecture
# Status: ACCEPTED
# Date: 2026-02-12
# Phase: 8 (Execution Graph Generation)

## Context

The Tachyon project requires a systematic approach to implementation that ensures all requirements are addressed in the correct order, with clear dependencies, verification criteria, and risk mitigation strategies. The execution graph serves as the master plan for converting the validated Blue Paper into actionable development tasks.

## Decision

We adopt a Directed Acyclic Graph (DAG) based execution graph with topological sorting for task scheduling and dependency management.

## Alternatives Considered

### Alternative 1: Linear Task List
- **Description:** Tasks executed sequentially in a fixed order
- **Pros:** Simple to implement and track
- **Cons:**
  - Inefficient: Tasks with no dependencies wait unnecessarily
  - Risk-averse: Cannot exploit parallelism
  - Single point of failure: One blocked task stops entire pipeline

### Alternative 2: Gantt Chart
- **Description:** Visual timeline with task bars
- **Pros:** Good for visualization and milestone tracking
- **Cons:**
  - Complex to maintain and update
  - Dependency management becomes implicit and error-prone
  - Difficult to verify correctness of execution order

### Alternative 3: Kanban Board
- **Description:** Columns representing task states (Todo, In Progress, Done)
- **Pros:** Simple, intuitive for teams
- **Cons:**
  - No explicit dependency tracking
  - Risk of executing tasks out of order
  - Limited support for automated verification

### Alternative 4: DAG with Topological Sort (SELECTED)
- **Description:** Directed graph with nodes (tasks) and edges (dependencies) with topological sort algorithm
- **Pros:**
  - Guarantees all prerequisites are satisfied before execution
  - Enables parallel execution of independent tasks
  - Mathematically verifiable correctness
  - Supports critical path analysis
  - Clear visualization of dependencies
  - Supports automated scheduling
- **Cons:**
  - More complex to maintain than linear list
  - Requires careful dependency definition to avoid cycles

## Rationale

The DAG with topological sort is selected because:

1. **Correctness Guarantee:** Topological sort ensures that if task B depends on task A, task A will always be completed before task B begins. This prevents partial implementation states and integration errors.

2. **Parallelism Identification:** Independent tasks can be identified and executed in parallel, reducing overall project duration from 24 weeks to approximately 16-18 weeks when parallelized.

3. **Critical Path Analysis:** The longest path through the DAG can be identified, allowing resource allocation to high-impact tasks and better deadline management.

4. **Automation Support:** The graph structure enables automated task scheduling, CI/CD integration, and progress tracking.

5. **Traceability:** Each task is explicitly linked to requirements, enabling bidirectional traceability and compliance verification.

6. **Risk Mitigation:** Dependencies expose risk points where a single task failure can block multiple downstream tasks, enabling proactive risk management.

7. **Standards Compliance:** This approach aligns with IEEE 1016-2009 (Software Design Descriptions) and ISO/IEC 25010 (Software Quality) requirements for traceability and verification.

## Consequences

### Positive Consequences

- **Predictable Execution:** Task execution order is mathematically determined, reducing uncertainty
- **Resource Optimization:** Independent tasks can be parallelized across developers
- **Progress Tracking:** Clear milestones and completion criteria for each task
- **Dependency Visibility:** All task dependencies are explicit, enabling better coordination
- **Automated Verification:** Verification criteria are defined for each task, enabling automated testing

### Negative Consequences

- **Increased Planning Overhead:** Initial effort required to define 127 tasks with dependencies
- **Dependency Management Complexity:** Careful dependency definition required to avoid cycles
- **Graph Maintenance:** Changes to tasks require graph updates and re-sorting
- **Tooling Requirements:** Need tooling for graph visualization and topological sort

## Implementation Details

### Graph Structure

```toml
[graph]
type = "DAG"
algorithm = "topological_sort"
total_tasks = 127
phases = 4
critical_path_tasks = 42
estimated_duration_weeks = 24
```

### Task Definition Schema

Each task contains:

```toml
[task.TXXX]
id = "TXXX"
name = "Task Name"
phase = 1-4
description = "Human-readable task description"
priority = "CRITICAL|HIGH|MEDIUM|LOW"
estimated_effort_hours = N
prerequisites = ["TYYY", "TZZZ"]
outputs = ["path/to/artifact1", "path/to/artifact2"]
verification_criteria = [
    "Criterion 1",
    "Criterion 2",
    "Criterion 3",
    "Criterion 4"
]
acceptance_criteria = ["AC-XXX-01", "AC-XXX-02"]
requirement_ids = ["RQ-XXX", "RQ-YYY"]
risk_level = "CRITICAL|HIGH|MEDIUM|LOW"
contingency_plans = [
    "Contingency 1",
    "Contingency 2"
]
```

### Topological Sort Algorithm

```rust
fn topological_sort(tasks: &[Task]) -> Result<Vec<Task>, Error<CycleError>> {
    let mut in_degree = HashMap::new();
    let mut queue = VecDeque::new();
    let mut sorted = Vec::new();
    
    // Initialize in-degree and queue with zero in-degree tasks
    for task in tasks {
        in_degree.insert(task.id, task.prerequisites.len());
        if task.prerequisites.is_empty() {
            queue.push_back(task.clone());
        }
    }
    
    // Process tasks in topological order
    while let Some(task) = queue.pop_front() {
        sorted.push(task);
        
        for dependent_id in &task.dependents {
            if let Some(degree) = in_degree.get_mut(dependent_id) {
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(get_task(dependent_id));
                }
            }
        }
    }
    }
    
    // Detect cycles
    if sorted.len() != tasks.len() {
        return Err(CycleError);
    }
    
    Ok(sorted)
}
```

### Phase Structure

The execution graph is organized into 4 phases:

1. **Phase 1: Core Engine** (8 weeks)
   - Content Management (T001-T006, T008)
   - Rendering Engine (T009-T014)
   - Search Engine (T015-T016)
   - Performance Baseline (T017)

2. **Phase 2: The Shell** (6 weeks)
   - Desktop GUI (T020-T021)
   - Web Interface (T024-T026)
   - Integration (T027-T029)

3. **Phase 3: The Editor** (4 weeks)
   - Editor Components (T030-T032)
   - Security (T033-T036)

4. **Phase 4: Ecosystem** (6 weeks)
   - Advanced Features (T040-T045, T070-T073)
   - Monitoring (T050-T053)
   - Deployment (T060-T063)
   - Additional Security (T080-T082)
   - Performance (T090-T092)

### Dependency Tracking

Dependencies are tracked at multiple levels:

- **Direct Dependencies:** Tasks that must complete before this task starts
- **Transitive Dependencies:** All ancestors in the dependency graph
- **Critical Dependencies:** Dependencies on the critical path
- **Optional Dependencies:** Tasks that can be bypassed with degraded functionality

### Verification Strategy

Each task defines verification criteria that:

1. **Measurable:** Can be objectively verified (pass/fail)
2. **Atomic:** Each task produces verifiable artifacts
3. **Complete:** Covers all acceptance criteria for the mapped requirements
4. **Automated:** Can be verified by CI/CD pipeline

### Risk Mitigation

Risk levels are assigned to each task:

- **CRITICAL:** High impact, high probability, or both
- **HIGH:** Moderate impact or probability
- **LOW:** Low impact or probability

Contingency plans define alternative approaches when:

1. Primary implementation fails
2. Performance targets cannot be met
3. Dependencies prove insufficient
4. External constraints emerge

## References

- Master Plan: `.specs/08_roadmap/master_plan.toml`
- Blue Paper: `.specs/02_architecture/blue_paper.md`
- Requirements: `.specs/00_requirements/requirements.md`
- Traceability Matrix: `.specs/00_requirements/traceability_matrix.md`
- IEEE 1016-2009: Software Design Descriptions
- ISO/IEC 25010: Software Quality

## Related Decisions

- ADR-073: Task Dependencies Specification
- ADR-074: Verification Criteria Definition
- ADR-075: Risk Mitigation Strategy
- ADR-076: Traceability Matrix Update

## Status

**Status:** ACCEPTED
**Date:** 2026-02-12
**Reviewers:** Project Manager, Technical Lead
**Next Review:** After Phase 8 completion
