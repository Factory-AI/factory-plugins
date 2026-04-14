#!/usr/bin/env bash
# vm-ctl.sh — self-contained VM control for agents
# All input via virsh send-key. Nothing touches the host compositor.
#
# Required env vars:
#   DROID_VM_NAME      — libvirt domain name
#   DROID_VM_SSH_KEY   — path to SSH private key for the VM
#   DROID_VM_SSH_USER  — SSH username in the VM
#
# Optional:
#   DROID_VM_CONN      — libvirt connection URI (default: qemu:///system)
#   DROID_VM_CAPTURE   — remote capture dir (default: C:/capture)
#
# Usage: vm-ctl.sh <command> [args]

set -euo pipefail

CONN="${DROID_VM_CONN:-qemu:///system}"
NAME="${DROID_VM_NAME:?vm-ctl: set DROID_VM_NAME (libvirt domain name)}"
SSH_KEY="${DROID_VM_SSH_KEY:?vm-ctl: set DROID_VM_SSH_KEY (path to SSH key)}"
SSH_USER="${DROID_VM_SSH_USER:?vm-ctl: set DROID_VM_SSH_USER (e.g. droid)}"
VM_CAPTURE_DIR="${DROID_VM_CAPTURE:-C:/capture}"

_ssh() {
  local ip
  ip=$(virsh --connect "$CONN" domifaddr "$NAME" 2>/dev/null | awk '/ipv4/{print $4}' | cut -d/ -f1)
  [[ -z "$ip" ]] && { echo "vm-ctl: can't resolve VM IP" >&2; exit 1; }
  ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    -o ConnectTimeout=5 "$SSH_USER@$ip" "$@" 2>/dev/null
}

_scp_to() {
  local src="$1" dst="$2"
  local ip
  ip=$(virsh --connect "$CONN" domifaddr "$NAME" 2>/dev/null | awk '/ipv4/{print $4}' | cut -d/ -f1)
  scp -i "$SSH_KEY" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    "$src" "$SSH_USER@$ip:$dst" 2>/dev/null
}

# ── char → KEY_* ─────────────────────────────────────────────────
declare -A CHAR_KEYS=(
  [a]=KEY_A [b]=KEY_B [c]=KEY_C [d]=KEY_D [e]=KEY_E
  [f]=KEY_F [g]=KEY_G [h]=KEY_H [i]=KEY_I [j]=KEY_J
  [k]=KEY_K [l]=KEY_L [m]=KEY_M [n]=KEY_N [o]=KEY_O
  [p]=KEY_P [q]=KEY_Q [r]=KEY_R [s]=KEY_S [t]=KEY_T
  [u]=KEY_U [v]=KEY_V [w]=KEY_W [x]=KEY_X [y]=KEY_Y
  [z]=KEY_Z
  [0]=KEY_0 [1]=KEY_1 [2]=KEY_2 [3]=KEY_3 [4]=KEY_4
  [5]=KEY_5 [6]=KEY_6 [7]=KEY_7 [8]=KEY_8 [9]=KEY_9
  [" "]=KEY_SPACE
  [-]=KEY_MINUS    [=]=KEY_EQUAL
  ["["]=KEY_LEFTBRACE ["]"]=KEY_RIGHTBRACE
  [\\]=KEY_BACKSLASH [";"]=KEY_SEMICOLON ["'"]=KEY_APOSTROPHE
  [,]=KEY_COMMA    [.]=KEY_DOT       [/]=KEY_SLASH
  ['`']=KEY_GRAVE
)

# ── shifted chars ────────────────────────────────────────────────
declare -A SHIFT_CHARS=(
  ["!"]=KEY_1  ["@"]=KEY_2  ["#"]=KEY_3  ['$']=KEY_4
  ["%"]=KEY_5  ["^"]=KEY_6  ["&"]=KEY_7  ["*"]=KEY_8
  ["("]=KEY_9  [")"]=KEY_0
  [_]=KEY_MINUS    ["+"]=KEY_EQUAL
  ["{"]=KEY_LEFTBRACE ["}"]=KEY_RIGHTBRACE
  ["|"]=KEY_BACKSLASH [":"]=KEY_SEMICOLON ['"']=KEY_APOSTROPHE
  ["<"]=KEY_COMMA    [">"]=KEY_DOT        ["?"]=KEY_SLASH
  ["~"]=KEY_GRAVE
)

