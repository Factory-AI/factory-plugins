---
name: droid-cli
description: Background knowledge for droid-control workflows -- not invoked directly. Droid CLI target patterns, shortcuts, modes, and launch helpers.
user-invocable: false
---

# Droid CLI Target

The orchestrator routed you here. Layer these target-specific patterns on top of the driver skill you already loaded.

Droid-specific shortcuts, modes, and launch patterns.

## Shortcuts

| Action | Key chord | Result |
|---|---|---|
| Toggle Spec mode | `shift tab` | toggles Spec mode on/off |
| Cycle autonomy | `ctrl l` | Off > Low > Med > High > Off |
| Cycle models | `ctrl n` | cycles available models |
| Cycle reasoning | `tab` | High > none > Low > Medium > High |
| Cancel / close / stop | `esc` | stops streaming, closes overlays |
| Clear input | `ctrl u` | clears current line |
| Toggle bash mode | `!` on empty input | switches prompt between `>` and `$` |
| Help / shortcuts | `?` | opens keybinding help |
| Multiline input | `shift enter` | inserts newline without submitting |

## Dialogs

When a dialog shows `Use up/down to navigate...`: `up`/`down` moves the highlight, `enter` selects, `esc` closes.

## Slash commands

| Command | Purpose |
|---|---|
| `/help` | Show commands |
| `/settings` | Open settings menu |
| `/model` | Open model selector |
| `/clear` or `/new` | Start a new session |
| `/sessions` | Browse previous sessions |
| `/review` | Start AI code review |
| `/status` | Show current config |
| `/cost` | Show usage / cost |
| `/compress [prompt]` | Summarize and move to fresh session |

## File mentions

Type `@` to open file suggestions, filter by typing, `tab` to accept, `esc` to cancel:

```bash
$TCTL -s demo type "review @"
$TCTL -s demo type "package.json"
$TCTL -s demo press tab
```

## Visual cues

| State | What to look for |
|---|---|
| Spec mode on | input border shows `Spec` |
| Bash mode on | prompt is `$` |
| Idle / ready | prompt is `>` with no spinner |
| Dialog open | boxed menu + navigation hint |
| File suggestions | dropdown under input |
| Thinking | `Thinking...` and stop hint |

## Launching Droid

### How `droid-dev` works

`droid-dev` is a thin bash shim at `~/.local/bin/droid-dev`. It runs `bun` against whichever checkout `DROID_DEV_REPO_ROOT` points to — it does NOT use a pre-compiled binary. This means:

- **No per-branch builds.** One `npm run setup` (in any checkout) installs the shim. Switching branches is instant via `--repo-root`.
- **Prerequisite:** The target worktree must have `node_modules` installed (`npm install` at the repo root). If missing, the bun launch fails.
- **`tctl --repo-root`** sets `DROID_DEV_REPO_ROOT` automatically and pins the session to that worktree.

### `droid-dev`

`droid-dev` launches require `--repo-root` (or `--env DROID_DEV_REPO_ROOT=...`). `tctl` enforces this — launches without it will fail:

```bash
# tuistory (default — virtual PTY)
$TCTL launch "droid-dev" -s demo --backend tuistory \
  --repo-root /path/to/worktree \
  --cols 120 --rows 36

# true-input (real terminal proof — headless Wayland compositor)
$TCTL launch "droid-dev" -s demo --backend true-input \
  --repo-root /path/to/worktree
```

### Feature branch / worktree

For comparisons, launch separate sessions pointing at different worktrees:

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

### Comparison setup (before/after demos)

For before/after comparisons, you need two worktree paths — one for the baseline and one for the candidate.

1. **Find existing worktrees**: `git worktree list` in any checkout. The main clone (often on `dev` or `main`) is a valid baseline.
2. **Create if needed**: `git worktree add /tmp/baseline-worktree dev` (or the relevant base branch).
3. **Ensure `node_modules`**: Run `npm install` in any worktree that lacks it. This is the only setup needed — no `npm run setup` or CLI build per branch.
4. **Launch with `--repo-root`**: Each `tctl launch` pins to one worktree.

### Environment safety

`tctl`'s runner script launches `droid-dev` via `bash -lc`, which loads a clean login shell. Stale `FACTORY_*` env vars from parent processes are typically overridden by the runner. If you suspect env contamination, pass explicit overrides with `--env`.

## Exec mode

Non-interactive single-shot execution:

```bash
droid exec "analyze this file"
droid exec --auto medium "run the tests"
```

## Logging

Enable debug logging by passing the log file path via `--env`:

```bash
$TCTL launch "droid-dev" -s demo --backend tuistory \
  --repo-root /path/to/worktree \
  --env FACTORY_LOG_FILE=/tmp/droid-test.log
tail -f /tmp/droid-test.log
```
