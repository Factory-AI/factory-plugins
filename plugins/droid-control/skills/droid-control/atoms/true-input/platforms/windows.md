# True-Input: Windows via KVM/QEMU

Drive a Windows VM's console through `virsh send-key` -- keystrokes enter the VM's virtual HID device, hitting the Windows Console Subsystem the same way a physical keyboard would. No host compositor leakage, no SSH PTY distortion.

## Architecture

```
vm-ctl.sh type / press
        |
        v
  virsh send-key ──> QEMU virtual keyboard (HID)
        |
        v
  Windows Console Subsystem ──> PowerShell / TUI app
```

## Prerequisites

Host packages: `qemu-full`, `libvirt`, `virt-manager`, `swtpm`, `dnsmasq`.
VM: KVM/QEMU with UEFI + TPM 2.0, SPICE display, NAT networking, SSH enabled.

Required env vars:

```bash
export DROID_VM_NAME="<libvirt-domain-name>"        # libvirt domain name
export DROID_VM_SSH_KEY="<path-to-ssh-private-key>"  # SSH key for deployment
export DROID_VM_SSH_USER="<ssh-username>"             # SSH user in the VM
```

## Core pattern

```bash
VMCTL=${DROID_PLUGIN_ROOT}/scripts/windows/vm-ctl.sh

$VMCTL up                                    # start the VM
sleep 15                                     # wait for boot
$VMCTL login                                 # dismiss lock screen
sleep 5
$VMCTL pwsh                                  # open PowerShell via Start menu
sleep 4
$VMCTL type "Get-Process | Select-Object -First 5"
$VMCTL press enter
sleep 2
$VMCTL shot /tmp/result.png                  # screenshot via virsh
```

## Command reference (via vm-ctl.sh)

| Command | Purpose |
|---|---|
| `up` / `down` / `kill` / `reboot` / `status` | VM lifecycle |
| `type <text>` | Type literal text, char by char via virsh send-key |
| `press <key> [keys...]` | Send key or chord (e.g., `press ctrl c`, `press shift enter`) |
| `login [password]` | Dismiss lock screen + type password |
| `shot [path]` | Screenshot via `virsh screenshot` |
| `pwsh` / `wt` | Open PowerShell / Windows Terminal via Start menu |
| `snap [name]` / `snaps` / `restore <name>` | VM snapshot management |
| `ssh <cmd>` | Run command via SSH (deployment only -- **not** for TUI testing) |
| `deploy` | Push capture scripts to VM via SCP |

## Key differences from Linux true-input

| Concern | Linux (Wayland) | Windows (KVM) |
|---|---|---|
| Input path | `wtype` → Wayland compositor → terminal | `virsh send-key` → QEMU HID → Windows console |
| Snapshot | Scrubbed PTY log | `virsh screenshot` (PNG of display) |
| Wait mechanism | PTY log polling | Sleep-based (no PTY log access) |
| Recording | `wf-recorder` (compositor video) | Poll-based screenshot → ffmpeg concat |
| Session isolation | Per-session Wayland runtime dir | Full VM isolation |

## Recording

Poll-based recording via `virsh screenshot`:

```bash
VMREC=${DROID_PLUGIN_ROOT}/scripts/windows/vm-record.sh

$VMREC start /tmp/demo.mp4 5    # 5 fps
# ... interact with vm-ctl.sh ...
$VMREC stop                      # encodes to MP4
```

## Terminal encoding reference

### VT mode (`ENABLE_VIRTUAL_TERMINAL_INPUT`)

| Key | Bytes | Note |
|---|---|---|
| Enter | `0D` | |
| Shift+Enter | `0D` | Indistinguishable from Enter in VT mode |
| Ctrl+C | `03` | With processed input off |
| Tab | `09` | |
| Shift+Tab | `1B 5B 5A` | |
| Arrows | `1B 5B A/B/C/D` | |

Modifier codes: 2=Shift, 3=Alt, 5=Ctrl, 6=Shift+Ctrl

### Win32 console API (`ReadKey` / `ReadConsoleInput`)

Lossless. Preserves VirtualKeyCode + exact modifier state (left/right Ctrl/Alt/Shift). Shift+Enter is VKey=13 + ShiftPressed (distinct from plain Enter). Use `win-key-dumper.ps1` for this level of detail.

## Troubleshooting

| Problem | Fix |
|---|---|
| VM won't get DHCP | Check `firewall_backend = "iptables"` in `/etc/libvirt/network.conf`, restart libvirtd |
| Keystrokes hitting host | You're using ydotool. Use `vm-ctl.sh press` (virsh send-key) instead |
| Screenshot is black | VM may be asleep. `vm-ctl.sh press space` to wake |
| SSH connection refused | sshd may have stopped. Use `vm-ctl.sh type` to run `Start-Service sshd` |

## Recovery

```bash
VMCTL=${DROID_PLUGIN_ROOT}/scripts/windows/vm-ctl.sh
$VMCTL press esc
$VMCTL shot /tmp/state.png
$VMCTL restore <snapshot>    # hard reset to known state
```
