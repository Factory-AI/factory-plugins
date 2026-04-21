---
description: Test a claim about behavior and report whether the evidence supports or refutes it
argument-hint: '"<claim to test>" or "<PR-number> -- <specific claim>"'
---

Load skills: **droid-control**.

## Ground rule

**You are an investigator, not an advocate.** Your job is to find out whether a claim is true, not to make it look true. A conclusive "this is broken" finding with clear evidence is just as valuable as a "this works" finding. Never fabricate, hardcode, or stage evidence to match an expected outcome. If the behavior you observe contradicts the claim, that is the result -- report it.

## Parse Arguments

`$ARGUMENTS` can be:
- **Direct claim** ("Shift+Enter inserts a newline in Ghostty", "resize fix no longer clears screen")
- **PR reference + claim** (`11386 -- the fork flag creates a new session`)
- **PR reference only** (`11386`) -- fetch the PR, identify the most important testable claim

If a PR reference is found, fetch the PR description and diff for context.

Determine commitments:

- [ ] **Evidence type**: byte capture | screenshot | text snapshot | annotated browser snapshot
- [ ] **Comparison**: before/after (if the claim is about a change) OR single-state (if the claim is about current behavior)
- [ ] **Video proof**: YES if "video", "recording", "demo" appears (implies compose stage)
- [ ] **Showcase**: YES if "polished", "showcase" appears (implies video + showcase)

If showcase is committed, resolve the **preset** using the first matching rule:

| User keywords | Preset |
|---|---|
| `factory`, `official`, `branded` | `factory` |
| `factory hero`, `factory landing` | `factory-hero` |
| `hero`, `landing page`, `social`, `marketing` | `hero` |
| `presentation`, `slides`, `deck` | `presentation` |
| `minimal`, `inline`, `docs embed` | `minimal` |
| _(none of the above)_ | `macos` |

## Understand What to Test

Determine the single specific behavior to observe. What would a skeptic need to see?

- Byte-level claim (keyboard encoding, escape sequences) → needs raw PTY capture
- Visual claim (rendering, layout, colors) → needs screenshot from a real compositor
- Functional claim (feature works, flow completes) → needs interaction + state verification

## Load Skills

Use the **droid-control** routing tables:

1. **Target route** -- find the row matching your target, load listed driver/target skills
2. **Stage route** -- load **capture** + **verify** always; load **compose** if video proof or showcase was committed
3. **Artifact route** -- if showcase committed, also load **showcase**

## Capture

Follow the **capture** atom. Provide:
- The claim to test
- The evidence type(s) needed
- The minimal interaction sequence that demonstrates the behavior
- Whether this is a before/after comparison

**If the behavior does not match the claim:** Do not retry the interaction hoping for a different result. Capture a snapshot or screenshot of the actual state. This is evidence. If you suspect your test procedure is wrong (e.g., wrong branch, missing build step), verify the environment first -- but if the environment is correct and the behavior is wrong, that is a finding, not an error on your part.

## Compose (if committed)

Follow the **compose** atom if a video deliverable was committed. Hand it:

### Mechanical
- layout: side-by-side | single
- clips: [paths to recordings]
- title: "Verify: <claim>"
- output: ${RUN_DIR}/verify-<identifier>.mp4

### Creative
What the evidence shows and why it is conclusive -- whether it supports or refutes the claim.

## Verify

Follow the **verify** atom. It checks the deliverable against your commitments.

## Report

```
## Verify: <claim>

**Environment:** <driver, terminal/browser, OS>
**Branch:** <branch name or commit>

### Evidence

<snapshots, screenshots, byte captures, or video path>

### Conclusion

**CONFIRMED** | **REFUTED** | **INCONCLUSIVE**

<one-paragraph explanation of what the evidence shows>
```

### When the claim is refuted

If the evidence shows the behavior does not match the claim:

1. State the expected behavior (from the claim)
2. State the observed behavior (from the evidence)
3. Include the evidence (snapshots, screenshots, hex dumps) inline
4. Note any environmental factors that might be relevant (branch, commit, terminal, OS)

This is a valuable finding. The user asked you to test this claim precisely because they need to know whether it holds.

### When the result is inconclusive

If the environment prevented a clean test (e.g., missing dependency, build failure, test infra crash), report what blocked the test and what would be needed to resolve it. Do not guess at the outcome.

## Do NOT

- Retry a failing test more than once without changing the environment or procedure
- Hardcode expected output, mock responses, or stage a scenario to produce a desired result
- Assume that unexpected behavior means you made a mistake -- it may be a real bug
- Omit evidence that contradicts the claim
