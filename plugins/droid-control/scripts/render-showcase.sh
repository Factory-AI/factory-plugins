#!/usr/bin/env bash
# render-showcase.sh — Stage clips and render (or preview) a Remotion showcase video
#
# Usage:
#   render-showcase.sh --props props.json --output /tmp/out.mp4 clip1.cast [clip2.mp4]
#   render-showcase.sh --props-inline '{"clips":...}' --output /tmp/out.mp4 clip1.cast
#   render-showcase.sh --props props.json --still 150 --output /tmp/frame.png clip1.cast
#
# What it does:
#   1. Accepts .cast, .mp4, .webm clips; converts .cast to .mp4 without altering its timeline
#   2. Stages every clip as clip-<index>.<ext> inside a per-render directory under remotion/public/
#   3. Resolves fidelity (omitted => side-by-side: inspect, single: standard), width/height, speed
#   4. Sets clipDuration to the longest source duration in source seconds (ffprobe)
#   5. Runs npx remotion render Showcase (or remotion still with --still <frame>)
#   6. Removes only its own staged directory and work directory on exit
#
# Cancellation: SIGINT/SIGTERM delivered to the process group (Ctrl-C, task cancellation)
# stops the render and cleans up. A signal sent to this script's PID alone is deferred
# until the foreground `npx remotion` child exits; cleanup then still runs.
#
# Prerequisites: ffmpeg, ffprobe, node, npm (with remotion deps installed); agg for .cast inputs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REMOTION_DIR="${SCRIPT_DIR}/../remotion"
DROID_CLI_THEME='181818,e0d0c0,15161e,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,a9b1d6,414868,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,c0caf5'

PROPS_FILE="" PROPS_INLINE="" OUTPUT="" FIDELITY_OVERRIDE="" STILL_FRAME="" CLIPS=()
WORK_DIR="$(mktemp -d /tmp/render-showcase-XXXXXX)"
STAGE_DIR=""

cleanup() {
  rm -rf "$WORK_DIR"
  if [[ -n "$STAGE_DIR" ]]; then
    rm -rf "$STAGE_DIR"
  fi
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

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
import math
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

if fidelity_override:
    props["fidelity"] = fidelity_override

fidelity = props.get("fidelity")
if fidelity is None:
    fidelity = "inspect" if props.get("layout") == "side-by-side" else "standard"
    props["fidelity"] = fidelity
if fidelity not in ("compact", "standard", "inspect"):
    raise SystemExit(f"error: unsupported fidelity profile: {fidelity} (use compact, standard, or inspect, or omit it)")

speed = props.get("speed", 1)
if (
    not isinstance(speed, (int, float))
    or isinstance(speed, bool)
    or not math.isfinite(speed)
    or speed <= 0
):
    raise SystemExit(f"error: speed must be a positive finite number: {speed!r}")
props["speed"] = speed

if props.get("width") is None:
    props["width"] = 2560 if fidelity == "inspect" else 1920
if props.get("height") is None:
    props["height"] = 1440 if fidelity == "inspect" else 1080

with open(output_file, "w", encoding="utf-8") as f:
    json.dump(props, f)

print(fidelity)
PY
}

# Rewrites clips to their staged paths and sets clipDuration to the longest source duration.
finalize_props() {
  local props_json="$1"
  local clip_duration="$2"
  shift 2
  PROPS_JSON="$props_json" python3 - "$clip_duration" "$@" <<'PY'
import json
import os
import sys

props = json.loads(os.environ["PROPS_JSON"])
props["clipDuration"] = round(float(sys.argv[1]), 2)
props["clips"] = list(sys.argv[2:])
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

media_duration() {
  local media_path="$1"
  local duration
  duration="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$media_path" 2>/dev/null | head -1)"
  [[ -n "$duration" && "$duration" != "N/A" ]] || {
    echo "error: ffprobe could not read a duration from clip: $media_path" >&2
    exit 1
  }
  echo "$duration"
}

