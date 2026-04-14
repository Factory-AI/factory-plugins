# droid-control

Terminal, browser, and computer automation plugin for Droids.

Droids can read and write code. This plugin enables them to *operate* it: launch apps, type commands, click buttons, record what happens, and produce polished video evidence of it. No human hands required (they don't have any).

## What you get

**Record a demo video from a PR:**

```
/demo pr-1847
```

Droid reads the PR, scripts the interactions that prove the change works, records both branches in parallel, and renders a side-by-side comparison video. Factory preset for cinematic warmth, macos preset for clean and utilitarian.

**Verify a behavior claim:**

```
/verify "ESC cancels streaming in bash mode"
```

Droid launches the app, attempts the claim, and reports what actually happened, with screenshots and text snapshots as evidence. If the claim is false, that's a valid finding, not a failure.

**Run a QA flow against a web app:**

```
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

Plans and records a demo video. Accepts a PR number, GitHub URL, or free-text description. Comparison PRs get side-by-side layout by default; new features get single-branch. Add "showcase" for cinematic polish, "keys" for keystroke overlay.

### `/verify`

Tests a specific behavior claim and reports findings with evidence. Frames the droid as an investigator. Anti-fabrication rules prevent staging evidence to match expected outcomes.

### `/qa-test`

Automated QA against terminal CLIs or web/Electron apps. Accepts a URL, CLI command, or app description with optional test steps after `--`.

## How it works

Three layers:

- **Orchestrator** -- routes each request through three independent lookups (target, stage, artifact) to determine which skills to load. ~93 lines.
- **10 atom skills** -- self-contained background knowledge loaded on demand. Driver atoms (tuistory, true-input, agent-browser), target atoms (droid-cli, pty-capture), stage atoms (capture, compose, verify), and a polish atom (showcase).
- **3 commands** -- thin intent declarations that parse arguments into commitments, then delegate to atoms via hybrid handoffs.

Every workflow flows through **capture → compose → verify**. Commands declare *what* to produce; atoms own *how*.

## Video rendering

The compose stage uses [Remotion](https://www.remotion.dev/) (React-based video renderer) for all compositing. 6 visual presets, automatic cinematic layers (warm backgrounds, floating particles, noise overlay, motion blur transitions), and effect-driven layers (spotlight, zoom, keystroke overlay, section headers).

The `render-showcase.sh` helper handles the full pipeline: `.cast` conversion via `agg`, clip staging, duration detection, Remotion render, and cleanup.

## Prerequisites

| Stage | Platform | Required |
|---|---|---|
| tuistory | All | `tuistory`, `asciinema`, `agg` |
| true-input | Linux/Wayland | `cage`, `wtype`, Wayland terminal |
| true-input | Windows (KVM) | `libvirt`, `qemu`, KVM VM with SSH |
| true-input | macOS (QEMU) | `qemu`, `socat`, macOS VM with SSH |
| agent-browser | All | `agent-browser` |
| compose | All | `ffmpeg`, `ffprobe`, `agg` |
| showcase | All | Node.js (>= 18), Chrome/Chromium |

```bash
npm install -g tuistory                              # virtual PTY driver
pip install asciinema                                 # terminal recording
cargo install --git https://github.com/asciinema/agg   # .cast → .gif converter
sudo apt-get install -y ffmpeg                        # video processing
agent-browser install                                 # browser automation (downloads Chromium)
cd plugins/droid-control/remotion && npm install       # Remotion (video rendering)
```

Only install what you need for your use case. Terminal demos need tuistory, asciinema, agg, and ffmpeg. Web/Electron automation just needs agent-browser.
