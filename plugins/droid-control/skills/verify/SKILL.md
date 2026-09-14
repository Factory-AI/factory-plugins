---
name: verify
description: Background knowledge for droid-control workflows -- not invoked directly. Deliverable verification against commitments.
user-invocable: false
---

# Verify

The orchestrator routed you here. This atom checks the final deliverable against the commitments made at the start of the workflow.

## Inputs

You receive:

1. **Commitments** from the command's parse step -- the promises made about what the deliverable would contain
2. **Compose outputs** -- the finished artifact(s) and their metadata

Composition is optional: raw desktop screenshots/state and the driver's evidence handoff can be the deliverable. Verify the task's postcondition, not merely the action's exit status or `effect` field.

## Status vocabulary

Every step, check, and postcondition gets exactly one of these; commands and driver atoms use the same words.

| Status | Meaning | Counts as |
|---|---|---|
| `PASS` | The postcondition was observed | Met |
| `FAIL` | The postcondition was observed to be false | Not met — this is a finding |
| `BLOCKED` | The postcondition could not be observed: capture unavailable, permission wait unresolved, missing binary or connection, environment prevented the step | Neither. Name what blocked it and what unblocks it |

`BLOCKED` is never rolled into `PASS` or `FAIL`. A deliverable with `BLOCKED` steps is incomplete unless the user accepts the partial evidence. `/verify`'s claim verdict (`CONFIRMED` / `REFUTED` / `INCONCLUSIVE`) is about evidence sufficiency, not step status: any `BLOCKED` step that the claim depends on makes the verdict `INCONCLUSIVE`, but `INCONCLUSIVE` also covers steps that all ran and still did not decide the claim.

## Video deliverables

### Technical checks

Run `ffprobe` on the final .mp4, then a full decode:

```bash
ffprobe -v quiet -print_format json -show_format -show_streams <video>
ffmpeg -v error -xerror -i <video> -f null -   # -xerror: exit non-zero on the first decode error
```

| Check | Pass condition |
|---|---|
| Exists and decodes | ffprobe exits 0 with duration > 0, **and** the `ffmpeg -xerror` decode exits 0 with empty stderr |
| Resolution | Matches the fidelity compose resolved: 1920x1080 for `compact`/`standard`, 2560x1440 for `inspect` (the default for `side-by-side`), or the explicit `width`/`height` in the props |
| Pixel format | `pix_fmt=yuv420p` with `color_space=bt709` (a `yuvj420p` stream means the render bypassed `render-showcase.sh`) |
| File size | Under 5 MB for GitHub embeds (25 MB hard limit) |
| Duration | Equals `4s + longest_clip / speed + 3.5s` (compose's duration checkpoint) **and** falls within the compose pacing table's target range for this demo type: **30-45s** (single feature), **45-75s** (side-by-side comparison), **60-120s** (multi-phase). Below the minimum is a **failure** — re-compose with a lower speed factor or re-capture with more steps. |
| Filename | Includes PR number or meaningful identifier |

### Commitment checks

Walk through each commitment from the parse step:

| Commitment | How to verify |
|---|---|
| Title card | Video starts with a static frame showing PR info (check first 5s) |
| Side-by-side layout | Video shows two panels with a divider |
| Showcase polish | Resolution matches the resolved fidelity, window chrome and rounded corners visible |
| Keystroke overlay | Pill overlays appear at interaction points |
| Effects | Effects matching the committed tier are present (utilitarian: zoom/keystroke; full: spotlight, zoom, callout, keystroke) |
| Speed note | Title card mentions playback speed |

### Content checks

Metadata cannot show what is on screen. For every proof point, extract the frame and look at it:

```bash
ffmpeg -v error -y -ss <t> -i <video> -frames:v 1 "${RUN_DIR}/proof-<t>.png"   # then Read the PNG
```

- Every claim from the "what to prove" analysis has visible evidence in the video, confirmed in an extracted frame at the timestamp you report
- Both states (before/after, input/result) appear on screen
- The final state: clips play from `t = 4.0s` to `t = 4.0s + longest_clip / speed`, fully visible; after that the last frame is held under the outro crossfade. Extract the final proof frame just before the clips end (e.g. `t = 4.0 + longest_clip / speed - 0.1`), not from the crossfade
- In a side-by-side, a shorter clip holds its final frame; confirm the held panel shows the intended final state, not an unfinished step
- No dead time longer than 3 seconds without visible activity

## Screenshot/snapshot deliverables

### For proofs

| Check | Pass condition |
|---|---|
| Evidence exists | Screenshots/snapshots at every claimed proof point |
| Environment stated | Driver, terminal/browser, OS identified |
| Conclusion present | Evidence explicitly supports or refutes the claim |
| Before/after paired | If comparison, both branches shown at same capture points |

### For QA reports

| Check | Pass condition |
|---|---|
| Step coverage | Every defined test step has a status (`PASS` / `FAIL` / `BLOCKED`) |
| Evidence attached | Screenshots/snapshots at every step |
| Failures documented | `FAIL` steps have evidence and description; `BLOCKED` steps name the blocker and what unblocks it |
| Report structured | Markdown report follows the QA template |

## Failure handling

If any check fails:

1. Identify which stage produced the problem (capture or compose)
2. Report the specific failure: "Side-by-side layout was committed but the output is a single panel"
3. Go back to the failed stage and fix it
4. Re-verify after the fix

Do not report a deliverable as complete until every commitment is met.

## Output

```
## Verification

### Technical
- Decode: ffmpeg -xerror exit 0 ✓
- Resolution: 2560x1440 (inspect, side-by-side) ✓
- Duration: 52.5s = 4 + 45 + 3.5 ✓
- Size: 3.2 MB ✓
- Format: yuv420p / bt709 ✓

### Commitments
- [x] Title card with PR info
- [x] Side-by-side comparison layout
- [x] Showcase hero preset applied
- [x] Keystroke overlay visible

### Content
- [x] Fork creates independent session (frame at 0:18 read: new session id in header)
- [x] History diverges after fork (frame at 0:32 read: differing last messages)

All commitments met. Deliverable ready.
```
