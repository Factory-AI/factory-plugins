#!/usr/bin/env bash
# mac-ctl.sh — macOS VM control for agents via QEMU monitor socket
# All input via QEMU HMP `sendkey`. Nothing touches the host compositor.
#
# Required env vars:
#   DROID_MAC_MONITOR  — path to QEMU monitor unix socket
#   DROID_MAC_SSH_HOST — SSH host alias for the macOS VM (from ~/.ssh/config)
#
# Optional:
#   DROID_MAC_BOOT_SCRIPT — path to the QEMU boot script (for `up` command)
#   DROID_MAC_BOOT_ARGS   — extra env vars for the boot script (space-separated KEY=VAL)
#   DROID_MAC_SHOT_DIR    — screenshot output directory (default: /tmp)
#
# Usage: mac-ctl.sh <command> [args]

set -euo pipefail

MONITOR="${DROID_MAC_MONITOR:?mac-ctl: set DROID_MAC_MONITOR (path to QEMU monitor socket)}"
SSH_HOST="${DROID_MAC_SSH_HOST:?mac-ctl: set DROID_MAC_SSH_HOST (SSH config host alias)}"
BOOT_SCRIPT="${DROID_MAC_BOOT_SCRIPT:-}"
BOOT_ARGS="${DROID_MAC_BOOT_ARGS:-}"
SHOT_DIR="${DROID_MAC_SHOT_DIR:-/tmp}"

# ── QEMU monitor helpers ────────────────────────────────────────
_qmp() {
  # Send a command to QEMU HMP via the monitor socket.
  # Uses socat with a short timeout; strips echo noise.
  printf '%s\n' "$1" | socat -T1 - "UNIX-CONNECT:${MONITOR}" 2>/dev/null \
    | grep -v '^QEMU\|^(qemu)' || true
}

_sendkey() {
  # QEMU HMP sendkey: keys are separated by dashes for chords.
  # e.g., _sendkey "shift-a" sends Shift+A
  _qmp "sendkey $1"
}

# ── char → QEMU keyname ─────────────────────────────────────────
# QEMU HMP sendkey uses its own key names (not KEY_A, just 'a').
declare -A CHAR_KEYS=(
  [a]=a [b]=b [c]=c [d]=d [e]=e
  [f]=f [g]=g [h]=h [i]=i [j]=j
  [k]=k [l]=l [m]=m [n]=n [o]=o
  [p]=p [q]=q [r]=r [s]=s [t]=t
  [u]=u [v]=v [w]=w [x]=x [y]=y
  [z]=z
  [0]=0 [1]=1 [2]=2 [3]=3 [4]=4
  [5]=5 [6]=6 [7]=7 [8]=8 [9]=9
  [" "]=spc
  [-]=minus    [=]=equal
  ["["]=bracket_left ["]"]=bracket_right
  [\\]=backslash [";"]=semicolon ["'"]=apostrophe
  [,]=comma    [.]=dot       [/]=slash
  ['`']=grave_accent
)

# ── shifted chars ────────────────────────────────────────────────
declare -A SHIFT_CHARS=(
  ["!"]=1  ["@"]=2  ["#"]=3  ['$']=4
  ["%"]=5  ["^"]=6  ["&"]=7  ["*"]=8
  ["("]=9  [")"]=0
  [_]=minus    ["+"]=equal
  ["{"]=bracket_left ["}"]=bracket_right
  ["|"]=backslash [":"]=semicolon ['"']=apostrophe
  ["<"]=comma    [">"]=dot        ["?"]=slash
  ["~"]=grave_accent
)

# ── named keys → QEMU HMP key names ─────────────────────────────
declare -A NAMED_KEYS=(
  [enter]=ret         [return]=ret
  [esc]=esc           [escape]=esc
  [tab]=tab           [space]=spc
  [backspace]=backspace [bs]=backspace
  [delete]=delete     [del]=delete
  [insert]=insert     [ins]=insert
  [home]=home         [end]=end
  [pageup]=pgup       [pgup]=pgup
  [pagedown]=pgdn     [pgdn]=pgdn
  [up]=up             [down]=down
  [left]=left         [right]=right
  [ctrl]=ctrl         [lctrl]=ctrl         [rctrl]=ctrl_r
  [alt]=alt           [lalt]=alt           [ralt]=alt_r
  [shift]=shift       [lshift]=shift       [rshift]=shift_r
  [super]=meta_l      [win]=meta_l         [meta]=meta_l
  [cmd]=meta_l        [command]=meta_l
  [f1]=f1   [f2]=f2   [f3]=f3   [f4]=f4
  [f5]=f5   [f6]=f6   [f7]=f7   [f8]=f8
  [f9]=f9   [f10]=f10 [f11]=f11 [f12]=f12
  [capslock]=caps_lock [caps]=caps_lock
)

