---
name: full-stack-playbook
description: |
  Playbook for full-stack web application missions. Provides guidance on vertical slice milestones,
  walking skeleton, frontend/backend workers, and quality enforcement.
  Use for CRUD apps, dashboards, e-commerce sites, and similar projects with distinct frontend and backend layers.
---
 
# Full-Stack Web Application Playbook

This playbook guides you through executing a full-stack web application mission. Use this for CRUD apps, dashboards, e-commerce sites, and similar projects with distinct frontend and backend layers.

## Milestone Strategy: Vertical Slices

Structure your milestones as **vertical slices** of functionality, not horizontal layers.

**Good milestones:**
- "user-auth" (login, signup, sessions - full stack)
- "product-catalog" (listing, search, detail pages - full stack)
- "checkout" (cart, payment, confirmation - full stack)

**Bad milestones:**
- "all-api-endpoints" (horizontal - can't test in isolation)
- "frontend-pages" (horizontal - can't test without backend)

Each milestone should leave the app in a coherent, testable state where a user can complete a meaningful flow.

## Walking Skeleton First

Before any real feature implementation, establish a **walking skeleton** that spans the full user-facing surface with the thinnest possible implementation.

**Why:** This lets work-scrutiny-validator determine testing needs and user-testing-validator exercise the complete surface from the start. Prevents building features in isolation only to discover integration issues later.

**What to include:**
- All planned UI pages (can be stubs/placeholders)
- All planned API endpoints (can return mock data)
- All planned CLI commands (can be no-ops)
- Basic routing and navigation

**Skeleton milestone should be first.** Once the skeleton exists, subsequent milestones fill in real functionality.

## Worker Types for Full-Stack

### frontend-worker

- Implements UI features (pages, components, state)
- **TDD: Write component/integration tests FIRST (before any implementation)**
- **MUST do manual browser verification with agent-browser CLI:**
- **Visual quality is essential** - check for:
  - Placement and alignment issues
  - Z-index problems (overlapping, hidden elements)
  - Overflow and content clipping/scroll issues
  - Inconsistent margins/padding
  - Missing states (hover/focus/disabled/loading/empty/error)
- **Fix issues found:**
  - Issues with own work (including from manual testing) → must fix
  - Manageable existing issues under their skill → fix them
  - Large scope or outside their skill → report to orchestrator
  - Include any fixes in whatWasImplemented

### backend-worker

- Implements API endpoints and services
- **TDD: Write API tests FIRST (before any implementation)**
- Verifies actual endpoint behavior (not just tests passing)
- **Fix issues found:**
  - Issues with own work (including from manual testing) → must fix
  - Manageable existing issues under their skill → fix them
  - Large scope or outside their skill → report to orchestrator
  - Include any fixes in whatWasImplemented

## Quality Enforcement Flow

```text
1. Orchestrator creates implementation features grouped by milestone
2. Implementation workers build features (TDD + manual verification)
3. When milestone X completes → system injects work-scrutiny-validator
4. Work-scrutiny validates handoffs match diffs, re-runs tests
5. Work-scrutiny determines what user testing is needed and creates testing features
6. User-testing validates user flows via browser (if UI) or appropriate surface
7. Failed validation surfaces bugs → orchestrator creates fix features
8. Repeat until milestone passes, then move to next milestone
```

## Common Pitfalls

1. **Building backend without frontend** - Leads to API designs that don't match UI needs. Build vertical slices instead.

2. **Skipping the skeleton** - Causes integration issues late in the mission. Always start with skeleton.

3. **Features too large** - "Build the product page" is too big. Break into: "product list", "product detail", "product search", etc.

4. **Forgetting error states** - Workers often implement happy path only. expectedBehavior should include error cases.

5. **Not testing cross-feature interactions** - Login affects what's visible in the catalog. Test these connections.

6. **No lasting test infrastructure** - Per-worker TDD produces unit/integration tests, but consider whether the mission also needs dedicated features for shared test fixtures, seed data, or e2e test suites - especially when building on an existing codebase that already has e2e coverage.

## Example Milestone Breakdown

For an e-commerce app:

**Milestone: skeleton**
- Stub all routes (home, catalog, product, cart, checkout, auth)
- Stub all API endpoints
- Basic navigation between pages

**Milestone: user-auth**
- Login page + endpoint
- Signup page + endpoint
- Session management
- Protected route handling

**Milestone: product-catalog**
- Product list page + endpoint
- Product detail page + endpoint
- Product search + filtering

**Milestone: checkout**
- Cart management (add, remove, update quantity)
- Checkout flow (shipping, payment)
- Order confirmation

### Example: backend-worker Skill

````markdown
---
name: backend-worker
description: Implement backend features including API endpoints, services, and business logic.
---

# Backend Worker

## When to Use This Skill

Features involving API endpoints, backend services, database operations, or server-side logic.

## Work Procedure

### 1. Understand the Feature

Read the feature's description, expectedBehavior, and preconditions.
Identify endpoints/services to create, input/output shapes, and error cases.

### 2. Write Tests First (TDD)

Before implementing, write tests that define expected behavior:

- API tests: request/response shapes, status codes, error responses
- Unit tests: service logic, edge cases

Run tests - they should fail (red).

### 3. Implement

Write minimum code to make tests pass. Follow patterns in AGENTS.md.
Run tests - they should pass (green).

### 4. Manual Verification

Tests passing isn't enough. Actually call the endpoints:

- Make real HTTP requests (curl, httpie, or similar)
- Test error cases manually
- Check logs for unexpected errors

### 5. Fix Issues Found

- Issues with your own work (including from manual verification) → must fix
- Manageable existing issues that fall under your skill → fix them
- Large scope issues or outside your skill → report to orchestrator in discoveredIssues

Include any fixes in whatWasImplemented.

### 6. Run Verification Steps

Execute each step in the feature's verificationSteps array.

## Handoff Requirements

Use the EndFeatureRun tool with structured handoff. Here's what thorough work looks like:

### Example Handoff (Product Search Endpoint)

```json
{
  "whatWasImplemented": "GET /api/products/search endpoint with query parameter, relevance sorting, pagination via cursor, and input validation requiring minimum 2-character queries. Also fixed existing bug in GET /api/products/:id that was returning soft-deleted products.",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      {
        "command": "npm test -- --grep 'product search'",
        "exitCode": 0,
        "observation": "4 tests passed: 'returns matching products sorted by relevance', 'returns empty array for no matches', 'paginates with cursor', 'rejects query under 2 chars'"
      },
      {
        "command": "curl 'http://localhost:3000/api/products/search?q=wireless+headphones&limit=5'",
        "exitCode": 0,
        "observation": "returned 5 products, first was Sony WH-1000XM5 with score 0.94, response included cursor eyJvZmZzZXQiOjV9"
      },
      {
        "command": "curl 'http://localhost:3000/api/products/search?q=xyznonexistent'",
        "exitCode": 0,
        "observation": "returned {\"results\":[],\"message\":\"No products found matching 'xyznonexistent'\"}"
      },
      {
        "command": "curl 'http://localhost:3000/api/products/search?q=a'",
        "exitCode": 0,
        "observation": "returned 400 with {\"error\":\"Query must be at least 2 characters\",\"code\":\"INVALID_QUERY\"}"
      }
    ],
    "interactiveChecks": []
  },
  "tests": {
    "added": [
      {
        "file": "tests/api/product-search.test.ts",
        "cases": [
          {
            "name": "returns matching products sorted by relevance",
            "verifies": "basic search returns results ordered by score descending"
          },
          {
            "name": "returns empty array with message for no matches",
            "verifies": "graceful handling when nothing matches query"
          },
          {
            "name": "paginates results with cursor",
            "verifies": "cursor-based pagination for large result sets"
          },
          {
            "name": "returns 400 for query under 2 characters",
            "verifies": "input validation rejects too-short queries"
          }
        ]
      }
    ],
    "coverage": "search query handling, relevance sorting, pagination, input validation, empty results"
  },
  "discoveredIssues": [
    {
      "severity": "suggestion",
      "description": "Response time ~800ms for common terms like 'phone' - requires DB indexing or caching layer (outside feature scope)",
      "suggestedFix": "Consider adding search index or caching layer"
    }
  ]
}
```

## When to Return to Orchestrator

- Requirements are ambiguous or contradictory
- Existing bugs affect this feature
- Scope is larger than expected (e.g., requires unmentioned migration)
- Design decision affects other features
```
