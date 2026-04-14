#!/usr/bin/env bash
# render-showcase.sh — Stage clips and render a Remotion showcase video
#
# Usage:
#   render-showcase.sh --props props.json --output /tmp/out.mp4 clip1.cast [clip2.mp4]
#   render-showcase.sh --props-inline '{"clips":...}' --output /tmp/out.mp4 clip1.cast
#
# What it does:
#   1. Converts .cast clips to .mp4 when needed, using a fidelity profile
#   2. Copies clip files into the Remotion public/ directory
#   3. Auto-patches clipDuration in props if missing (via ffprobe)
#   4. Runs npx remotion render Showcase
#   5. Cleans up staged/generated clips
#
# Prerequisites: ffmpeg, ffprobe, node, npm (with remotion deps installed)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REMOTION_DIR="${SCRIPT_DIR}/../remotion"
DROID_CLI_THEME='181818,e0d0c0,15161e,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,a9b1d6,414868,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,c0caf5'

PROPS_FILE="" PROPS_INLINE="" OUTPUT="" FIDELITY_OVERRIDE="auto" CLIPS=()
WORK_DIR="$(mktemp -d /tmp/render-showcase-XXXXXX)"
STAGED=()
STAGED_BASES=()
STAGED_SOURCES=()

cleanup() {
  local f
  for f in "${STAGED[@]}"; do
    rm -f "$f"
  done
  rm -rf "$WORK_DIR"
}

trap cleanup EXIT

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

normalize_props() {
  local props_json="$1"
  local output_file="$2"
  local fidelity_override="$3"
  local props_source="$4"
  PROPS_JSON="$props_json" python3 - "$output_file" "$fidelity_override" "$props_source" <<'PY'
import json
import os
import sys

output_file = sys.argv[1]
fidelity_override = sys.argv[2]
props_source = sys.argv[3]
raw_props = os.environ["PROPS_JSON"]

if not raw_props.strip():
    raise SystemExit(f"error: props JSON is empty: {props_source}")

try:
    props = json.loads(raw_props)
except json.JSONDecodeError as error:
    raise SystemExit(f"error: props JSON is invalid: {props_source}: {error.msg} at line {error.lineno} column {error.colno}")

if fidelity_override != "auto":
    props["fidelity"] = fidelity_override

fidelity = props.get("fidelity")
if fidelity is None:
    fidelity = "standard"
    props["fidelity"] = fidelity

if props.get("width") is None:
    props["width"] = 2560 if fidelity == "inspect" else 1920
if props.get("height") is None:
    props["height"] = 1440 if fidelity == "inspect" else 1080

with open(output_file, "w", encoding="utf-8") as f:
    json.dump(props, f)

print(fidelity)
PY
}

props_number() {
  local props_json="$1"
  local key="$2"
  PROPS_JSON="$props_json" python3 - "$key" <<'PY'
import json
import os
import sys

key = sys.argv[1]
value = json.loads(os.environ["PROPS_JSON"]).get(key)
print("" if value is None else value)
PY
}

rewrite_props_clips() {
  local props_json="$1"
  shift
  PROPS_JSON="$props_json" python3 - "$@" <<'PY'
import json
import os
import sys

props = json.loads(os.environ["PROPS_JSON"])
props["clips"] = list(sys.argv[1:])
print(json.dumps(props))
PY
}

cast_dimensions() {
  local cast_path="$1"
  python3 - "$cast_path" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    header = json.loads(f.readline())

print(header.get("width", 120), header.get("height", 36))
PY
}

convert_cast_clip() {
  local cast_clip="$1"
  local output_clip="$2"
  local fidelity="$3"
  local speed="$4"

  require_cmd agg
  require_cmd ffmpeg

  local cols rows agg_fps_cap agg_idle_limit ffmpeg_crf ffmpeg_preset gif_clip
  read -r cols rows < <(cast_dimensions "$cast_clip")

  case "$fidelity" in
    compact)
      agg_fps_cap="24"
      agg_idle_limit="3"
      ffmpeg_crf="21"
      ffmpeg_preset="medium"
      ;;
    inspect)
      agg_fps_cap="30"
      agg_idle_limit="5"
      ffmpeg_crf="14"
      ffmpeg_preset="slow"
      ;;
    standard)
      agg_fps_cap="30"
      agg_idle_limit="5"
      ffmpeg_crf="18"
      ffmpeg_preset="slow"
      ;;
    *)
      echo "error: unsupported fidelity profile: $fidelity" >&2
      exit 1
      ;;
  esac

  gif_clip="${output_clip%.mp4}.gif"
  agg --speed "$speed" \
    --renderer fontdue \
    --cols "$cols" \
    --rows "$rows" \
    --fps-cap "$agg_fps_cap" \
    --idle-time-limit "$agg_idle_limit" \
    --theme "$DROID_CLI_THEME" \
    "$cast_clip" \
    "$gif_clip"

  ffmpeg -y -i "$gif_clip" \
    -movflags +faststart \
    -pix_fmt yuv420p \
    -preset "$ffmpeg_preset" \
    -crf "$ffmpeg_crf" \
    -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" \
    "$output_clip" >/dev/null 2>&1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --props)        PROPS_FILE="$2";   shift 2 ;;
    --props-inline) PROPS_INLINE="$2"; shift 2 ;;
    --fidelity)     FIDELITY_OVERRIDE="$2"; shift 2 ;;
    --output|-o)    OUTPUT="$2";       shift 2 ;;
    -h|--help)      sed -n '2,10p' "$0" | sed 's/^# \?//'; exit 0 ;;
    -*)             echo "error: unknown option '$1'" >&2; exit 1 ;;
    *)              CLIPS+=("$1");     shift ;;
  esac
