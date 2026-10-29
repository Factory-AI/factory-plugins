---
name: property-based-testing
description: "Dramatically enhance code quality and reliability with property-based testing to help build functional ambitious features quickly."
---

# Employ property-based tests

While there may be occasional reasons to write conventional unit or integration tests, by default the overwhelming majority of automated tests should be property-based. Appropriate test coverage should land in the same commit as the corresponding production code.

## Inputs

- The change to production code or a bug report
- The high-level project goals

## Common Pitfalls

- Don't reimplement production code. Refactor production code so helper methods can be called directly from tests, potentially with extra parameters, as necessary.
- Good property-based tests afford greatly superior test coverage to an equivalent unit test, but only if the invariant they test is sensible. For this reason, they require more effort and contemplation to write well, and that extra effort and time can and should be expended. The time cost is paid back by drastically reducing time spent debugging down the road and enhancing the quality and reliability of the product.
- Think carefully about the best shape of the arbitraries to test the pertinent invariants. Avoid filtering arbitraries within reason as this slows down tests. Usually a filter can be replaced with a direct construction. Also think carefully about how the arbitrary will shrink; an arbitrary that shrinks better can save a lot of time in debugging down the road.
- Property-based tests are rare in your training data, so your first draft often will not be idiomatic. Look them over several times each to ensure they are idiomatic.
- Many bugs occur at third-party integration points, because our mental model doesn't match the real-world behavior of a third-party system. When interacting with APIs, find ways to prove that the system works as intended. For example, install an authoritative test bed or implementation of the API as a dev dependency, and ensure our integration is properly tested. Often consulting the API's documentation is the right place to start.
- If you discover additional pitfalls, please propose updates to this skill via chat to avoid falling into the same trap in the future, but never edit this skill directly.

## Bugs

### Step 1 — Reproduce Issue

Write a new or modify an existing idiomatic property-based test to cover the reported bug. Reason carefully about the system and high-level project goals. Are there other similar or related issues in the unit under test or analogous ones in other units? Propose follow-up work to the user via chat.

### Step 2 - Fix Production Code

Correct the production-code in the appropriate manner that addresses the root cause. Do not weaken test coverage to produce a green build and give a false sense of confidence. If you cannot identify the root cause, explain your findings via chat, not in the repo, and ask the user to work with you to troubleshoot the issue.

## Features

### Step 1 - Brainstorm Invariants

Brainstorm the most elegant invariants that illustrate the intended behavior of the new or modified feature. Reason carefully about the system and high-level project goals. What guarantees does the unit under test need to provide to achieve those? Clarify intended behavior with the user if there is ambiguity or cases that may not have been thoroughly thought out or considered based on recent prompts.

### Step 2 - Add Test Coverage

Enforce your best ideas as idiomatic property-based tests. The product needs to work correctly when stress tested by third parties, so adequate test coverage is essential. Think in terms of what is necessary to achieve the project's end goals, and ensure the unit under test does its part in service of those objectives. Clarify project goals and raise concerns about the unit under test's functionality or code quality if necessary.

## Report back

What properties you determined the pertinent production code must have and how you used property-based testing to prove that it does
