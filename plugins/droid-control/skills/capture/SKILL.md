---
name: capture
description: Background knowledge for droid-control workflows -- not invoked directly. Recording lifecycle for terminal and browser sessions.
user-invocable: false
---

# Capture

The orchestrator routed you here. This atom owns the full recording lifecycle: launch a target, execute an interaction script, collect raw outputs.

You should already have a **driver atom** loaded (tuistory, true-input, or agent-browser) and optionally a **target atom** (droid-cli). This atom layers the recording discipline on top.

## Inputs

The command that invoked you should have provided:

- **Target**: what to launch and on which branch(es)
- **Interaction script**: the sequence of actions to perform
- **What to capture**: recordings (.cast/.mp4), screenshots, text snapshots, byte sequences
- **Keystroke logging**: whether to emit a keystroke TSV for later overlay

## Recording lifecycle

### 1. Pre-flight

Before recording anything:

- Terminal size is consistent across all sessions (`--cols 120 --rows 36`)
- **Browser viewport size matches the composition layout** (see "Browser viewport sizing" below) — mismatched aspects letterbox in the final video
- Branch/worktree paths and env vars are correct
- Recording format matches the driver: `.cast` for tuistory, `.mp4` for true-input, screenshots for agent-browser
- If comparing branches, both sessions use identical terminal / viewport dimensions and launch parameters
- For `droid-dev` captures, `--repo-root` is **mandatory** — `tctl` will refuse to launch without it
- **Color env vars are set** (see below)

### Browser viewport sizing

Panel aspect ratio in the final composition is **layout-dependent**. At the default 1920×1080 output with factory preset margins, the window-chrome panels that clips render into come out roughly:

| Layout | Panel aspect | Recommended browser viewport |
|---|---|---|
| `single` | ~1760×920 (≈16:9 landscape) | **1280×720** or **1440×810** |
| `side-by-side` | ~872×920 per panel (≈8:9, near-square / slight portrait) | **960×1000**, **900×1000**, or **1024×1080** |

Feeding a 16:9 landscape recording into a near-square side-by-side panel triggers `objectFit: "contain"` letterboxing — you get a thin strip of content with giant black bars above and below. Two ways to avoid it:

1. **Match aspects at capture time** (preferred) — pick the viewport from the table above based on the committed layout.
2. **Opt into cropping at compose time** — pass `"objectFit": "cover"` in showcase props. Crops the edges of the clip instead of letterboxing. Use when the relevant UI is centered and the clip's edges are expendable.

If you're unsure of the layout when capturing, default to `960×1000` — it is workable in both layouts (slight horizontal letterbox in `single`, no letterbox in `side-by-side`).

```bash
TCTL=${DROID_PLUGIN_ROOT}/bin/tctl
# RUN_ID and RUN_DIR should already be set by the parent (see droid-control ground rule 5)
```

### 2. Launch and record

**CRITICAL: tuistory's virtual PTY does not advertise color support by default.** Node.js apps (Ink/chalk) detect this and suppress ALL color escape codes, producing a monochrome recording. You **must** pass `FORCE_COLOR=3` and `COLORTERM=truecolor` to force full 24-bit color output. Without these, agg has nothing to theme and the video will look grey/desaturated regardless of the agg theme chosen.

**Single branch:**
```bash
$TCTL launch "droid-dev" -s ${RUN_ID}-demo --backend tuistory \
  --repo-root /path/to/worktree \
  --cols 120 --rows 36 --record ${RUN_DIR}/demo.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor
```

**Comparison (before/after):**
```bash
$TCTL launch "droid-dev" -s ${RUN_ID}-before --backend tuistory \
  --repo-root /path/to/baseline-worktree \
  --cols 120 --rows 36 --record ${RUN_DIR}/before.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor

$TCTL launch "droid-dev" -s ${RUN_ID}-after --backend tuistory \
  --repo-root /path/to/candidate-worktree \
  --cols 120 --rows 36 --record ${RUN_DIR}/after.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor
```

**Browser:** size the viewport to match the composition layout (see table above).

```bash
# side-by-side layout → near-square panel
agent-browser open <url> --viewport 960x1000
agent-browser record start ${RUN_DIR}/demo.webm

# single layout → 16:9 panel
agent-browser open <url> --viewport 1280x720
agent-browser record start ${RUN_DIR}/demo.webm
```

### 3. Execute the interaction script

Film for a viewer with no context. You are a director, not an operator.

- **Record before setup** -- the baseline state is act 1.
- **Hold after state changes** -- 2-3 seconds so text is readable. Use `snapshot --trim` as natural verification beats.
- **Verify between steps** -- `wait` or `snapshot` to confirm state before proceeding. Don't blindly fire the next key.
- **Verification IS evidence.** A snapshot that shows nothing changed after pressing ESC proves the session is frozen. A snapshot that shows an error message proves the command was blocked. Always snapshot after actions where the *absence* of a response is the point -- the viewer needs to see it too.

For comparison recordings, both branches run **identical interactions** -- only the behavior differs.

### 4. Keystroke logging

If the workflow requires keystroke overlay, emit a TSV file during recording. Since every interaction is scripted, the timing data is already known.

Write each keystroke's timestamp (seconds from recording start) and a human-readable label:

```
0.5	droid --fork
1.2	Enter
2.8	Ctrl+C
4.0	Esc
```

Use readable key names (`Ctrl+C`, not `\x03`). Save alongside the recording (e.g., `/tmp/keys.tsv`).

### 5. Close and verify raw outputs

```bash
$TCTL -s demo close    # finalizes the .cast / stops recording
```

Before handing off, confirm every expected output file exists and is non-empty:
- Recording files (.cast, .mp4, .webm)
- Screenshot files (.png)
- Keystroke TSV (if committed)
- Text snapshot logs (if needed for the report)

## Evidence capture patterns

| Proof type | How to capture |
|---|---|
| Functional behavior | Text snapshots: `$TCTL -s <name> snapshot --trim` |
| Visual rendering | Screenshots: `$TCTL -s <name> screenshot -o /tmp/proof-N.png` |
| Keyboard encoding | PTY bytes: `${DROID_PLUGIN_ROOT}/scripts/capture-terminal-bytes.py --backend <terminal> --combo <keys>` |
| Web/Electron | Screenshots: `agent-browser screenshot --annotate /tmp/proof-N.png` |
| Before/after | Run the same sequence on both branches at the same capture points |

## Outputs

Hand these to the **compose** stage:

```
## Capture outputs
- clips: [/tmp/before.cast, /tmp/after.cast]
- screenshots: [/tmp/proof-1.png, /tmp/proof-2.png]
- keys: /tmp/keys.tsv (if keystroke logging was requested)
- driver: tuistory | true-input | agent-browser
- terminal_size: 120x36          # for tuistory / true-input
- viewport: 960x1000             # for agent-browser; report so compose knows the clip aspect
```

## Recovery

If a session gets stuck mid-recording:

```bash
$TCTL -s <name> press esc         # bail out of stuck dialog
$TCTL -s <name> snapshot --trim   # check visible state
$TCTL -s <name> close             # hard reset
```

For browser: `agent-browser close`.

Then re-launch and re-record. Partial recordings are not usable.
