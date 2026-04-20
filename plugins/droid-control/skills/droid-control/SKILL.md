---
name: droid-control
description: Control terminal TUIs and web/Electron apps for testing, demos, QA, and computer-use tasks. Use when you need to automate a CLI, drive a browser, record a demo, or capture proof artifacts.
---

# Droid Control

Automate terminals and browsers. Three routing decisions, then atoms guide you the rest of the way.

## Ground rules

1. **Real apps, real environments.** Non-deterministic behavior (LLM responses, network latency, variable output) is expected. Handle it with `wait` / `wait-idle`. Never substitute fixtures or mocked data.
2. **Commit to execution.** Once you've chosen a driver, run the plan. If something fails mid-run, recover and retry -- don't re-evaluate the approach.
3. **Atoms are self-contained.** Load one and follow its mechanics. No cross-referencing needed.
4. **`tctl` is the ONLY way to launch recorded sessions.** `tctl` manages recording by wrapping `asciinema rec` around the PTY — raw `tuistory` has no recording capability and never will. Never call `tuistory launch` directly; unknown flags crash `tuistory-relay`. Always resolve `TCTL` to its absolute filesystem path before use, especially when delegating to workers (they don't inherit `${DROID_PLUGIN_ROOT}`).
5. **Isolate every run.** Multiple droids may be filming simultaneously on the same machine. Session names and output paths share a global namespace (`/tmp/tctl-sessions/`). At the start of every workflow, generate a run ID (`RUN_ID=$(date +%s)-$$` or similar) and use it as a prefix for all session names and a scoped temp directory for all output files:
   ```bash
   RUN_ID="$(date +%s)-$$"
   RUN_DIR="$(mktemp -d /tmp/droid-run-${RUN_ID}-XXXXXX)"
   # Session names: -s ${RUN_ID}-before, -s ${RUN_ID}-after
   # Output paths: ${RUN_DIR}/before.cast, ${RUN_DIR}/after.cast
   ```
   Never use bare session names like `-s demo`, `-s before`, `-s after` — they will collide with concurrent runs.

## Routing

Three independent lookups. Do all three, then load the union of skills they produce.

### 1. Target route — what are you driving?

| Target | Load these skills |
|---|---|
| Droid CLI (`droid-dev`, `droid exec`) | **droid-cli** + tuistory backend via `${DROID_PLUGIN_ROOT}/bin/tctl` |
| Droid CLI (real terminal proof) | **true-input** + **droid-cli** |
| Other terminal TUI | tuistory backend via `${DROID_PLUGIN_ROOT}/bin/tctl` |
| Other terminal TUI (real terminal proof) | **true-input** |
| Web page or Electron app | **agent-browser** |
| Raw terminal byte sequences | **true-input** + **pty-capture** |

**tuistory** is the default for terminal work. Use **true-input** only when you need real terminal rendering evidence.

### 2. Stage route — what does the workflow need?

Every workflow passes through stages. Load the atoms for each stage you'll use.

| Stage | Skill | When to load |
|---|---|---|
| Capture | **capture** | Always -- every workflow records or captures something |
| Compose | **compose** | When the deliverable is a produced artifact (video, annotated screenshots, comparison image) |
| Verify | **verify** | Always -- every deliverable gets checked against commitments |

### 3. Artifact route — does compose need polish tools?

Only relevant when **compose** is loaded.

| Artifact need | Also load |
|---|---|
| Showcase polish (window chrome, branded frame, cinematic background) | **showcase** |
| Effects and keystroke overlays | (compose handles this — they're fields in the Remotion props JSON) |

## Workflow shape

```
Command (intent + commitments)
  → Target route (load driver atoms)
  → Capture (record / screenshot / byte-capture)
  → Compose (assemble deliverable, if needed)
  → Verify (check against commitments)
  → Report
```

Commands declare **what** to produce. Atoms own **how**.

### Layout default

**Default: `single`.** One clip showing the target/final state. Pick this unless the deliverable is fundamentally a comparison.

| Case | Layout |
|---|---|
| Brand-new feature (no meaningful prior state) | `single` |
| Bug fix, single-clip proof of the working path | `single` |
| Walkthrough / tutorial / readme hero | `single` |
| Regression proof (broken vs fixed) | `side-by-side` |
| Behavior-preserving refactor (visual parity is the point) | `side-by-side` |
| User explicitly asks for a comparison | `side-by-side` |

Do not synthesize a "before" state to justify `side-by-side`. If there is no real baseline, use `single`.

## Delegation

The parent agent plans and orchestrates. Mechanical work runs in **worker subagents** via the Task tool. This keeps the parent's context clean and enables parallelism.

### What to delegate

| Task | Delegate? | Why |
|---|---|---|
| **Capture clip** (single layout) | YES | Worker runs the interaction script end-to-end and returns the `.cast` path |
| **Capture both clips** (comparison layout) | YES — `run_in_background=true` for each | Branches are independent; run in parallel |
| **Remotion render** | YES | Needs only props JSON, clip paths, output path. Runs `render-showcase.sh` (handles .cast conversion, fidelity profiles, duration detection, cleanup) |
| Planning, interaction scripting | NO — parent | Requires PR context and editorial judgment |
| Layout and prop construction | NO — parent | Requires editorial decisions about effects, timing, labels |
| Verification | NO — parent | Requires commitment context |
| Single ffprobe / file-existence check | NO — inline | Too trivial for subagent overhead |

### How to delegate

**Step 0: Resolve paths and generate a run ID.** Workers don't inherit `${DROID_PLUGIN_ROOT}`. Resolve once, paste everywhere:

```bash
TCTL="$(realpath "${DROID_PLUGIN_ROOT}/bin/tctl")"
RENDER="$(realpath "${DROID_PLUGIN_ROOT}/scripts/render-showcase.sh")"
RUN_ID="$(date +%s)-$$"
RUN_DIR="$(mktemp -d /tmp/droid-run-${RUN_ID}-XXXXXX)"
```

Use `${RUN_DIR}` for all output files (recordings, props, rendered video). Use `${RUN_ID}-` as a prefix for all session names. Never use bare names like `-s before` or hardcoded paths like `/tmp/before.cast`.

Give workers **exact commands** with the resolved absolute paths — not abstract instructions, not `tuistory`, not `${DROID_PLUGIN_ROOT}`. The parent does the thinking; the worker executes:

```
Task prompt for a capture worker:
  "Run these commands in order. Report the output file path and any errors.
   1. /abs/path/to/bin/tctl launch "droid-dev" -s 1712345678-42-before --backend tuistory \
        --repo-root /abs/path/to/baseline/worktree \
        --cols 120 --rows 36 --record /tmp/droid-run-1712345678-42-xxxx/before.cast \
        --env FORCE_COLOR=3 --env COLORTERM=truecolor
   2. /abs/path/to/bin/tctl -s 1712345678-42-before wait ">" --timeout 15000
   3. /abs/path/to/bin/tctl -s 1712345678-42-before type "hello world"
   4. /abs/path/to/bin/tctl -s 1712345678-42-before press enter
   5. /abs/path/to/bin/tctl -s 1712345678-42-before wait-idle
   6. /abs/path/to/bin/tctl -s 1712345678-42-before close"
```

```
Task prompt for a Remotion render worker:
  "Run this command. Report the output file path and any errors.
   /abs/path/to/scripts/render-showcase.sh \
     --props /tmp/droid-run-1712345678-42-xxxx/showcase-props.json \
     --output /tmp/droid-run-1712345678-42-xxxx/demo.mp4 \
     /tmp/droid-run-1712345678-42-xxxx/before.cast /tmp/droid-run-1712345678-42-xxxx/after.cast"
```

### Parallel capture pattern (comparison flows only)

Only applicable when the Layout default table above selects `side-by-side`. For a `single` layout, launch one capture worker and skip this section.

For before/after comparison demos, launch both capture workers simultaneously:

```
1. Parent constructs the interaction script (identical for both branches)
2. Launch worker A: capture the baseline/reference branch with `--repo-root` set to that worktree
3. Launch worker B: capture the candidate/change branch with `--repo-root` set to that worktree
4. Wait for both to complete (TaskOutput)
5. Collect .cast paths from results
6. Continue to compose
```

## Shared tooling

Terminal drivers use the unified `tctl` wrapper. agent-browser has its own CLI and does not use `tctl`.

Drivers can be combined in one workflow — e.g., `tctl` for a CLI and `agent-browser` for a web UI it interacts with.

## Prerequisites

| Stage | Platform | Required | Optional |
|---|---|---|---|
| tuistory | All | `tuistory`, `asciinema`, `agg` | `tmux` |
| true-input | Linux/Wayland | `cage`, `wtype`, Wayland terminal, `/dev/dri/*` | `grim`, `wf-recorder` |
| true-input | Windows (KVM) | `libvirt`, `qemu`, KVM VM with SPICE + SSH, `DROID_VM_*` env vars | `virt-manager` |
| true-input | macOS (QEMU) | `qemu`, `socat`, macOS VM with SSH, `DROID_MAC_*` env vars | — |
| agent-browser | All | `agent-browser` (+ `agent-browser install`) | — |
| compose | All | `ffmpeg`, `ffprobe`, `agg` | — |
| showcase | All | Node.js (>= 18), Chrome/Chromium | — |

### Install commands

```bash
# tuistory driver + recording
npm install -g tuistory                              # virtual PTY driver
pip install asciinema                                # terminal recording (tctl wraps this)
cargo install --git https://github.com/asciinema/agg  # .cast -> .gif converter (compose needs this)

# true-input driver (Linux/Wayland)
sudo apt-get install -y cage wtype                   # required: headless compositor + keystroke injection
sudo apt-get install -y grim wf-recorder             # optional: screenshots + video recording

# agent-browser driver
agent-browser install                                # one-time: downloads bundled Chromium

# compose + showcase (video rendering)
sudo apt-get install -y ffmpeg                       # video processing (includes ffprobe)
cd ${DROID_PLUGIN_ROOT}/remotion && npm install       # Remotion dependencies
# Chrome or Chromium must be installed for Remotion rendering
```
