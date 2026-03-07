---
name: rfspec
version: 1.2.0
description: |
  Multi-model spec generation and synthesis. Use when the user wants to:
  - Get competing proposals from different AI models
  - Compare approaches to a problem from different perspectives
  - Synthesize the best parts of several proposals into one spec
  Keywords: rfspec, competing specs, multi-model, compare approaches,
  multiple perspectives, request for spec, fan out, model comparison.
  NOT for: single-model generation, code review, or running tests.
---

# rfspec -- Request for Spec

Fan out a prompt to multiple models, compare their responses, and help the user pick or synthesize the best result.

## Quick Reference

| Task | Action |
|------|--------|
| Generate competing specs | `/rfspec <prompt>` |
| Pick one result | Select via AskUser after comparison |
| Synthesize results | Combine strongest elements when user chooses synthesis |
| Save final spec | Write to `specs/active/YYYY-MM-DD-<slug>.md` |

## Workflow

1. Run `/rfspec <user's prompt>` -- fires parallel model calls, returns labeled options (A, B, C).
2. Evaluate the results -- see [references/evaluation-guide.md](references/evaluation-guide.md).
3. Present the choice to the user via AskUser.
4. Present the selected or synthesized spec via ExitSpecMode for user review.
5. Save to `specs/active/` only after the user approves in spec mode.

## Saving

**Do not save immediately.** After the user picks or synthesis is complete, present the
final spec via ExitSpecMode for review. Only after approval, save to:

```
specs/active/YYYY-MM-DD-<slug>.md
```

Where `<slug>` is a short kebab-case name derived from the topic.

## Pitfalls

- Don't summarize each option individually -- compare them against each other.
- Don't concatenate when synthesizing -- resolve contradictions and produce a coherent document.
- If all options are rejected, gather feedback and re-run with a refined prompt.

## Verification

After saving a spec:

1. Confirm the file exists at the expected path.
2. Verify it contains the selected or synthesized content.
3. Report the saved path to the user.

## Examples

Example 1: User wants competing specs
User says: "Get me specs from multiple models for adding a dark mode toggle"
Actions:

1. Run `/rfspec add a dark mode toggle to the settings page with persistent user preference`
2. Read Options A, B, C
3. Compare: "Option A uses CSS variables with a React context, Option B uses Tailwind's dark class with localStorage, Option C uses a theme provider with system preference detection."
4. Present choice via AskUser
Result: User picks Option B, saved to `specs/active/2026-03-06-dark-mode-toggle.md`

Example 2: User wants synthesis
User says: "rfspec this: refactor the auth module to use JWT"
Actions:

1. Run `/rfspec refactor the auth module to use JWT`
2. Compare results, noting Option A has better token rotation but Option C has cleaner middleware
3. User selects "Synthesize"
4. Combine Option A's rotation logic with Option C's middleware structure
Result: Synthesized spec saved to `specs/active/2026-03-06-auth-jwt-refactor.md`

Example 3: All options rejected
User says: "None of these work, they all miss the caching layer"
Actions:

1. Ask what's missing -- user explains the Redis caching requirement
2. Offer to re-run: `/rfspec refactor auth module to use JWT with Redis session caching`
Result: New round of specs generated with caching addressed

## References

- [references/evaluation-guide.md](references/evaluation-guide.md) -- how to compare, synthesize, and handle rejection
- [references/troubleshooting.md](references/troubleshooting.md) -- error codes and fixes