done

[[ -n "$OUTPUT" ]] || { echo "error: --output required" >&2; exit 1; }
[[ -n "$PROPS_FILE" || -n "$PROPS_INLINE" ]] || { echo "error: --props or --props-inline required" >&2; exit 1; }

# Read props JSON
if [[ -n "$PROPS_FILE" ]]; then
  [[ -r "$PROPS_FILE" ]] || { echo "error: props file is not readable: $PROPS_FILE" >&2; exit 1; }
  PROPS=$(cat "$PROPS_FILE")
  PROPS_SOURCE="$PROPS_FILE"
else
  PROPS="$PROPS_INLINE"
  PROPS_SOURCE="--props-inline"
fi

NORMALIZED_PROPS="${WORK_DIR}/props.json"
FIDELITY="$(normalize_props "$PROPS" "$NORMALIZED_PROPS" "$FIDELITY_OVERRIDE" "$PROPS_SOURCE")"
PROPS=$(cat "$NORMALIZED_PROPS")
SPEED="$(props_number "$PROPS" "speed")"
if [[ -z "$SPEED" ]]; then
  SPEED="1"
fi

RENDER_ARGS=()
case "$FIDELITY" in
  compact)
    RENDER_ARGS+=(--codec=h264 --crf=21 --jpeg-quality=92 --pixel-format=yuv420p --x264-preset=medium)
    ;;
  inspect)
    RENDER_ARGS+=(--codec=h264 --crf=14 --video-image-format=png --pixel-format=yuv420p --x264-preset=slow)
    ;;
  standard)
    RENDER_ARGS+=(--codec=h264 --crf=18 --jpeg-quality=96 --pixel-format=yuv420p --x264-preset=slow)
    ;;
  *)
    echo "error: unsupported fidelity profile: $FIDELITY" >&2
    exit 1
    ;;
esac

# Stage clips into public/
for clip in "${CLIPS[@]}"; do
  base=$(basename "$clip")
  source_clip="$clip"
  staged_base="$base"

  if [[ "$clip" == *.cast ]]; then
    staged_base="${base%.cast}.mp4"
    source_clip="${WORK_DIR}/${staged_base}"
    convert_cast_clip "$clip" "$source_clip" "$FIDELITY" "$SPEED"
  fi

  cp "$source_clip" "${REMOTION_DIR}/public/${staged_base}"
  STAGED+=("${REMOTION_DIR}/public/${staged_base}")
  STAGED_BASES+=("$staged_base")
  STAGED_SOURCES+=("$source_clip")
done

PROPS="$(rewrite_props_clips "$PROPS" "${STAGED_BASES[@]}")"

# Auto-detect clipDuration if not set in props (uses first clip)
HAS_DURATION=$(echo "$PROPS" | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d.get('clipDuration') else 'no')" 2>/dev/null || echo "no")
if [[ "$HAS_DURATION" == "no" && ${#STAGED_SOURCES[@]} -gt 0 ]]; then
  DUR=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "${STAGED_SOURCES[0]}" 2>/dev/null | head -1)
  if [[ -n "$DUR" ]]; then
    PROPS=$(echo "$PROPS" | python3 -c "
import sys, json
d = json.load(sys.stdin)
d['clipDuration'] = round(float('${DUR}'), 2)
json.dump(d, sys.stdout)
")
    echo "auto-detected clipDuration: ${DUR}s" >&2
  fi
fi

# Render
cd "$REMOTION_DIR"
npx remotion render Showcase --props="$PROPS" "${RENDER_ARGS[@]}" "$OUTPUT" 2>&1

echo "$OUTPUT"
