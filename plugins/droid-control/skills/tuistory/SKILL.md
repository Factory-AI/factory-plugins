---
name: tuistory
description: Background knowledge for droid-control workflows -- not invoked directly. Tuistory driver mechanics for terminal TUI automation via virtual PTY.
user-invocable: false
---

# Tuistory Driver

The orchestrator routed you here. Use these mechanics to execute your plan.

Launch a target command in a virtual PTY with Playwright-style CLI for typing, pressing keys, waiting, snapshotting, and recording.

## When to use

- Routine TUI automation and regression checks
- Deterministic `wait` / `wait-idle` against the virtual screen buffer
- Text snapshots of exactly what the user would see
- Any scenario where you do **not** need to prove real terminal keyboard encoding

If you need to prove what Ghostty or Kitty actually emits for a given keystroke, use **true-input** instead.

## Prerequisites

```bash
npm install -g tuistory    # or: bun add -g tuistory
```

Optional: `tmux` (scrollback flows), `asciinema` (recordings), `agg` (`.cast` to `.gif`).

## Core pattern

```bash
TCTL=${DROID_PLUGIN_ROOT}/bin/tctl

$TCTL launch "droid-dev" -s demo --backend tuistory \
  --repo-root /path/to/worktree \
  --cols 120 --rows 36 \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor
$TCTL -s demo wait ">" --timeout 15000
$TCTL -s demo type "hello"
$TCTL -s demo press enter
$TCTL -s demo snapshot --trim
$TCTL -s demo close
```

**Note:** `--repo-root` is mandatory for `droid-dev` launches — `tctl` enforces it.

**Always pass `--env FORCE_COLOR=3 --env COLORTERM=truecolor`** when launching. The virtual PTY doesn't advertise color support, so Node.js apps (Ink/chalk) suppress all color escape codes without these.

## Command reference (via tctl)

| Command | Purpose |
|---|---|
| `launch <cmd> -s <name> --backend tuistory` | Start a tuistory session |
| `type <text>` | Send literal text |
| `press <key> [keys...]` | Send key chord (e.g., `press shift enter`) |
| `wait <pattern>` | Block until text or `/regex/` appears |
| `wait-idle` | Block until output stabilizes |
| `snapshot [--trim]` | Print cleaned text (`--trim` strips trailing blanks) |
| `close` | Tear down session |

Launch options: `--cols <n>`, `--rows <n>`, `--cwd <path>`, `--env KEY=VALUE`, `--record <path>`.

## Recording

Pass `--record` at launch. `tctl` wraps `asciinema rec` around the PTY, so recording **must** be set at launch time (raw `tuistory` cannot record):

```bash
$TCTL launch "droid-dev" -s demo --backend tuistory \
  --repo-root /path/to/worktree \
  --cols 120 --rows 36 --record /tmp/demo.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor
# ... interact ...
$TCTL -s demo close    # finalizes the .cast
```

Before/after comparison -- launch two sessions against different worktrees:

```bash
$TCTL launch "droid-dev" -s before --backend tuistory \
  --repo-root /path/to/baseline-worktree \
  --cols 120 --rows 36 --record /tmp/before.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor

$TCTL launch "droid-dev" -s after --backend tuistory \
  --repo-root /path/to/candidate-worktree \
  --cols 120 --rows 36 --record /tmp/after.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor
```

Playback: `asciinema play /tmp/demo.cast`

## tmux (scrollback flows only)

Only needed when the demo requires terminal-emulator scrollback and the app uses the standard buffer (not alternate screen). True-input sessions rarely need tmux because the real terminal owns native scrollback.

Use `--tmux` at launch. `tctl` wraps the command in tmux, starts tmux with `TERM=xterm-256color`, and preconfigures `default-terminal=tmux-256color`, `terminal-features=...,xterm-256color:RGB`, `terminal-overrides=...,xterm-256color:Tc` (fallback for tmux < 3.2), `COLORTERM=truecolor` (via `set-environment`), `escape-time=50`, and `mode-keys=vi`.

Copy-mode: `ctrl-b [` to enter, `g g` top, `shift-g` bottom, `ctrl-u`/`ctrl-d` half-page, `/` search, `q` to exit (not `esc`).

Launch with tmux:

```bash
$TCTL launch "droid-dev" -s demo --backend tuistory --tmux \
  --repo-root /path/to/worktree \
  --cols 120 --rows 36 --record /tmp/demo.cast \
  --env FORCE_COLOR=3 --env COLORTERM=truecolor
```

When recording includes tmux redraws, asciinema must wrap tmux, not the reverse.

## Known dead ends

- **Raw `asciinema rec`**: Do not call `asciinema rec` directly. `tctl --record` wraps `asciinema rec` around the PTY so that tuistory-relay still owns the session and interactive TUIs (Ink/React) receive stdin correctly. Calling `asciinema rec` manually bypasses this wiring — stdin forwarding breaks, typed keys echo on the outer PTY instead of reaching the child, and tuistory commands (`wait`, `snapshot`, `close`) cannot find the session.
- **Raw `tuistory launch` with tctl flags**: `tuistory` has no `--record`, `--backend`, `--repo-root`, or `--env` flags. Passing them crashes `tuistory-relay`. Use `tctl` for all launches.

## Recovery

```bash
$TCTL -s demo press esc         # bail out of a stuck dialog
$TCTL -s demo snapshot --trim   # check visible state
$TCTL -s demo close             # hard reset
```

## Escape hatch: raw tuistory (last resort)

If `tctl` itself is broken or unavailable, you can fall back to raw `tuistory` for non-recording sessions only. Raw `tuistory` accepts only `--cols`, `--rows`, and `-s` — no other flags. Do not pass `--record`, `--backend`, `--repo-root`, `--env`, or `--tmux`.

```bash
tuistory launch "my-tui-app" -s demo --cols 120 --rows 36
tuistory -s demo wait ">" --timeout 15000
tuistory -s demo snapshot --trim
tuistory -s demo close
```
