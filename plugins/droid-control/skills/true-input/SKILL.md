---
name: true-input
description: Background knowledge for droid-control workflows -- not invoked directly. True-input driver mechanics for real terminal emulator automation via headless Wayland compositor.
user-invocable: false
---

# True-Input Driver

The orchestrator routed you here. Use these mechanics to execute your plan.

Drive a real terminal emulator, injecting keystrokes through the platform's native HID input path. This proves exactly what bytes the terminal emits -- no synthetic injection, no PTY distortion.

## When to use

- Proving that a terminal really sends the sequence you expect (e.g., Ghostty's `Shift+Enter`)
- Recording demos that reflect actual terminal rendering
- Validating that Droid handles a keystroke correctly end-to-end in a specific terminal

If you don't need real terminal proof, use **tuistory** -- it's faster and more deterministic.

## Platform support

| Platform | Status | Driver | Read |
|---|---|---|---|
| Linux / Wayland | Implemented | `cage` + `wtype` + any Wayland terminal | [platforms/linux.md](platforms/linux.md) |
| macOS (QEMU) | Implemented | QEMU monitor `sendkey` to a macOS VM | [platforms/macos.md](platforms/macos.md) |
| Windows (KVM) | Implemented | `virsh send-key` to a KVM/QEMU VM | [platforms/windows.md](platforms/windows.md) |

**Read the platform file for your target OS.** Each contains prerequisites, core pattern, command reference, encoding reference, recording, troubleshooting, and recovery -- specific to that platform.

## Key differences from tuistory

| Concern | tuistory | true-input |
|---|---|---|
| Snapshot source | Virtual screen buffer | Scrubbed PTY log (Linux) or screenshot (VM platforms) |
| Wait mechanism | Event-driven (screen redraws) | Log polling (Linux) or sleep-based (VMs) |
| Recording | Must wrap launch (`--record`) | Can start/stop any time |
| Keyboard encoding | Synthetic (bypasses terminal) | Real terminal encoding path |

## Known dead ends

- **Xvfb + xdotool**: bypasses real keyboard processing entirely
- **uinput + Xvfb**: Xvfb does not consume kernel input devices
- **SSH for TUI testing**: PTY layer distorts input encoding; use SSH only for deployment
- **Raw `asciinema rec`**: true-input records via `wf-recorder` (Wayland screen capture), not asciinema. Use `tctl --record` or `tctl record start/stop`. Calling `asciinema rec` directly has no access to the compositor and produces nothing useful.
