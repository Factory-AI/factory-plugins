---
name: desktop-control
description: Background knowledge for droid-control workflows -- not invoked directly. Desktop-control driver mechanics for native GUI app automation via trycua cua-driver.
user-invocable: false
---

# Desktop-Control Driver

The orchestrator routed you here. Use these mechanics to execute your plan.

Drive native desktop GUI apps through upstream [trycua/cua](https://github.com/trycua/cua) `cua-driver`: enumerate apps and windows, snapshot accessibility trees, click/type/scroll by `element_index` or pixel coordinates, and verify by re-snapshot -- all without bringing the target to the foreground.

## When to use

- Automating a native desktop app (Finder, Notepad, System Settings, native editors)
- Driving native dialogs and security/permission sheets that no DOM or PTY can reach
- Visual QA of native UI: per-window screenshots, accessibility-tree assertions

If the target is a terminal TUI, use **tuistory** or **true-input**. If it is a web page or an Electron app, use **agent-browser** -- CDP beats accessibility trees for anything Chromium-based.

## Platform support

| Platform | Upstream tier | Read |
|---|---|---|
| macOS | Production | [platforms/macos.md](platforms/macos.md) |
| Windows | Production | [platforms/windows.md](platforms/windows.md) |
| Linux | Pre-release (real caveats) | [platforms/linux.md](platforms/linux.md) |

**Read the platform file for your target OS.** Each contains permissions, daemon launch, and platform-specific patterns and failure modes.

## Prerequisites

```bash
# one-time install: per-user, no sudo/admin
curl -fsSL https://raw.githubusercontent.com/trycua/cua/main/libs/cua-driver/scripts/install.sh | bash
# Windows (PowerShell):
#   irm https://raw.githubusercontent.com/trycua/cua/main/libs/cua-driver/scripts/install.ps1 | iex

cua-driver doctor           # platform probes: permissions, daemon, accessibility plumbing
cua-driver skills install   # fetch the upstream skill pack to ~/.cua-driver/skills/cua-driver
```

The upstream pack (`~/.cua-driver/skills/cua-driver/SKILL.md` + your platform's doc) is the deep reference -- full tool surface, window-state behavior matrix, forbidden-command lists -- and it updates with the binary. **Read it before any nontrivial workflow.** This atom owns the droid-control integration: routing, run isolation, delegation, evidence handoff.

## Daemon lifecycle

`element_index` workflows **require the daemon**. Without it each CLI invocation is a fresh process and the per-`(pid, window_id)` element cache dies between calls.

```bash
cua-driver serve            # start the daemon (macOS needs the LaunchServices form -- see platforms/macos.md)
cua-driver status           # daemon + socket health
cua-driver stop
```

Permissions are checked and granted through the driver, not by hand-editing system settings (macOS-only gate; a no-op surface on Windows/Linux):

```bash
cua-driver permissions status   # read-only; answers via the running daemon
cua-driver permissions grant    # attributed prompt flow -- the correct way to grant
```

## Core loop

Tool names are `snake_case` and invoked directly: `cua-driver <tool> '<json>'`. (`cua-driver call <tool>` is legacy; do not use it.) `cua-driver list-tools` for the inventory, `cua-driver describe <tool>` for any schema.

Every workflow is Discover -> Observe -> Act -> Verify against an explicit `(pid, window_id)`:

```bash
cua-driver launch_app '{"name":"TextEdit"}'
#  -> {pid: 844, windows: [{window_id: 10725, ...}]}   # list_windows only needed for long-lived pids
cua-driver get_window_state '{"pid":844,"window_id":10725}' --screenshot-out-file "${RUN_DIR}/before.png"
cua-driver click '{"pid":844,"window_id":10725,"element_index":14,"session":"'"${RUN_ID}"'-desktop"}'
cua-driver get_window_state '{"pid":844,"window_id":10725}' --screenshot-out-file "${RUN_DIR}/after.png"
```

**Snapshot before AND after every action.** The pre-action `get_window_state` resolves the `element_index` you are about to use -- indices are per-snapshot, per `(pid, window_id)`, and stale ones fail with `No cached AX state`. The post-action snapshot is the evidence the action landed; without it a silent no-op looks like success.

Addressing-mode preference:

1. **`element_index`** (default) -- semantic, works on hidden and backgrounded windows, no foreground change.
2. **Pixel** `click '{"pid":N,"window_id":W,"x":X,"y":Y}'` -- for surfaces the tree does not reach (canvases, custom-drawn controls). Coordinates are window-local screenshot pixels, top-left origin.
3. **Keyboard** (`press_key`, `hotkey`) and platform fallbacks -- last resort; see the platform files.

## Run isolation (ground rule 5 -> cua sessions)

cua sessions are the desktop equivalent of `tctl` session prefixes: a session owns its agent cursor, config overrides, and recording scope. Declare one per run, derived from the workflow's `RUN_ID`, and pass it on every action:

```bash
cua-driver start_session '{"session":"'"${RUN_ID}"'-desktop"}'
# ... every action carries "session":"${RUN_ID}-desktop" ...
cua-driver end_session '{"session":"'"${RUN_ID}"'-desktop"}'
```

Parallel workers each declare their **own session** and pass `creates_new_application_instance: true` to `launch_app` so each gets its own window. The element cache is keyed on `(pid, window_id)` and the cursor on `session`, so isolated workers cannot collide.

## Delegation

`cua-driver` is on PATH -- workers need no `${DROID_PLUGIN_ROOT}` resolution. As with the other drivers, give capture workers **exact commands** with the parent's run scope baked in:

```
Task prompt for a desktop capture worker:
  "Run these commands in order. Report screenshot paths and any errors.
   1. cua-driver start_session '{"session":"1712345678-42-notepad"}'
   2. cua-driver launch_app '{"name":"Notepad","creates_new_application_instance":true}'
      -> note the returned pid and window_id
   3. cua-driver get_window_state '{"pid":<pid>,"window_id":<wid>}' --screenshot-out-file /tmp/droid-run-1712345678-42-xxxx/before.png
   4. cua-driver type_text '{"pid":<pid>,"window_id":<wid>,"element_index":<text-area>,"text":"hello","session":"1712345678-42-notepad"}'
   5. cua-driver get_window_state '{"pid":<pid>,"window_id":<wid>}' --screenshot-out-file /tmp/droid-run-1712345678-42-xxxx/after.png
   6. cua-driver end_session '{"session":"1712345678-42-notepad"}'"
```

## Evidence handoff

| Proof type | How to capture |
|---|---|
| Window state | `get_window_state ... --screenshot-out-file ${RUN_DIR}/proof-N.png` (also keeps the PNG out of the tool response) |
| Full display | `cua-driver screenshot '{"out_file":"'"${RUN_DIR}"'/screen.png"}'` |
| Semantic assertions | `tree_markdown` from `get_window_state` (filter with `"query":"..."`) |
| Video | `cua-driver recording start` / `recording stop` -> session-scoped `recording.mp4` |

Hand PNG/mp4 paths to **compose** / **verify** like any other driver output. Keep raw tool output alongside screenshots whenever GUI behavior is the thing under test.

## Critical rules

1. **Never change the user's frontmost app.** If a command says activate, foreground, raise, or make key -- stop; the per-pid event paths exist precisely so you do not need it. Platform forbidden-lists live in the upstream pack.
2. **Re-snapshot after every action and report what you observed**, not what you intended. An unchanged tree after an action is a finding, not a formality.
3. **Destructive actions need explicit user intent.** Do not delete files, send messages, or submit forms unless the workflow asked for exactly that.
