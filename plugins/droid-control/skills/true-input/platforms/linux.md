# True-Input: Linux / Wayland

## Prerequisites

```bash
sudo apt-get install -y cage wtype          # required
sudo apt-get install -y grim wf-recorder    # optional: screenshots / video
ls /dev/dri/                                # must be non-empty
```

At least one Wayland terminal: `ghostty`, `kitty`, or `alacritty`.

## Architecture

`tctl` launches a headless Wayland compositor (`cage`) with an isolated per-session runtime directory, opens a real terminal emulator inside it, injects keystrokes via `wtype`, and monitors the PTY log stream via `script`. All socket/runtime management is handled by `tctl`.

## Core pattern

```bash
TCTL=${DROID_PLUGIN_ROOT}/bin/tctl

$TCTL launch "droid-dev" -s proof --backend true-input --cols 120 --rows 36
$TCTL -s proof wait ">" --timeout 15000
$TCTL -s proof type "hello"
$TCTL -s proof press shift enter
$TCTL -s proof snapshot --trim
$TCTL -s proof screenshot -o /tmp/proof.png
$TCTL -s proof close
```

Auto-detection order: ghostty > kitty > alacritty. Force a specific terminal:

```bash
$TCTL launch "droid-dev" -s proof --backend ghostty
```

## Command reference (via tctl)

| Command | Purpose |
|---|---|
| `launch <cmd> -s <name> --backend true-input` | Start session in headless compositor |
| `type <text>` | Send literal text via wtype |
| `press <key> [keys...]` | Send key chord (supports modifiers) |
| `wait <pattern>` | Block until text or `/regex/` in PTY log |
| `wait-idle` | Block until PTY log stabilizes |
| `snapshot [--trim]` | Print scrubbed PTY log output |
| `screenshot [-o <path>]` | Capture compositor PNG |
| `record start <path>` | Start wf-recorder video |
| `record stop` | Stop video recording |
| `close` | Tear down compositor + terminal |

## Recording

Recording can start and stop independently of the session:

```bash
$TCTL -s proof record start /tmp/proof.mp4
# ... interact ...
$TCTL -s proof record stop
```

Or record from launch via `--record`:

```bash
$TCTL launch "droid-dev" -s proof --backend true-input --record /tmp/proof.mp4
```

## Terminal encoding reference

### Ghostty

Falls back to xterm `modifyOtherKeys` (Kitty keyboard protocol disabled due to arrow-key issues):

| Key | Bytes |
|---|---|
| `Enter` | `\r` |
| `Shift+Enter` | `\x1b[27;2;13~` |

### Kitty

CSI-u protocol by default:

| Key | Bytes |
|---|---|
| `Enter` | `\r` |
| `Shift+Enter` | `\x1b[13;2u` |
| keypad `Enter` | `\x1b[57414u` |

### VS Code / Cursor / Windsurf

Integrated terminals need explicit keybinding setup. Droid's helper writes `workbench.action.terminal.sendSequence` bindings that emit `\\\r\n` for `Shift+Enter`.

### tmux

tmux may rewrite sequences when relaying. With extended keys enabled, `Shift+Enter` typically appears as `\x1b[27;2;13~` regardless of the inner terminal's native encoding.

## Recovery

```bash
$TCTL -s proof press esc
$TCTL -s proof snapshot --trim
$TCTL -s proof close
```
