---
name: pty-capture
description: Background knowledge for droid-control workflows -- not invoked directly. Capture ground-truth byte sequences from real terminal emulators.
user-invocable: false
---

# PTY Byte Capture

The orchestrator routed you here. Use these mechanics to execute your plan.

Capture the exact bytes a real terminal emits for a given keystroke. Use this when the question is "what sequence does terminal X send for key Y?" rather than "does the UI look right?"

## Platform support

| Platform | Status | Read |
|---|---|---|
| Linux / Wayland | Implemented | [platforms/linux.md](platforms/linux.md) |
| Windows (KVM) | Implemented | [platforms/windows.md](platforms/windows.md) |
| macOS (QEMU) | Implemented | [platforms/macos.md](platforms/macos.md) |

**Read the platform file for your target OS.** Each contains the capture architecture, prerequisites, usage pattern, and platform-specific notes.

## Known dead ends

- **Xvfb + xdotool**: bypasses real keyboard processing entirely
- **uinput + Xvfb**: Xvfb does not consume kernel input devices
- **SSH PTY for keystroke injection**: distorts the input encoding; SSH is only for output capture or deployment

## Follow-on

Feed captured bytes into terminal compatibility fixtures and replay tests in `apps/cli`.
