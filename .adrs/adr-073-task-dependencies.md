# ADR-073: Task Dependencies Specification
# Status: ACCEPTED
# Date: 2026-02-12
# Phase: 8 (Execution Graph Generation)

## Context

The Tachyon execution graph consists of 127 tasks organized into 4 phases. Each task may have dependencies on other tasks, which must be completed before the dependent task can begin. This specification defines the dependency structure, validation rules, and management strategies for task dependencies.

## Decision

We adopt a formal dependency specification with explicit prerequisite definitions, dependency validation rules, and dependency management strategies to ensure correct task execution order.

## Alternatives Considered

### Alternative 1: Implicit Dependencies
- **Description:** Dependencies inferred from task names and descriptions
- **Pros:** Less documentation overhead
- **Cons:**
  - Ambiguous: Hard to determine exact relationships
  - Error-prone: Implicit dependencies can be missed
  - Unverifiable: Cannot validate dependency correctness

### Alternative 2: Tag-Based Dependencies
- **Description:** Tasks tagged with category tags, dependencies inferred from tags
- **Pros:** Simple filtering and grouping
- **Cons:**
  - Coarse-grained: Tag-based dependencies are too general
  - Limited precision: Cannot capture specific task relationships
  - Maintenance overhead: Tags must be kept in sync with task changes

### Alternative 3: Formal Dependency Specification (SELECTED)
- **Description:** Each task explicitly lists prerequisite task IDs
- **Pros:**
  - Explicit: No ambiguity about what must complete first
  - Verifiable: Dependencies can be programmatically validated
  - Manageable: Clear visualization of dependency graph
  - Automated: CI/CD can enforce execution order
- **Cons:**
  - Initial effort: Must define all 127 task dependencies manually
  - Maintenance burden: Dependencies must be updated when tasks change
  - Complexity: More upfront effort than implicit approaches

## Rationale

Formal dependency specification is selected because:

1. **Correctness Guarantee:** Explicit dependencies prevent circular dependencies and ensure tasks execute in the correct order, reducing integration errors.

2. **Critical Path Visibility:** Clear dependency chains enable identification of the longest path through the graph, allowing accurate project duration estimation and resource allocation.

3. **Parallel Execution Optimization:** Independent tasks can be identified and executed in parallel, reducing overall project timeline.

4. **Risk Identification:** Dependency relationships expose risk points where a single task failure can block multiple downstream tasks, enabling proactive risk mitigation.

5. **Traceability Compliance:** Explicit dependencies align with IEEE 1016-2009 (Software Design Descriptions) requirements for traceability and design documentation.

6. **Automation Support:** Machine-readable dependency specification enables automated task scheduling, dependency validation, and progress tracking in CI/CD pipelines.

## Consequences

### Positive Consequences

- **Deterministic Execution:** Task execution order is mathematically guaranteed by topological sort
- **Dependency Validation:** Automated validation prevents invalid execution sequences
- **Parallelism Support:** Independent tasks can be identified and executed concurrently
- **Progress Tracking:** Clear completion criteria for each task and its dependencies
- **Risk Management:** Dependency bottlenecks are visible and can be addressed proactively

### Negative Consequences

- **Initial Complexity:** Defining 127 tasks with explicit dependencies requires significant upfront effort
- **Maintenance Overhead:** Adding or removing tasks requires updating dependency specifications
- **Cycle Detection:** Must implement cycle detection to prevent invalid dependency graphs
- **Update Burden:** Task changes may require cascading dependency updates

## Dependency Categories

### Direct Dependencies

Tasks that must complete immediately before this task can begin. These are the most critical dependencies as they create direct blocking points.

**Example:**
```toml
[task.T009]
name = "JIT Compiler Implementation"
prerequisites = ["T001", "T008"]
```

In this example, T009 cannot start until both T001 (Markdown Parser) and T008 (Content Redaction) are complete.

### Transitive Dependencies

All ancestor tasks in the dependency graph, not just immediate predecessors. These must be considered for critical path analysis and resource planning.

**Example:**
```
T001 -> T009 -> T027 -> T035
```

Here, T035 (RBAC Middleware) has transitive dependencies on T001, T009, and T027.

### Critical Dependencies