convert_cast_clip() {
  local cast_clip="$1"
  local output_clip="$2"
  local fidelity="$3"

  require_cmd agg
  require_cmd ffmpeg

  local cols rows agg_fps_cap ffmpeg_crf ffmpeg_preset gif_clip
  read -r cols rows < <(cast_dimensions "$cast_clip")

  case "$fidelity" in
    compact)  agg_fps_cap="24"; ffmpeg_crf="21"; ffmpeg_preset="medium" ;;
    inspect)  agg_fps_cap="30"; ffmpeg_crf="14"; ffmpeg_preset="slow" ;;
    standard) agg_fps_cap="30"; ffmpeg_crf="18"; ffmpeg_preset="slow" ;;
  esac

  # The composition applies `speed` to every clip, so the cast must keep its own timeline
  # here: agg's default idle limit (5s) would silently compress pauses in casts only.
  gif_clip="${output_clip%.mp4}.gif"
  agg --speed 1 \
    --idle-time-limit 1000000000 \
    --renderer fontdue \
    --cols "$cols" \
    --rows "$rows" \
    --fps-cap "$agg_fps_cap" \
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
    --still)        STILL_FRAME="$2";  shift 2 ;;
    --output|-o)    OUTPUT="$2";       shift 2 ;;
    -h|--help)      sed -n '2,16p' "$0" | sed 's/^# \?//'; exit 0 ;;
    -*)             echo "error: unknown option '$1'" >&2; exit 1 ;;
    *)              CLIPS+=("$1");     shift ;;
  esac
done

[[ -n "$OUTPUT" ]] || { echo "error: --output required" >&2; exit 1; }
[[ -n "$PROPS_FILE" || -n "$PROPS_INLINE" ]] || { echo "error: --props or --props-inline required" >&2; exit 1; }
[[ ${#CLIPS[@]} -gt 0 ]] || { echo "error: at least one clip path required" >&2; exit 1; }
[[ -z "$STILL_FRAME" || "$STILL_FRAME" =~ ^[0-9]+$ ]] || { echo "error: --still requires a frame number" >&2; exit 1; }

for clip in "${CLIPS[@]}"; do
  [[ -r "$clip" ]] || { echo "error: clip is not readable: $clip" >&2; exit 1; }
  case "$clip" in
    *.cast|*.mp4|*.webm) ;;
    *) echo "error: unsupported clip type: $clip (clips are .cast, .mp4, or .webm; stills belong in the screenshot artifact path)" >&2; exit 1 ;;
  esac
done

require_cmd ffprobe

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

# --color-space=bt709 is what makes --pixel-format=yuv420p hold in the file: with Remotion's
# default color space, JPEG frames encode as full-range yuvj420p.
RENDER_ARGS=(--codec=h264 --pixel-format=yuv420p --color-space=bt709)
case "$FIDELITY" in
  compact)  RENDER_ARGS+=(--crf=21 --jpeg-quality=92 --x264-preset=medium) ;;
  inspect)  RENDER_ARGS+=(--crf=14 --video-image-format=png --x264-preset=slow) ;;
  standard) RENDER_ARGS+=(--crf=18 --jpeg-quality=96 --x264-preset=slow) ;;
esac

# Stage clips into a directory owned by this render only. Names are indexed so two inputs
# that share a basename (before/recording.mp4, after/recording.mp4) stay distinct.
STAGE_DIR="$(mktemp -d "${REMOTION_DIR}/public/render-XXXXXX")"
STAGE_REL="$(basename "$STAGE_DIR")"
STAGED_REFS=()
CLIP_DURATION="0"

for i in "${!CLIPS[@]}"; do
  clip="${CLIPS[$i]}"
  ext="${clip##*.}"
  if [[ "$ext" == "cast" ]]; then
    ext="mp4"
    staged_clip="${STAGE_DIR}/clip-${i}.${ext}"
    convert_cast_clip "$clip" "${WORK_DIR}/clip-${i}.mp4" "$FIDELITY"
    mv "${WORK_DIR}/clip-${i}.mp4" "$staged_clip"
  else
    staged_clip="${STAGE_DIR}/clip-${i}.${ext}"
    cp "$clip" "$staged_clip"
  fi
  STAGED_REFS+=("${STAGE_REL}/clip-${i}.${ext}")

  duration="$(media_duration "$staged_clip")"
  CLIP_DURATION="$(python3 -c 'import sys; print(max(float(sys.argv[1]), float(sys.argv[2])))' "$CLIP_DURATION" "$duration")"
done

PROPS="$(finalize_props "$PROPS" "$CLIP_DURATION" "${STAGED_REFS[@]}")"
echo "clipDuration (longest source): ${CLIP_DURATION}s" >&2

cd "$REMOTION_DIR"
if [[ -n "$STILL_FRAME" ]]; then
  npx remotion still Showcase --props="$PROPS" --frame="$STILL_FRAME" "$OUTPUT" 2>&1
else
  npx remotion render Showcase --props="$PROPS" "${RENDER_ARGS[@]}" "$OUTPUT" 2>&1
fi

echo "$OUTPUT"
