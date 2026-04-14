# PTY Byte Capture: macOS (QEMU)

Capture terminal byte sequences from a macOS VM by running a capture script inside the guest over SSH, then injecting keystrokes via QEMU monitor `sendkey`.

## Architecture

```
mac-ctl.sh press  →  QEMU monitor sendkey  →  virtual USB kbd  →  macOS HID  →  Terminal.app
                                                                                      |
                                                                           capture script (SSH)
                                                                                      |
                                                                                  hex output
```

The capture script runs inside the macOS VM via SSH. Keystrokes arrive through the real HID path (QEMU virtual keyboard), not through the SSH PTY -- so captured bytes reflect what macOS Terminal.app actually delivers.

## Prerequisites

The macOS VM must have:
- SSH enabled (System Settings → General → Sharing → Remote Login)
- Python 3 installed (ships with macOS, or via `brew install python3`)

## Usage

```bash
MACCTL=${DROID_PLUGIN_ROOT}/scripts/macos/mac-ctl.sh

# Start a hex dumper inside the VM via SSH (runs in background)
$MACCTL ssh 'python3 -c "
import sys, tty, termios
fd = sys.stdin.fileno()
old = termios.tcgetattr(fd)
tty.setraw(fd)
sys.stdout.write(\"READY\n\")
sys.stdout.flush()
try:
    while True:
        b = sys.stdin.buffer.read(1)
        if not b: break
        sys.stdout.write(f\"{b[0]:02x} \")
        sys.stdout.flush()
        if b[0] == 0x11: break  # Ctrl+Q exits
finally:
    termios.tcsetattr(fd, termios.TCSADRAIN, old)
"' &
CAPTURE_PID=$!
sleep 2

# Now inject keystrokes via QEMU monitor (true HID path)
$MACCTL press shift ret
sleep 0.5

# Kill the capture session
kill $CAPTURE_PID 2>/dev/null

# For visual proof, take a screenshot
$MACCTL shot /tmp/macos-capture.png
```

## Important notes

- The SSH session provides the **output channel** only. The actual keystrokes travel through QEMU's virtual USB keyboard → macOS HID → Terminal.app's PTY.
- This captures what Terminal.app delivers to its PTY child, which may differ from what iTerm2 or Ghostty delivers for the same key.
- For Ghostty or iTerm2 capture, open those apps via `mac-ctl.sh spotlight` instead of using the SSH terminal.
