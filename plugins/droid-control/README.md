# droid-control

Terminal, browser, and computer automation plugin for Droids.

Droids can read and write code. This plugin lets them *operate* it: launch apps, type commands, click buttons, record what happens, and produce polished evidence. No human hands required.

## What you get

**Record a demo video from a PR:**

```text
/demo pr-1847
```

Droid reads the PR, scripts the interactions that prove the change works, records both branches in parallel, and renders a side-by-side comparison video. Use Factory presets for cinematic warmth or macos/minimal presets for clean utilitarian demos.

**Verify a behavior claim:**

```text
/verify "ESC cancels streaming in bash mode"
```

Droid launches the app, tests the claim, and reports what actually happened with screenshots, snapshots, or byte captures. If the claim is false, that is a valid finding, not a failed run.

**Run a QA flow against a web app:**

```text
/qa-test https://app.example.com -- login, create a project, invite a member
```

Droid drives the browser through the flow, captures each step, and reports pass/fail with annotated screenshots.

## Quick start

```bash
# Register the Factory plugins marketplace (if not already added)
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins

# Install the plugin
droid plugin install droid-control@factory-plugins --scope user

# Install Remotion dependencies (one-time, only needed for video rendering)
# Find the plugin install path with: droid plugin list --scope user
cd <plugin-path>/remotion && npm install
```

Or use the `/plugins` UI: Browse tab, select droid-control, install.

Then open a Droid session and run `/demo`, `/verify`, or `/qa-test`.

## Commands

### `/demo`

Plans and records a demo video. Accepts a PR number, GitHub URL, or free-text description. Comparison PRs get side-by-side layout by default; new features get single-branch. Add `showcase` for cinematic polish or `keys` for keystroke overlay.

### `/verify`

Tests a specific behavior claim and reports findings with evidence. The droid is an investigator, not an advocate: contradictory evidence is surfaced instead of hidden.

### `/qa-test`

Runs automated QA against terminal CLIs, web apps, or Electron apps. Accepts a URL, CLI command, app name, PR reference, or free-text flow with optional test steps after `--`.

## How it works

`droid-control` is a composition system for agent attention:

1. **Commands** parse user intent into commitments.
2. **The orchestrator** routes by target, stage, and artifact needs.
3. **Atom skills** provide only the mechanics needed right now: drivers, target patterns, capture, compose, verify, and showcase polish.
4. **Workers** handle mechanical capture/render jobs while the parent droid keeps planning and verification context.
5. **Verify** checks the final evidence against the original commitments.

For the full rationale and runtime pipeline, see [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Video rendering

The compose stage uses [Remotion](https://www.remotion.dev/) for video compositing. Presets provide window chrome, spacing, palettes, backgrounds, particles, noise, color grading, configurable transitions (`motion-blur`, `flash`, `whip-pan`, `light-leak`, `glitch-lite`), zooms, spotlights, keystroke overlays, section headers, and syntax-highlighted code annotations.

The `render-showcase.sh` helper owns the full pipeline: `.cast` conversion via `agg`, clip staging, duration detection, Remotion rendering, and cleanup.

## Prerequisites

| Stage | Platform | Required |
|---|---|---|
| tuistory | All | `tuistory`, `asciinema`, `agg` |
| true-input | Linux/Wayland | `cage`, `wtype`, Wayland terminal |
| true-input | Windows (KVM) | `libvirt`, `qemu`, KVM VM with SSH |
| true-input | macOS (QEMU) | `qemu`, `socat`, macOS VM with SSH |
| agent-browser | All | `agent-browser` |
| desktop-control | All | `cua-driver` |
| compose | All | `ffmpeg`, `ffprobe`, `agg` |
| showcase | All | Node.js (>= 18), Chrome/Chromium |

```bash
npm install -g tuistory                               # virtual PTY driver
pip install asciinema                                 # terminal recording
cargo install --git https://github.com/asciinema/agg  # .cast -> .gif converter
sudo apt-get install -y ffmpeg                        # video processing
agent-browser install                                 # browser automation (downloads Chromium)
curl -fsSL https://raw.githubusercontent.com/trycua/cua/main/libs/cua-driver/scripts/install.sh | bash  # native desktop GUI automation
cd plugins/droid-control/remotion && npm install      # Remotion video rendering
```

Only install what you need for your use case. Terminal demos need tuistory, asciinema, agg, and ffmpeg. Web/Electron automation just needs agent-browser. Native desktop GUI automation just needs cua-driver.
