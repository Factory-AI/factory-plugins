# True-Input: macOS via QEMU Monitor

Drive a macOS VM's terminal through QEMU HMP `sendkey` -- keystrokes enter the VM's virtual USB keyboard, hitting macOS's HID subsystem the same way a physical keyboard would. No host compositor leakage, no SSH PTY distortion.

## Architecture

```
mac-ctl.sh type / press
        |
        v
  socat → QEMU monitor socket → sendkey → QEMU virtual USB keyboard (HID)
        |
        v
  macOS HID subsystem → Terminal.app / iTerm2 / Ghostty
```

## Prerequisites

Host packages: `qemu-system-x86_64` (or `aarch64`), `socat`, `ffmpeg` or ImageMagick (for PNG conversion).
VM: QEMU macOS guest with UEFI, user-mode networking with SSH port forward.
The VM must be started with `MONITOR_SOCKET` env var pointing to a unix socket so `mac-ctl.sh` can connect.

Required env vars:

```bash
export DROID_MAC_MONITOR="<path-to-qemu-monitor-socket>"  # QEMU monitor unix socket
export DROID_MAC_SSH_HOST="<ssh-config-host-alias>"        # SSH config host alias
```

Optional env vars:

```bash
export DROID_MAC_BOOT_SCRIPT="<path-to-qemu-boot-script>"  # for `up` command
export DROID_MAC_SHOT_DIR="/tmp"
```

## Core pattern

```bash
MACCTL=${DROID_PLUGIN_ROOT}/scripts/macos/mac-ctl.sh

$MACCTL up                                       # start VM (needs DROID_MAC_BOOT_SCRIPT)
$MACCTL wait-boot 120                            # poll SSH until ready
$MACCTL terminal                                 # open Terminal.app via Spotlight
sleep 3
$MACCTL type "echo hello world"
$MACCTL press enter
sleep 2
$MACCTL shot /tmp/result.png                     # screendump via QEMU monitor → PNG
```

## Command reference (via mac-ctl.sh)

| Command | Purpose |
|---|---|
| `up` | Start VM via boot script (requires `DROID_MAC_BOOT_SCRIPT`) |
| `down` | Graceful shutdown via SSH `sudo shutdown -h now` |
| `kill` | Force-quit QEMU process via monitor `quit` |
| `status` | Check VM state via monitor socket |
| `wait-boot [timeout]` | Poll SSH until macOS is ready (default 120s) |
| `type <text>` | Type literal text, char by char via QEMU `sendkey` |
| `press <key> [key ...]` | Send key or chord (e.g., `press cmd c`, `press shift enter`) |
| `shot [path]` | Screenshot via `screendump` → PPM → PNG |
| `terminal` | Open Terminal.app via Spotlight (Cmd+Space) |
| `iterm` | Open iTerm2 via Spotlight |
| `spotlight [query]` | Open Spotlight, optionally type + launch a query |
| `ssh <cmd>` | Run command via SSH (deployment only -- **not** for TUI testing) |

### macOS-specific keys

`cmd` / `command` maps to `meta_l` (the macOS Command key). Chords work as expected:

```bash
$MACCTL press cmd c           # Copy
$MACCTL press cmd shift 3     # Screenshot (macOS native)
$MACCTL press cmd space       # Spotlight
$MACCTL press cmd q           # Quit foreground app
```

## Key differences from other platforms

| Concern | Linux (Wayland) | Windows (KVM/virsh) | macOS (QEMU monitor) |
|---|---|---|---|
| Input path | `wtype` → compositor → terminal | `virsh send-key` → QEMU HID | `sendkey` via monitor socket → QEMU HID |
| Snapshot | Scrubbed PTY log | `virsh screenshot` (PNG) | `screendump` (PPM → PNG) |
| Wait mechanism | PTY log polling | Sleep-based | SSH polling (`wait-boot`) + sleep-based |
| Recording | `wf-recorder` | Poll-based screenshot → ffmpeg | Poll-based screenshot → ffmpeg |
| Session isolation | Per-session Wayland runtime dir | Full VM isolation | Full VM isolation |
| App launcher | N/A (direct command) | Start menu (`win` key) | Spotlight (`cmd+space`) |

## Recording

Poll-based recording via `screendump`:

```bash
MACREC=${DROID_PLUGIN_ROOT}/scripts/macos/mac-record.sh

$MACREC start /tmp/demo.mp4 5    # 5 fps
# ... interact with mac-ctl.sh ...
$MACREC stop                      # encodes PPM frames to MP4
```

## Troubleshooting

| Problem | Fix |
|---|---|
| Monitor socket not found | VM isn't running, or was started without `MONITOR_SOCKET` env var |
| SSH times out on boot | macOS VMs under QEMU take 60-120s to boot. Increase `wait-boot` timeout |
| `screendump` is black | VM may be asleep. `mac-ctl.sh press space` to wake |
| Keystrokes not registering | Check that QEMU was started with `-device usb-kbd` (not evdev passthrough) |
| PPM not converting to PNG | Install ImageMagick (`magick`) or `ffmpeg` on the host |

## Recovery

```bash
MACCTL=${DROID_PLUGIN_ROOT}/scripts/macos/mac-ctl.sh
$MACCTL press esc
$MACCTL shot /tmp/state.png
$MACCTL kill                     # hard-kill QEMU process
$MACCTL up                       # cold restart
```

## Future: native macOS (no VM)

For bare-metal Macs (not VMs), the QEMU monitor approach isn't available. Potential paths:
- **Wawona** (native macOS Wayland compositor) + `wtype` + `waypipe` -- would reuse Linux plumbing
- **AppleScript** / Accessibility API injection into Terminal.app / iTerm2
- **cliclick** for mouse/keyboard automation

These are not yet implemented. Use this QEMU approach for VM-based macOS testing.
