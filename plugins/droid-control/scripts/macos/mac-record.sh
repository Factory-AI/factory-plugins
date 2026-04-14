#!/usr/bin/env bash
# mac-record.sh — poll-based screen recorder for macOS QEMU VM
# Takes periodic screenshots via QEMU monitor and encodes to MP4.
#
# Required env vars:
#   DROID_MAC_MONITOR — path to QEMU monitor unix socket
#
# Usage:
#   mac-record.sh start <output.mp4> [fps]
#   mac-record.sh stop

set -euo pipefail

MONITOR="${DROID_MAC_MONITOR:?mac-record: set DROID_MAC_MONITOR}"
PID_FILE="/tmp/mac-record.pid"
FRAME_DIR="/tmp/mac-record-frames"

_qmp() {
  printf '%s\n' "$1" | socat -T1 - "UNIX-CONNECT:${MONITOR}" 2>/dev/null | grep -v '^QEMU\|^(qemu)' || true
}

cmd_start() {
  local output="${1:?usage: mac-record.sh start <output.mp4> [fps]}"
  local fps="${2:-5}"
  local interval
  interval=$(awk "BEGIN{printf \"%.3f\", 1/$fps}")

  [[ -f "$PID_FILE" ]] && { echo "mac-record: already recording (PID $(cat "$PID_FILE"))" >&2; exit 1; }

  rm -rf "$FRAME_DIR"
  mkdir -p "$FRAME_DIR"

  (
    local n=0
    while true; do
      local frame
      frame=$(printf '%s/frame_%06d.ppm' "$FRAME_DIR" "$n")
      _qmp "screendump ${frame}" >/dev/null
      n=$((n + 1))
      sleep "$interval"
    done
  ) &

  local pid=$!
  echo "$pid" > "$PID_FILE"
  echo "$output" > /tmp/mac-record-output
  echo "$fps" > /tmp/mac-record-fps
  echo "mac-record: started (PID $pid, ${fps} fps → $output)"
}

cmd_stop() {
  [[ ! -f "$PID_FILE" ]] && { echo "mac-record: not recording" >&2; exit 1; }

  local pid
  pid=$(cat "$PID_FILE")
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -f "$PID_FILE"

  local output fps
  output=$(cat /tmp/mac-record-output)
  fps=$(cat /tmp/mac-record-fps)

  local frame_count
  frame_count=$(find "$FRAME_DIR" -name 'frame_*.ppm' 2>/dev/null | wc -l)
  if [[ "$frame_count" -eq 0 ]]; then
    echo "mac-record: no frames captured" >&2
    exit 1
  fi

  echo "mac-record: encoding $frame_count frames → $output"
  ffmpeg -y -framerate "$fps" -i "${FRAME_DIR}/frame_%06d.ppm" \
    -c:v libx264 -pix_fmt yuv420p -preset fast -crf 23 \
    "$output" 2>/dev/null

  rm -rf "$FRAME_DIR" /tmp/mac-record-output /tmp/mac-record-fps
  echo "mac-record: $output"
}

case "${1:-}" in
  start) shift; cmd_start "$@" ;;
  stop)  cmd_stop ;;
  *)     echo "usage: mac-record.sh start <output.mp4> [fps] | stop" >&2; exit 1 ;;
esac