# ── named keys ───────────────────────────────────────────────────
declare -A NAMED_KEYS=(
  [enter]=KEY_ENTER     [return]=KEY_ENTER
  [esc]=KEY_ESC         [escape]=KEY_ESC
  [tab]=KEY_TAB         [space]=KEY_SPACE
  [backspace]=KEY_BACKSPACE [bs]=KEY_BACKSPACE
  [delete]=KEY_DELETE   [del]=KEY_DELETE
  [insert]=KEY_INSERT   [ins]=KEY_INSERT
  [home]=KEY_HOME       [end]=KEY_END
  [pageup]=KEY_PAGEUP   [pgup]=KEY_PAGEUP
  [pagedown]=KEY_PAGEDOWN [pgdn]=KEY_PAGEDOWN
  [up]=KEY_UP           [down]=KEY_DOWN
  [left]=KEY_LEFT       [right]=KEY_RIGHT
  [ctrl]=KEY_LEFTCTRL   [lctrl]=KEY_LEFTCTRL  [rctrl]=KEY_RIGHTCTRL
  [alt]=KEY_LEFTALT     [lalt]=KEY_LEFTALT     [ralt]=KEY_RIGHTALT
  [shift]=KEY_LEFTSHIFT [lshift]=KEY_LEFTSHIFT [rshift]=KEY_RIGHTSHIFT
  [super]=KEY_LEFTMETA  [win]=KEY_LEFTMETA     [meta]=KEY_LEFTMETA
  [f1]=KEY_F1   [f2]=KEY_F2   [f3]=KEY_F3   [f4]=KEY_F4
  [f5]=KEY_F5   [f6]=KEY_F6   [f7]=KEY_F7   [f8]=KEY_F8
  [f9]=KEY_F9   [f10]=KEY_F10 [f11]=KEY_F11 [f12]=KEY_F12
  [capslock]=KEY_CAPSLOCK [caps]=KEY_CAPSLOCK
  [printscreen]=KEY_SYSRQ [prtsc]=KEY_SYSRQ
  [scrolllock]=KEY_SCROLLLOCK [pause]=KEY_PAUSE
)

# ── helpers ──────────────────────────────────────────────────────
send_key() {
  virsh --connect "$CONN" send-key "$NAME" --codeset linux "$@" 2>/dev/null
}

ensure_running() {
  local state
  state=$(virsh --connect "$CONN" domstate "$NAME" 2>/dev/null || true)
  if [[ "$state" != "running" ]]; then
    echo "vm-ctl: not running" >&2
    exit 1
  fi
}

