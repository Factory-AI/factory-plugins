---
name: tui-application-playbook
description: |
  Playbook for terminal user interface (TUI) application missions. Provides guidance on vertical slice
  milestones, walking skeleton, TUI/backend workers, tuistory-based manual verification, and quality enforcement.
  Use for interactive CLI tools, terminal dashboards, text editors, and similar TUI projects.
---

# TUI Application Playbook

This playbook guides you through executing a terminal user interface (TUI) application mission. Use this for CLI tools with interactive interfaces, terminal dashboards, text-based editors, and similar projects rendered in the terminal.

## Dependency: tuistory skill

This playbook requires the `tuistory` skill for TUI verification and testing. If the `tuistory` skill is not already available in your `<available_skills>`, install it from the `droid-evolved` plugin:

```bash
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
droid plugin install droid-evolved@factory-plugins
```

After installation, the `tuistory` skill will appear in your available skills.

## Milestone Strategy: Vertical Slices

Structure your milestones as **vertical slices** of functionality, not horizontal layers.

**Good milestones:**
- "navigation" (menu system, views, keybindings - full stack)
- "data-display" (list views, detail views, formatting - full stack)
- "editing" (input handling, validation, persistence - full stack)

**Bad milestones:**
- "all-keybindings" (horizontal - can't test in isolation)
- "rendering-layer" (horizontal - can't test without data/state)

Each milestone should leave the app in a coherent, testable state where a user can complete a meaningful flow.

## Walking Skeleton First

Before any real feature implementation, establish a **walking skeleton** that spans the full user-facing surface with the thinnest possible implementation.

**Why:** This lets work-scrutiny-validator determine testing needs and user-testing-validator exercise the complete surface from the start. Prevents building features in isolation only to discover integration issues later.

**What to include:**
- All planned views/screens (can be stubs/placeholders)
- All planned commands and keybindings (can be no-ops)
- Basic navigation between views
- Minimal rendering pipeline

**Skeleton milestone should be first.** Once the skeleton exists, subsequent milestones fill in real functionality.

## Worker Types for TUI

### tui-worker

- Implements TUI features (views, components, input handling, state)
- **TDD: Write tests FIRST (before any implementation)**
- **MUST do manual TUI verification with tuistory:**
  - Launch the app, navigate to the relevant view, and verify rendering and interactions
  - Use `tuistory snapshot --trim` to capture terminal output and verify visual correctness
  - Test keyboard interactions (`tuistory press <key>`), input handling (`tuistory type "<text>"`)
  - Check for rendering artifacts, alignment issues, overflow, and missing states
- **Fix issues found:**
  - Issues with own work (including from manual testing) → must fix
  - Manageable existing issues under their skill → fix them
  - Large scope or outside their skill → report to orchestrator
  - Include any fixes in whatWasImplemented

### backend-worker

- Implements data layer, services, and business logic that the TUI consumes
- **TDD: Write tests FIRST (before any implementation)**
- Verifies actual behavior (not just tests passing)
- **Fix issues found:**
  - Issues with own work (including from manual testing) → must fix
  - Manageable existing issues under their skill → fix them
  - Large scope or outside their skill → report to orchestrator
  - Include any fixes in whatWasImplemented

## Quality Enforcement Flow

```text
1. Orchestrator creates implementation features grouped by milestone
2. Implementation workers build features (TDD + manual verification via tuistory)
3. When milestone X completes → system injects work-scrutiny-validator
4. Work-scrutiny validates handoffs match diffs, re-runs tests
5. Work-scrutiny determines what user testing is needed and creates testing features
6. User-testing validates TUI flows via tuistory
7. Failed validation surfaces bugs → orchestrator creates fix features
8. Repeat until milestone passes, then move to next milestone
```

## Common Pitfalls

1. **Building state management without UI** - Leads to data structures that don't match rendering needs. Build vertical slices instead.

2. **Skipping the skeleton** - Causes integration issues late in the mission. Always start with skeleton.

3. **Features too large** - "Build the settings view" is too big. Break into: "settings navigation", "settings editing", "settings persistence", etc.

4. **Forgetting edge states** - Workers often implement happy path only. expectedBehavior should include empty states, error states, overflow/truncation, and resize handling.

5. **Not testing keyboard interactions** - TUI apps are keyboard-driven. Every view needs its keybindings tested, including edge cases (rapid input, conflicting shortcuts).

6. **Not verifying visually with tuistory** - Unit tests can't catch rendering issues. Workers must use tuistory to verify layout, alignment, and visual state.

7. **No lasting test infrastructure** - Per-worker TDD produces unit/integration tests, but consider whether the mission also needs dedicated features for shared test fixtures or e2e test suites using tuistory.
