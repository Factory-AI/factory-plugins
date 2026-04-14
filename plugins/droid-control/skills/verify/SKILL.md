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

## Video deliverables

### Technical checks

Run `ffprobe` on the final .mp4:

```bash
ffprobe -v quiet -print_format json -show_format -show_streams <video>
```

| Check | Pass condition |
|---|---|
| Exists and plays | ffprobe exits 0, duration > 0 |
| Resolution | 1920x1080 (or target resolution) |
| Pixel format | yuv420p |
| File size | Under 5 MB for GitHub embeds (25 MB hard limit) |
| Duration | Must fall within the compose pacing table's target range for this demo type: **30-45s** (single feature), **45-75s** (side-by-side comparison), **60-120s** (multi-phase). Below the minimum is a **failure** — re-compose with a lower speed factor or re-capture with more steps. |
| Filename | Includes PR number or meaningful identifier |

### Commitment checks

Walk through each commitment from the parse step:

| Commitment | How to verify |
|---|---|
| Title card | Video starts with a static frame showing PR info (check first 5s) |
| Side-by-side layout | Video shows two panels with a divider |
| Showcase polish | Resolution is 1920x1080, window chrome and rounded corners visible |
| Keystroke overlay | Pill overlays appear at interaction points |
| Effects | Effects matching the committed tier are present (utilitarian: zoom/keystroke; full: spotlight, zoom, callout, keystroke) |
| Speed note | Title card mentions playback speed |

### Content checks

- Every claim from the "what to prove" analysis has visible evidence in the video
- Both states (before/after, input/result) appear on screen
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
| Step coverage | Every defined test step has a result (PASS/FAIL) |
| Evidence attached | Screenshots/snapshots at every step |
| Failures documented | Failed steps have evidence and description |
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
- Resolution: 1920x1080 ✓
- Duration: 42s ✓
- Size: 3.2 MB ✓
- Format: yuv420p ✓

### Commitments
- [x] Title card with PR info
- [x] Side-by-side comparison layout
- [x] Showcase hero preset applied
- [x] Keystroke overlay visible

### Content
- [x] Fork creates independent session (visible at 0:18)
- [x] History diverges after fork (visible at 0:32)

All commitments met. Deliverable ready.
```
