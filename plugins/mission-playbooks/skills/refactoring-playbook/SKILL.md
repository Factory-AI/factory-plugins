---
name: refactoring-playbook
description: |
  Playbook for code modernization, architecture migrations, and large-scale refactoring.
  Provides guidance on characterization testing, strangler fig pattern, incremental migration,
  and behavior preservation. Use for migrations, modernization, and dependency upgrades.
---

# Refactoring & Migration Playbook

This playbook guides missions involving code modernization, architecture migrations, dependency upgrades, or large-scale refactoring. The core challenge: **change implementation while preserving behavior**.

## Key Principle: Tests Before Changes

Refactoring without tests is just changing code and hoping. Before modifying any code:
- If tests exist: ensure they pass and cover the behavior you're changing
- If tests are missing: add characterization tests that capture current behavior first

## Milestone Strategy: Incremental Safe Transformation

Structure milestones around safe transformation phases:

- **characterization** - Add tests capturing current behavior (if missing)
- **scaffold** - Set up new patterns/infrastructure alongside old (strangler fig)
- **migrate-batch-N** - Migrate components incrementally, tests pass after each batch
- **cutover** - Switch to new implementation, remove old code
- **cleanup** - Remove scaffolding, polish

Each batch is one milestone. Never "big bang" - always small, verifiable steps where tests pass after each commit.

## Worker Types

Refactoring missions use workers that coordinate through shared state in `.factory/library/`.

### characterization-worker

Adds tests for existing behavior before any changes.

1. Read `migration-plan.md` to understand what's being migrated
2. Identify code paths that lack test coverage
3. Write characterization tests that capture current behavior (not ideal behavior)
4. Update `migration-status.md` with test coverage status

Tests must pass against current code. These tests become the safety net for migration.

### scaffold-worker

Sets up new infrastructure to coexist with old (strangler fig pattern).

1. Read `migration-plan.md` for target architecture/patterns
2. Create new modules/infrastructure alongside existing code
3. Set up adapters/facades so old code can gradually switch to new
4. Update `migration-status.md` with scaffold status

Old tests must still pass. New infrastructure should be testable but not yet used.

### migration-worker

Migrates specific components from old to new implementation.

1. Read `migration-status.md` to find next component to migrate
2. Migrate ONE component/module (keep scope small)
3. Update call sites to use new implementation
4. Run full test suite - must pass before completing
5. Update `migration-status.md` marking component as migrated

Each migration is one atomic commit. If tests fail, fix or revert - never leave broken.

### verification-worker

Ensures behavior is preserved across the migration.

1. Read `migration-status.md` to understand what changed
2. Run full test suite including characterization tests
3. Perform manual verification of migrated functionality
4. Compare old vs new behavior for edge cases
5. Document any behavioral differences found

## Information Flow

```text
characterization-worker ──writes──▶ migration-status.md (test coverage)
                                            │
                                            ▼ reads
scaffold-worker ──writes──▶ migration-status.md (scaffold ready)
                                            │
                                            ▼ reads
migration-worker ──writes──▶ migration-status.md (component X migrated)
                                            │
                                            ▼ reads
verification-worker ──writes──▶ migration-status.md (batch verified)
```

Each worker:
1. Reads migration-plan.md and migration-status.md before starting
2. Ensures all tests pass before marking work complete
3. Updates migration-status.md after completing

## Orchestrator Setup

Before starting migration, create:

1. **`.factory/library/migration-plan.md`**:
   - Current state (what exists now)
   - Target state (what we're migrating to)
   - Scope (what's included, what's explicitly excluded)
   - Approach (strangler fig, parallel run, etc.)
   - Risk areas (complex logic, external dependencies)

2. **`.factory/library/migration-status.md`**:
   - Components list with status (pending, in-progress, migrated, verified)
   - Test coverage status
   - Scaffold status
   - Issues/blockers discovered

3. **`.factory/services.yaml`** - Ensure `test` command runs full suite

## Feature Structure

### Characterization Phase
```
characterize-<area>  (characterization-worker) - Add tests for <area>
```

### Scaffold Phase
```
scaffold-<component> (scaffold-worker) - Set up new <component> alongside old
```

### Migration Batches
```
migrate-<component>  (migration-worker) - Migrate <component> to new implementation
verify-batch-N       (verification-worker) - Verify batch N preserves behavior
```

### Cutover & Cleanup
```
cutover-<area>       (migration-worker) - Remove old <area>, switch fully to new
cleanup-<area>       (migration-worker) - Remove adapters, polish
```

## Example Worker Skill: migration-worker

```markdown
---
name: migration-worker
description: Migrate components from old to new implementation incrementally.
---

# Migration Worker

## Procedure

1. **Read status** - Check `migration-plan.md` and `migration-status.md`. Identify your assigned component. Return to orchestrator if scaffold not ready.

2. **Understand the component** - Read current implementation. Identify all call sites. Note edge cases and error handling.

3. **Migrate incrementally**:
   - Update component to use new patterns/infrastructure
   - Update call sites one at a time
   - Run tests after each change
   - Keep changes in atomic commits

4. **Verify** - Run full test suite. All tests must pass.

5. **Update status** in `migration-status.md`:
   ```
   ## UserService
   
   **Status:** MIGRATED
   **Commit:** abc123
   **Changes:** Migrated from class to functional, now uses new data layer
   **Call sites updated:** 12
   **Tests:** All 47 tests pass
   ```

## Example Handoff

{
  "salientSummary": "Migrated UserService to the functional pattern and updated 12 call sites; ran `npm test` (47 passing) and updated `.factory/library/migration-status.md` with the MIGRATED status + commit.",
  "whatWasImplemented": "Migrated UserService from class-based to functional pattern. Updated 12 call sites. All 47 existing tests pass.",
  "verification": {
    "commandsRun": [
      {"command": "npm test", "exitCode": 0, "observation": "47 tests pass"},
      {"command": "grep 'Status: MIGRATED' .factory/library/migration-status.md", "exitCode": 0, "observation": "Status updated"}
    ]
  }
}

## Return to Orchestrator When

- Scaffold not ready for this component
- Tests fail and fix is non-trivial
- Migration reveals architectural issue requiring plan change
- Component has undocumented dependencies
```

## Common Pitfalls

1. **Changing behavior during migration** - Refactoring changes structure, not behavior. Behavior changes are separate features.
2. **Big bang migrations** - Each commit should leave tests passing. Never batch multiple components.
3. **Skipping characterization** - Without tests capturing current behavior, you can't verify preservation.
4. **Incomplete call site updates** - Use grep/find-references to ensure all usages are updated.
5. **Not updating migration status** - Next worker needs to know what's done.
6. **Mixing refactoring with features** - Keep them separate. Refactor first, then add features.

## When to Stop

- **Complete** - All components migrated, verified, old code removed
- **Blocked** - Discovered issue requiring architectural decision
- **Scope change** - User decides to adjust what's being migrated