Dependencies on tasks that are on the critical path or have high-risk impact. Failure of these tasks delays the entire project.

**Critical Path Tasks:**
- T001: Markdown Parser (blocks T009, T007, T041, T042, T043)
- T003: File Watcher (blocks T014, T028)
- T009: JIT Compiler (blocks T010, T011, T012, T013, T040)
- T013: LRU Cache (blocks T014, T027)
- T015: Tantivy Indexer (blocks T016, T023)
- T016: Search Query Engine (blocks T027, T023)
- T024: Web Interface Setup (blocks T025, T026, T027)
- T027: HTTP API Endpoints (blocks T035, T044)
- T028: WebSocket Server (blocks T036)
- T030: Editor Component (blocks T031, T032)
- T034: Authentication Implementation (blocks T035, T070, T072, T073)
- T035: RBAC Middleware (blocks T071)
- T024: Axum HTTP Server (blocks T027, T050, T052)
- T050: Metrics Collection (blocks T053)
- T060: Docker Configuration (blocks T061)

### Optional Dependencies

Dependencies that can be bypassed with degraded functionality if they fail. These tasks are marked with lower priority in the execution graph.

**Example:**
```toml
[task.T043]
name = "Content Migration Importers"
priority = "LOW"
```

If T043 fails, the system can still function without Notion/Confluence import capabilities.

## Dependency Validation Rules

### Rule 1: No Circular Dependencies

The dependency graph must be acyclic (DAG). Circular dependencies prevent topological sort and indicate invalid task specification.

**Validation:**
```rust
fn validate_no_cycles(tasks: &[Task]) -> Result<(), Error<CycleError>> {
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    
    fn check_cycles(task: &Task) -> bool {
        if visited.contains(&task.id) {
            return false; // Already checked
        }
        visited.insert(task.id.clone());
        recursion_stack.insert(task.id.clone());
        
        for dep_id in &task.prerequisites {
            if recursion_stack.contains(dep_id) {
                return false; // Cycle detected
            }
            if let Some(dep_task) = get_task(dep_id) {
                if !check_cycles(dep_task) {
                    return false;
                }
            }
        }
        
        recursion_stack.remove(&task.id);
        true
    }
    
    for task in tasks {
        if !check_cycles(task) {
            return Err(CycleError::CycleDetected(task.id.clone()));
        }
    }
    
    Ok(())
}
```

### Rule 2: Prerequisite Existence

All task IDs listed in `prerequisites` must correspond to valid tasks in the execution graph.

**Validation:**
```rust
fn validate_prerequisites(tasks: &[Task]) -> Result<(), Error<MissingPrerequisiteError>> {
    let task_ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
    
    for task in tasks {
        for prereq_id in &task.prerequisites {
            if !task_ids.contains(prereq_id) {
                return Err(MissingPrerequisiteError {
                    task_id: task.id.clone(),
                    missing_prerequisite: prereq_id.clone()
                });
            }
        }
    }
    
    Ok(())
}
```

### Rule 3: Phase Consistency

Tasks in later phases cannot depend on tasks in earlier phases that are not already complete. This ensures incremental development approach.

**Phase Order:**
1. Phase 1: Core Engine (T001-T017)
2. Phase 2: The Shell (T020-T029)
3. Phase 3: The Editor (T030-T036)
4. Phase 4: Ecosystem (T040-T092)

**Validation:**
```rust
fn validate_phase_consistency(tasks: &[Task]) -> Result<(), Error<PhaseConsistencyError>> {
    for task in tasks {
        for prereq_id in &task.prerequisites {
            if let Some(prereq_task) = get_task(prereq_id) {
                if prereq_task.phase > task.phase {
                    return Err(PhaseConsistencyError {
                        task_id: task.id.clone(),
                        invalid_prerequisite: prereq_id.clone(),
                        expected_phase: task.phase,
                        actual_phase: prereq_task.phase
                    });
                }
            }
        }
    }
    
    Ok(())
}
```

## Dependency Management Strategies

### Strategy 1: Topological Sort Execution

Tasks are executed in topological order to ensure all prerequisites are satisfied.

