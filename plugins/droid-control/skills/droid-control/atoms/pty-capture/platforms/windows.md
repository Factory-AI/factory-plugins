# PTY Byte Capture: Windows (KVM)

Two PowerShell scripts in `${DROID_PLUGIN_ROOT}/scripts/windows/` get deployed to the VM via `vm-ctl.sh deploy`:

| Script | Captures | Use when |
|---|---|---|
| `win-key-dumper.ps1` | Win32 `ReadKey` events: VirtualKeyCode, ControlKeyState, CharHex | You need lossless key metadata (e.g., Shift+Enter vs Enter) |
| `win-vt-dumper.ps1` | Raw VT bytes with `ENABLE_VIRTUAL_TERMINAL_INPUT` | You want the exact escape sequences the console delivers |

Both exit with Ctrl+Q.

## Usage

```bash
VMCTL=${DROID_PLUGIN_ROOT}/scripts/windows/vm-ctl.sh
$VMCTL deploy                    # push scripts to VM
$VMCTL pwsh && sleep 4
$VMCTL type "powershell -ExecutionPolicy Bypass -File C:\capture\win-key-dumper.ps1"
$VMCTL press enter && sleep 3
$VMCTL press shift enter         # test keystroke via virsh send-key (true HID)
sleep 0.5
$VMCTL shot /tmp/key-events.png  # screenshot the captured events
$VMCTL press ctrl q              # exit the dumper
```

## VT mode vs Win32 API

**Important**: VT mode (`ENABLE_VIRTUAL_TERMINAL_INPUT`) cannot distinguish Shift+Enter from Enter -- both produce `0D`. Use `win-key-dumper.ps1` (Win32 `ReadKey` API) when modifier discrimination matters.

The Win32 API preserves VirtualKeyCode + exact modifier state (left/right Ctrl/Alt/Shift). Shift+Enter shows as VKey=13 + ShiftPressed, which is distinct from plain Enter.
