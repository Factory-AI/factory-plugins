---
description: Plan and record a demo video of a feature or PR
argument-hint: '"<PR-number> [-- notes]" or "<description of what to demo>"'
---

Load skills: **droid-control**.

## Parse Arguments

`$ARGUMENTS` can be:
- **PR reference** (`11386`, `#11386`, `pr-11386`, full GitHub URL) with optional `-- notes` after
- **Free-text description** of what to demo

If a PR reference is found, fetch the PR description, diff, and linked ticket via `gh pr view`. The `-- notes` narrow scope or set constraints.

Determine the deliverable requirements from the arguments. These are **commitments**, not suggestions -- every checked item must be present in the final output:

- [ ] **Layout**: side-by-side (if "compare" / "before and after" or default for fixes) OR single-branch (if "single branch" / "new feature")
- [ ] **Showcase wrapping**: YES if "showcase", "polished", "hero", "landing page", "social", or "marketing" appears
- [ ] **Keystroke overlay**: YES if "keys", "keystrokes", or "key combos" appears (implies showcase wrapping)
- [ ] **Effects tier**: one of three tiers (default to **utilitarian** for demos, **full** for showcase; only **none** if user explicitly opts out)

Effects tiers:

| Tier | What's included | When |
|---|---|---|
| **utilitarian** | Zoom for readability, keystroke overlay for user actions | Default for all demos |
| **full** | All effect types -- spotlight, zoom, callout, keystroke overlay | Default when showcase wrapping is committed |
| **none** | No effects | Only if user explicitly says "no effects" |

Do not plan specific effects here -- that's a compose-time decision made after capture, when you have actual recordings to work with.

If showcase wrapping is committed, resolve the **preset** using the first matching rule:

| User keywords | Preset |
|---|---|
| `factory`, `official`, `branded` | `factory` |
| `factory hero`, `factory landing` | `factory-hero` |
| `hero`, `landing page`, `social`, `marketing` | `hero` |
| `presentation`, `slides`, `deck` | `presentation` |
| `minimal`, `inline`, `docs embed` | `minimal` |
| _(none of the above — e.g., just "showcase" or "polished")_ | `macos` |

State these commitments (including the resolved preset) in the demo plan. Present the plan and **wait for user approval** before recording.

## Understand What to Prove

For each change in the PR, ask: what could a viewer confuse this with?

- A new mode could look like an existing mode renamed
- A fix could look like the bug not being triggered
- A performance gain is invisible without a timing reference

Design the demo so the viewer sees something that **only happens if the feature works as claimed**. Both states (before/after, old/new, input/result) must appear on screen -- off-camera verification doesn't count.

For simple PRs this is one sentence. For complex ones, sketch a brief table:

| Claim | Confused with | What proves it |
|-------|--------------|----------------|
| ... | ... | ... |

## Load Skills

Use the **droid-control** routing tables. Do all three lookups:

1. **Target route** -- find the row matching your target, load listed driver/target skills
2. **Stage route** -- load **capture** + **compose** + **verify** (demos always need all three)
3. **Artifact route** -- if showcase or keystroke overlay was committed, also load **showcase**

## Plan the Interaction Script

Script the sequence for each branch:

1. Launch and establish visible baseline
2. Exercise the feature
3. Show the disambiguating result (the thing that rules out the null hypothesis)
4. Stress/edge case if visually interesting
5. Clean exit

For comparison demos, both branches run **identical interactions** -- only the behavior differs. Verify state between steps; don't blindly fire the next key.

## Capture

Follow the **capture** atom. It owns recording lifecycle, pre-flight, keystroke logging, and evidence collection.

Provide the capture stage with:
- Target app and branch(es)
- The interaction script from above
- Whether to emit a keystroke TSV

**Delegation:** For before/after comparisons, capture both branches **in parallel** using worker subagents with `run_in_background=true`. Construct the exact `tctl` commands for each worker (see the delegation section in the droid-control skill). Wait for both to complete before proceeding to compose.

## Compose

Follow the **compose** atom. It owns the full video assembly pipeline.

**Delegation:** Launch one worker subagent for the mechanical render:
- Worker A: render the final video via `render-showcase.sh` directly from `.cast` / `.mp4` inputs

`render-showcase.sh` owns `.cast -> agg -> .mp4`, Remotion composition, fidelity profile selection, duration detection, and cleanup. Wait for the worker to finish, then verify the output.

Hand compose a hybrid handoff:

### Mechanical (structured)
- layout: side-by-side | single
- fidelity: auto | compact | standard | inspect (optional; auto => side-by-side=inspect, single=standard)
- labels: ["BEFORE (<baseline branch>)", "AFTER (<candidate branch or PR>)"]
- speed: 3x
- title: "PR #11386 — Add --fork flag"
- subtitle: "Demo: --fork creates a forked session from current context"
- clips: [/tmp/before.cast, /tmp/after.cast]
- keys: /tmp/keys.tsv (if committed)
- preset: hero | macos | minimal | presentation | factory | factory-hero (if committed)
- effects tier: utilitarian | full | none
- output: /tmp/demo-pr-11386.mp4

### Creative (natural language)
What the viewer should take away. Which moments to hold. How to frame the story. Whether phase cards are warranted. The compose atom uses this -- along with the effects tier -- for editorial decisions: title card phrasing, trim points, emphasis, and choosing specific effects to apply.

## Verify

Follow the **verify** atom. It checks the final deliverable against the commitments from Parse Arguments.

## Report

- File path, resolution, duration, size
- What each phase demonstrates -- map back to the claims. Call out timestamps where the disambiguating evidence appears.
- What's not covered and why.
