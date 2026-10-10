#!/usr/bin/env bash
# render-showcase.sh — Render (or preview) a droid-control showcase video
#
# Usage:
#   render-showcase.sh --props props.json --output /tmp/out.mp4 clip1.cast [clip2.mp4]
#   render-showcase.sh --props-inline '{"title":...}' --output /tmp/out.mp4 clip1.cast
#   render-showcase.sh --props props.json --still 150 --output /tmp/frame.png clip1.cast
#   render-showcase.sh --props props.json --fidelity compact --output /tmp/out.mp4 clip1.mp4
#
# Builds the fframes composition in ../fframes (cargo; a no-op once built) and runs it. The
# droid-showcase binary owns everything else: props validation and defaults, clip staging
# (.cast via agg), probing, rendering and the H.264 encode. It prints its resolved render
# plan as one `showcase plan: {...}` line on stderr and the output path on stdout.
#
# Prerequisites: cargo (stable Rust), clang/libclang and libx264 for the first build;
# ffmpeg and ffprobe; agg for .cast inputs.

set -euo pipefail

FFRAMES_DIR="$(cd "$(dirname "$0")/../fframes" && pwd)"

case "${1:-}" in
  -h|--help) sed -n '2,16p' "$0" | sed 's/^# \?//'; exit 0 ;;
esac

command -v cargo >/dev/null 2>&1 || {
  echo "error: required command not found: cargo (install Rust from https://rustup.rs)" >&2
  exit 1
}

cargo build --release --quiet --manifest-path "${FFRAMES_DIR}/Cargo.toml" >&2
exec "${FFRAMES_DIR}/target/release/droid-showcase" "$@"
