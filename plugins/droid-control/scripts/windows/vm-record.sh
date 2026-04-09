#!/usr/bin/env bash
# vm-record.sh — screen recorder for the Windows VM
# Agent-only tool. Polls virsh screenshot and encodes to MP4 via ffmpeg.
#
# Usage:
#   vm-record.sh start [output.mp4] [fps]
#   vm-record.sh stop

set -euo pipefail

_CONN="${DROID_VM_CONN:-qemu:///system}"
_NAME="${DROID_VM_NAME:?vm-record: set DROID_VM_NAME}"
_PIDFILE="/tmp/.vm-rec.pid"
_METADIR="/tmp/.vm-rec-meta"

cmd_start() {
  local out="${1:-/tmp/vm-recording.mp4}"
  local fps="${2:-5}"

  if [[ -f "$_PIDFILE" ]] && kill -0 "$(cat "$_PIDFILE")" 2>/dev/null; then
    echo "vm-record: already recording (use 'stop' first)" >&2
    exit 1
  fi

  local state
  state=$(virsh --connect "$_CONN" domstate "$_NAME" 2>/dev/null)
  if [[ "$state" != "running" ]]; then
    echo "vm-record: VM not running" >&2
    exit 1
  fi

  local framedir
  framedir=$(mktemp -d /tmp/vm-rec-XXXXXX)
  mkdir -p "$_METADIR"
  echo "$framedir" > "$_METADIR/framedir"
  echo "$out"      > "$_METADIR/output"
  echo "$fps"      > "$_METADIR/fps"

  echo "vm-record: started → $out (${fps} fps)"

  nohup bash -c "
    n=0
    while true; do
      frame=\$(printf '%s/frame-%06d.png' '$framedir' \"\$n\")
      virsh --connect '$_CONN' screenshot '$_NAME' \"\$frame\" >/dev/null 2>&1 || true
      (( n++ ))
      sleep \$(echo 'scale=4; 1/$fps' | bc)
    done
  " >/dev/null 2>&1 &
  echo "$!" > "$_PIDFILE"
}

cmd_stop() {
  if [[ ! -f "$_PIDFILE" ]] || ! kill -0 "$(cat "$_PIDFILE")" 2>/dev/null; then
    echo "vm-record: no recording in progress" >&2
    exit 1
  fi

  kill "$(cat "$_PIDFILE")" 2>/dev/null
  wait "$(cat "$_PIDFILE")" 2>/dev/null || true
  rm -f "$_PIDFILE"

  local framedir out fps nframes
  framedir=$(cat "$_METADIR/framedir")
  out=$(cat "$_METADIR/output")
  fps=$(cat "$_METADIR/fps")
  nframes=$(find "$framedir" -name 'frame-*.png' 2>/dev/null | wc -l)

  if (( nframes == 0 )); then
    echo "vm-record: no frames captured" >&2
    rm -rf "$framedir" "$_METADIR"
    exit 1
  fi

  echo "vm-record: encoding $nframes frames → $out"
  ffmpeg -y -framerate "$fps" \
    -i "$framedir/frame-%06d.png" \
    -c:v libx264 -pix_fmt yuv420p \
    -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" \
    "$out" >/dev/null 2>&1

  local size
  size=$(du -h "$out" | cut -f1)
  echo "vm-record: $out ($size, $nframes frames)"

  rm -rf "$framedir" "$_METADIR"
}

case "${1:-}" in
  start) shift; cmd_start "$@" ;;
  stop)  cmd_stop ;;
  *)
    echo "usage: vm-record.sh start [output.mp4] [fps]"
    echo "       vm-record.sh stop"
    exit 1
    ;;
esac