resolve_key() {
  local name="${1,,}"
  if [[ -n "${NAMED_KEYS[$name]+_}" ]]; then
    echo "${NAMED_KEYS[$name]}"
  elif [[ "$1" == KEY_* ]]; then
    echo "$1"
  elif [[ ${#1} -eq 1 && "$1" =~ [a-zA-Z] ]]; then
    echo "KEY_${1^^}"
  else
    echo "vm-ctl: unknown key '$1'" >&2
    exit 1
  fi
}

type_char() {
  local ch="$1"
  local lower="${ch,,}"

  if [[ "$ch" =~ ^[A-Z]$ ]]; then
    send_key KEY_LEFTSHIFT "${CHAR_KEYS[$lower]}"
  elif [[ -n "${CHAR_KEYS[$ch]+_}" ]]; then
    send_key "${CHAR_KEYS[$ch]}"
  elif [[ -n "${SHIFT_CHARS[$ch]+_}" ]]; then
    send_key KEY_LEFTSHIFT "${SHIFT_CHARS[$ch]}"
  else
    echo "vm-ctl type: unsupported char '$ch'" >&2
    return 1
  fi
}

type_text() {
  local text="$1"
  local i ch
  for (( i=0; i<${#text}; i++ )); do
    ch="${text:$i:1}"
    type_char "$ch" || return 1
    sleep 0.05
  done
}

# ── commands ─────────────────────────────────────────────────────
cmd_up()      { virsh --connect "$CONN" start "$NAME" 2>/dev/null && echo "vm-ctl: started" || echo "vm-ctl: already running or failed" >&2; }
cmd_down()    { virsh --connect "$CONN" shutdown "$NAME" 2>/dev/null && echo "vm-ctl: shutting down" || echo "vm-ctl: not running" >&2; }
cmd_kill()    { virsh --connect "$CONN" destroy "$NAME" 2>/dev/null && echo "vm-ctl: force-stopped" || echo "vm-ctl: not running" >&2; }
cmd_reboot()  { virsh --connect "$CONN" reboot "$NAME" 2>/dev/null && echo "vm-ctl: rebooting" || echo "vm-ctl: not running" >&2; }
cmd_status()  { virsh --connect "$CONN" domstate "$NAME" 2>/dev/null || echo "vm-ctl: not found" >&2; }

cmd_type() {
  ensure_running
  [[ -z "${1:-}" ]] && { echo "usage: vm-ctl.sh type <text>" >&2; exit 1; }
  type_text "$*"
}

cmd_press() {
  ensure_running
  [[ -z "${1:-}" ]] && { echo "usage: vm-ctl.sh press <key> [key ...]" >&2; exit 1; }
  local keys=()
  for tok in "$@"; do
    keys+=("$(resolve_key "$tok")")
  done
  send_key "${keys[@]}"
}

cmd_login() {
  ensure_running
  local pwd="${1:-factory}"
  send_key KEY_ENTER
  sleep 2
  type_text "$pwd"
  sleep 0.3
  send_key KEY_ENTER
  echo "vm-ctl: login sent"
}

cmd_shot() {
  ensure_running
  local out="${1:-/tmp/vm-screenshot.png}"
  virsh --connect "$CONN" screenshot "$NAME" "$out" >/dev/null 2>&1 \
    && echo "vm-ctl: $out" \
    || { echo "vm-ctl: screenshot failed" >&2; exit 1; }
}

cmd_pwsh() {
  ensure_running
  send_key KEY_LEFTMETA
  sleep 1.5
  type_text "powershell"
  sleep 1
  send_key KEY_ENTER
  echo "vm-ctl: opening PowerShell"
}

cmd_wt() {
  ensure_running
  send_key KEY_LEFTMETA
  sleep 1.5
  type_text "terminal"
  sleep 1
  send_key KEY_ENTER
  echo "vm-ctl: opening Windows Terminal"
}

cmd_snap() {
  local name="${1:-$(date +%Y%m%d-%H%M%S)}"
  virsh --connect "$CONN" snapshot-create-as "$NAME" "$name" \
    && echo "vm-ctl: snapshot '$name' created"
}

cmd_snaps()   { virsh --connect "$CONN" snapshot-list "$NAME"; }

cmd_restore() {
  [[ -z "${1:-}" ]] && { echo "usage: vm-ctl.sh restore <snapshot>" >&2; exit 1; }
  virsh --connect "$CONN" snapshot-revert "$NAME" "$1" \
    && echo "vm-ctl: reverted to '$1'"
}

cmd_ip()      { virsh --connect "$CONN" domifaddr "$NAME" 2>/dev/null || echo "vm-ctl: couldn't get IP" >&2; }

cmd_ssh() {
  ensure_running
  if [[ $# -eq 0 ]]; then
    echo "usage: vm-ctl.sh ssh <command>" >&2
    echo "  NOTE: SSH is for deployment/scripting ONLY." >&2
    echo "  DO NOT use for TUI testing -- SSH PTY layer distorts input." >&2
    exit 1
  fi
  _ssh "$@"
}

cmd_deploy() {
  ensure_running
  local script_dir
  script_dir="$(cd "$(dirname "$0")" && pwd)"
  _scp_to "$script_dir/win-vt-dumper.ps1" "$VM_CAPTURE_DIR/"
  _scp_to "$script_dir/win-key-dumper.ps1" "$VM_CAPTURE_DIR/"
  echo "vm-ctl: deployed capture scripts to $VM_CAPTURE_DIR"
}

cmd_run() {
  ensure_running
  [[ -z "${1:-}" ]] && { echo "usage: vm-ctl.sh run <ps1-script> [args]" >&2; exit 1; }
  local script="$1"; shift
  _ssh "powershell -ExecutionPolicy Bypass -File $VM_CAPTURE_DIR/$script $*"
}

cmd_keys() {
  echo "Named keys for 'press':"
  printf '%s\n' "${!NAMED_KEYS[@]}" | sort | column
}

cmd_help() {
  cat <<'EOF'
usage: vm-ctl.sh <command> [args]

  Lifecycle:  up | down | kill | reboot | status
  Input:      type <text> | press <key> [key ...] | login [pwd]
  Capture:    shot [path] | wait [secs]
  Shortcuts:  pwsh | wt
  Snapshots:  snap [name] | snaps | restore <name>
  Info:       ip | keys | help

  SSH (deployment only -- NOT for TUI testing):
    ssh <cmd>     run a command in the VM via SSH
    deploy        push capture scripts to C:\capture
    run <ps1>     execute a PowerShell script in C:\capture
EOF
}

# ── dispatch ─────────────────────────────────────────────────────
case "${1:-help}" in
  up)       cmd_up ;;
  down)     cmd_down ;;
  kill)     cmd_kill ;;
  reboot)   cmd_reboot ;;
  status)   cmd_status ;;
  type)     shift; cmd_type "$@" ;;
  press)    shift; cmd_press "$@" ;;
  login)    shift; cmd_login "$@" ;;
  shot)     shift; cmd_shot "$@" ;;
  wait)     sleep "${2:-1}" ;;
  pwsh)     cmd_pwsh ;;
  wt)       cmd_wt ;;
  snap)     shift; cmd_snap "$@" ;;
  snaps)    cmd_snaps ;;
  restore)  shift; cmd_restore "$@" ;;
  ip)       cmd_ip ;;
  keys)     cmd_keys ;;
  ssh)      shift; cmd_ssh "$@" ;;
  deploy)   cmd_deploy ;;
  run)      shift; cmd_run "$@" ;;
  help|-h|--help) cmd_help ;;
  *)        echo "vm-ctl: unknown command '$1'" >&2; cmd_help >&2; exit 1 ;;
esac
