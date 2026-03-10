---
name: rfspec
version: 1.3.0
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

| Task                     | Action                                                 |
| ------------------------ | ------------------------------------------------------ |
| Generate competing specs | `/rfspec <prompt>` (background)                        |
| Poll for results         | Check `<run_dir>/done` sentinel                        |
| Pick one result          | Select via AskUser after comparison                    |
| Synthesize results       | Combine strongest elements when user chooses synthesis |
| Save final spec          | Write to `specs/active/YYYY-MM-DD-<slug>.md`           |

## Workflow

The `/rfspec` command spawns three `droid exec` calls in parallel. These take
several minutes, far exceeding the Execute tool timeout. You MUST use the
fire-and-forget + poll pattern.

### Step 1 -- Launch (background)

Run the command with `fireAndForget=true`:

```
Execute: /rfspec <user's prompt>
  fireAndForget: true
```

The script immediately prints `RFSPEC_RUN_DIR=<path>` to its log file.
Read the log file (path printed by Execute) to capture the run directory.

### Step 2 -- Poll for completion

Tell the user the models are running and you will check back. Then poll:

```
Execute: cat <run_dir>/done 2>/dev/null || echo "PENDING"
```

Poll every 30-60 seconds. The sentinel contains `STATUS=complete` or
`STATUS=failed`. While waiting, you can do other work or let the user know
progress.

### Step 3 -- Read results

Once `done` exists, read the results:

```
Read: <run_dir>/results.md
```

This file contains all three model outputs as markdown sections (Option A, B, C).

### Step 4 -- Evaluate and present

Evaluate the results -- see [references/evaluation-guide.md](references/evaluation-guide.md).
Present the choice to the user via AskUser.

### Step 5 -- Finalize

Present the selected or synthesized spec via ExitSpecMode for user review.
Save to `specs/active/` only after the user approves in spec mode.

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

1. Execute `/rfspec add a dark mode toggle ...` with `fireAndForget=true`
2. Read the background log to get `RFSPEC_RUN_DIR`
3. Tell user: "Models are running, I'll check back shortly."
4. Poll `<run_dir>/done` until `STATUS=complete`
5. Read `<run_dir>/results.md`, compare Options A, B, C
6. Present choice via AskUser
   Result: User picks Option B, saved to `specs/active/2026-03-06-dark-mode-toggle.md`

Example 2: User wants synthesis
User says: "rfspec this: refactor the auth module to use JWT"
Actions:

1. Launch background, poll for completion
2. Read results, compare -- Option A has better token rotation, Option C has cleaner middleware
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