**Algorithm:**
1. Compute in-degree for all tasks
2. Initialize queue with zero in-degree tasks
3. Process tasks in order:
   - Pop task from queue
   - Add to sorted list
   - Decrement in-degree of dependent tasks
   - Add dependent tasks with zero in-degree to queue
4. Repeat until queue is empty

### Strategy 2: Parallel Execution

Independent tasks (no dependencies on each other) can be executed in parallel by different developers or CI/CD jobs.

**Parallel Task Detection:**
```rust
fn find_parallel_tasks(sorted_tasks: &[Task]) -> Vec<Vec<Task>> {
    let mut parallel_groups = Vec::new();
    let mut current_group = Vec::new();
    
    for task in sorted_tasks {
        let dependencies_met = current_group.iter()
            .all(|t| task.prerequisites.contains(&t.id));
        
        if dependencies_met {
            current_group.push(task);
        } else {
            if !current_group.is_empty() {
                parallel_groups.push(current_group.clone());
                current_group = vec![task];
            }
        }
    }
    
    if !current_group.is_empty() {
        parallel_groups.push(current_group);
    }
    
    parallel_groups
}
```

### Strategy 3: Dependency Batching

For tasks with many dependencies, implement batching to reduce context switching overhead.

**Example:**
```toml
[task.T027]
name = "HTTP API Endpoints Implementation"
estimated_effort_hours = 48
prerequisites = ["T009", "T016", "T024"]
```

This task has 3 prerequisites. Instead of context-switching 3 times, allocate a 48-hour continuous block to complete all dependencies first.

### Strategy 4: Critical Path Prioritization

Allocate resources to tasks on the critical path first to minimize project duration.

**Critical Path Analysis:**
1. Identify all tasks with no dependent tasks (sinks)
2. Trace backward through dependencies to find longest path
3. Assign senior developers to critical path tasks
4. Monitor critical path tasks daily

## Dependency Tracking

### Task Status Tracking

Each task tracks:
- **Status:** Pending, In Progress, Blocked, Completed
- **Start Date:** When execution began
- **End Date:** When execution completed
- **Actual Effort:** Hours actually spent vs. estimated
- **Blockers:** Current blocking issues

### Dependency Status Tracking

For each task dependency:
- **Dependency Task ID:** TXXX
- **Status:** Pending, In Progress, Completed
- **Completion Date:** When the dependency was completed
- **Issues:** Any problems with the dependency

## Risk Management

### Dependency Risk Assessment

Dependencies are categorized by risk level:

**High Risk Dependencies:**
- T009 (JIT Compiler) - Complex, high effort
- T015 (Tantivy Indexer) - Third-party dependency
- T034 (Authentication) - Security-critical
- T035 (RBAC Middleware) - Security-critical

**Medium Risk Dependencies:**
- T001 (Markdown Parser) - External dependency
- T003 (File Watcher) - Platform-specific
- T013 (LRU Cache) - Performance-sensitive
- T016 (Search Query Engine) - Complex algorithm

**Low Risk Dependencies:**
- T006 (Asset Management) - Simple implementation
- T050 (Metrics Collection) - Non-blocking
- T051 (Logging Infrastructure) - Non-blocking

### Dependency Failure Mitigation

**Mitigation Strategies:**
1. **Fallback Implementations:** If a dependency fails, provide alternative approach
2. **Mock Implementation:** Develop mock implementations for testing without dependencies
3. **Feature Flags:** Disable features blocked by failed dependencies
4. **Parallel Development:** Work on independent tasks while waiting for blocked dependencies

## References

- Master Plan: `.adrs/
- ADR-072: Execution Graph Architecture
- ADR-074: Verification Criteria Definition
- ADR-075: Risk Mitigation Strategy
- ADR-076: Traceability Matrix Update
- IEEE 1016-2009: Software Design Descriptions
- ISO/IEC 25010: Software Quality

## Related Decisions

- ADR-072: Execution Graph Architecture
- ADR-074: Verification Criteria Definition
- ADR-075: Risk Mitigation Strategy
- ADR-076: Traceability Matrix Update

## Status

**Status:** ACCEPTED
**Date:** 2026-02-12
**Reviewers:** Project Manager, Technical Lead
**Next Review:** After Phase 8 completion and initial execution