# ── helpers ──────────────────────────────────────────────────────
ensure_running() {
  if [[ ! -S "$MONITOR" ]]; then
    echo "mac-ctl: monitor socket not found at $MONITOR" >&2
    exit 1
  fi
  local status
  status=$(_qmp "info status" | grep -o 'running\|paused' | head -1)
  if [[ "$status" != "running" ]]; then
    echo "mac-ctl: VM not running (status: ${status:-unknown})" >&2
    exit 1
  fi
}

resolve_key() {
  local name="${1,,}"
  if [[ -n "${NAMED_KEYS[$name]+_}" ]]; then
    echo "${NAMED_KEYS[$name]}"
  elif [[ ${#1} -eq 1 && "$1" =~ [a-zA-Z] ]]; then
    echo "${1,,}"
  else
    echo "mac-ctl: unknown key '$1'" >&2
    exit 1
  fi
}

type_char() {
  local ch="$1"
  local lower="${ch,,}"

  if [[ "$ch" =~ ^[A-Z]$ ]]; then
    _sendkey "shift-${lower}"
  elif [[ -n "${CHAR_KEYS[$ch]+_}" ]]; then
    _sendkey "${CHAR_KEYS[$ch]}"
  elif [[ -n "${SHIFT_CHARS[$ch]+_}" ]]; then
    _sendkey "shift-${SHIFT_CHARS[$ch]}"
  else
    echo "mac-ctl type: unsupported char '$ch'" >&2
    return 1
  fi
}

type_text() {
  local text="$1"
  local i ch
  for (( i=0; i<${#text}; i++ )); do
    ch="${text:$i:1}"
    type_char "$ch" || return 1
    sleep 0.08
  done
}

# ── commands ─────────────────────────────────────────────────────
cmd_up() {
  if [[ -S "$MONITOR" ]]; then
    echo "mac-ctl: monitor socket already exists -- VM may be running" >&2
    return 1
  fi
  if [[ -z "$BOOT_SCRIPT" ]]; then
    echo "mac-ctl: set DROID_MAC_BOOT_SCRIPT to start the VM" >&2
    exit 1
  fi
  # shellcheck disable=SC2086
  env MONITOR_SOCKET="$MONITOR" ENABLE_EVDEV_INPUT=0 $BOOT_ARGS \
    local logfile="${SHOT_DIR}/mac-ctl-boot.log"
    nohup "$BOOT_SCRIPT" > "$logfile" 2>&1 &
  echo "mac-ctl: started (PID $!, log: $logfile)"
}

cmd_down() {
  ensure_running
  # Graceful: Cmd+Opt+Eject → Sleep, or just use SSH
  ssh -o ConnectTimeout=5 "$SSH_HOST" 'sudo shutdown -h now' 2>/dev/null || true
  echo "mac-ctl: shutdown sent via SSH"
}

cmd_kill() {
  _qmp "quit"
  echo "mac-ctl: QEMU process terminated"
}

cmd_status() {
  if [[ ! -S "$MONITOR" ]]; then
    echo "not running (no monitor socket)"
    return
  fi
  _qmp "info status" | grep -oE 'running|paused|shutdown' | head -1 || echo "unknown"
}

cmd_type() {
  ensure_running
  [[ -z "${1:-}" ]] && { echo "usage: mac-ctl.sh type <text>" >&2; exit 1; }
  type_text "$*"
}

cmd_press() {
  ensure_running
  [[ -z "${1:-}" ]] && { echo "usage: mac-ctl.sh press <key> [key ...]" >&2; exit 1; }
  local chord=""
  for tok in "$@"; do
    local resolved
    resolved="$(resolve_key "$tok")"
    if [[ -z "$chord" ]]; then
      chord="$resolved"
    else
      chord="${chord}-${resolved}"
    fi
  done
  _sendkey "$chord"
}

cmd_shot() {
  ensure_running
  local ppm="${SHOT_DIR}/mac-screenshot-$$.ppm"
  local out="${1:-${SHOT_DIR}/mac-screenshot.png}"
  _qmp "screendump ${ppm}"
  sleep 0.3
  # Convert PPM to PNG (requires ffmpeg or ImageMagick on host)
  if command -v magick &>/dev/null; then
    magick "$ppm" "$out" 2>/dev/null
  elif command -v convert &>/dev/null; then
    convert "$ppm" "$out" 2>/dev/null
  elif command -v ffmpeg &>/dev/null; then
    ffmpeg -y -i "$ppm" "$out" 2>/dev/null
  else
    cp "$ppm" "$out"
    echo "mac-ctl: saved as PPM (install ImageMagick for PNG)" >&2
  fi
  rm -f "$ppm"
  echo "mac-ctl: $out"
}

cmd_terminal() {
  ensure_running
  # Spotlight → Terminal
  _sendkey "meta_l-spc"
  sleep 1
  type_text "Terminal"
  sleep 0.5
  _sendkey "ret"
  echo "mac-ctl: opening Terminal.app via Spotlight"
}

cmd_iterm() {
  ensure_running
  _sendkey "meta_l-spc"
  sleep 1
  type_text "iTerm"
  sleep 0.5
  _sendkey "ret"
  echo "mac-ctl: opening iTerm2 via Spotlight"
}

cmd_spotlight() {
  ensure_running
  local query="${1:-}"
  _sendkey "meta_l-spc"
  sleep 0.8
  if [[ -n "$query" ]]; then
    type_text "$query"
    sleep 0.5
    _sendkey "ret"
  fi
}

cmd_ssh_cmd() {
  ensure_running
  if [[ $# -eq 0 ]]; then
    echo "usage: mac-ctl.sh ssh <command>" >&2
    echo "  NOTE: SSH is for deployment/scripting ONLY." >&2
    echo "  DO NOT use for TUI testing -- SSH PTY layer distorts input." >&2
    exit 1
  fi
  ssh -o ConnectTimeout=5 "$SSH_HOST" "$@"
}

cmd_wait_boot() {
  local timeout="${1:-120}"
  local elapsed=0
  echo -n "mac-ctl: waiting for SSH..."
  while (( elapsed < timeout )); do
    if ssh -o ConnectTimeout=3 "$SSH_HOST" 'true' 2>/dev/null; then
      echo " ready (${elapsed}s)"
      return 0
    fi
    sleep 5
    elapsed=$((elapsed + 5))
    echo -n "."
  done
  echo " timeout after ${timeout}s" >&2
  return 1
}

cmd_keys() {
  echo "Named keys for 'press':"
  printf '%s\n' "${!NAMED_KEYS[@]}" | sort | column
}

cmd_help() {
  cat <<'EOF'
usage: mac-ctl.sh <command> [args]

  Lifecycle:  up | down | kill | status | wait-boot [timeout]
  Input:      type <text> | press <key> [key ...]
  Capture:    shot [path] | wait [secs]
  Shortcuts:  terminal | iterm | spotlight [query]
  Info:       keys | help

  SSH (deployment only -- NOT for TUI testing):
    ssh <cmd>     run a command in the VM via SSH

  macOS-specific notes:
    - `cmd` / `command` maps to the macOS Command key (meta_l)
    - Chords: `press cmd c` sends Cmd+C, `press cmd shift 3` sends screenshot
    - `up` requires DROID_MAC_BOOT_SCRIPT to be set
    - `shot` outputs PPM → PNG (needs ImageMagick or ffmpeg on host)
    - `wait-boot` polls SSH until the VM is ready (default 120s timeout)
EOF
}

# ── dispatch ─────────────────────────────────────────────────────
case "${1:-help}" in
  up)         cmd_up ;;
  down)       cmd_down ;;
  kill)       cmd_kill ;;
  status)     cmd_status ;;
  wait-boot)  shift; cmd_wait_boot "$@" ;;
  type)       shift; cmd_type "$@" ;;
  press)      shift; cmd_press "$@" ;;
  shot)       shift; cmd_shot "$@" ;;
  wait)       sleep "${2:-1}" ;;
  terminal)   cmd_terminal ;;
  iterm)      cmd_iterm ;;
  spotlight)  shift; cmd_spotlight "$@" ;;
  ssh)        shift; cmd_ssh_cmd "$@" ;;
  keys)       cmd_keys ;;
  help|-h|--help) cmd_help ;;
  *)          echo "mac-ctl: unknown command '$1'" >&2; cmd_help >&2; exit 1 ;;
esac
