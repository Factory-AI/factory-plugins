#!/usr/bin/env bash
set -euo pipefail

# ── guard: dependencies ──────────────────────────────────────────────
command -v jq >/dev/null 2>&1 || {
  echo "Error: jq is required but not installed. Install it with: brew install jq"
  exit 1
}
command -v droid >/dev/null 2>&1 || {
  echo "Error: droid CLI is required but not found on PATH."
  exit 1
}

PROMPT="$*"

if [ -z "$PROMPT" ]; then
  echo "Usage: /rfspec <your prompt>"
  echo ""
  echo "Sends your prompt to three models in parallel (Opus, GPT, Gemini),"
  echo "then lets you pick the best spec or synthesize a combination."
  exit 1
fi

# ── persistent output directory ──────────────────────────────────────
# Results go to a stable path so the calling session can poll for them.
# The temp dir is only used for the prompt file passed to droid exec.
RFSPEC_HOME="${HOME}/.factory/rfspec/runs"
RUN_ID="$(date +%Y%m%d-%H%M%S)-$$"
OUTDIR="${RFSPEC_HOME}/${RUN_ID}"
mkdir -p "$OUTDIR"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Wrap the raw prompt with spec-generation framing so subagents produce
# a structured spec proposal, not code or casual analysis.
cat >"$TMPDIR/prompt.md" <<SPEC_FRAME
You are generating a structured implementation spec. Do NOT write code.

Produce a spec document with these sections:
- **Objective**: What this achieves (1-2 sentences)
- **Context**: Relevant background, constraints, prior art
- **Plan**: Concrete implementation steps with file paths and approach
- **Validation**: How to verify the implementation works
- **Risks / Open Questions**: What could go wrong, what needs clarification

Be specific -- name files, functions, libraries, and patterns. Avoid vague hand-waving.

---

${PROMPT}
SPEC_FRAME
cp "$TMPDIR/prompt.md" "$OUTDIR/prompt.md"

# Print the output path IMMEDIATELY so the calling agent can capture it
# even if the Execute call times out before the models finish.
echo "RFSPEC_RUN_DIR=${OUTDIR}"
echo "Firing three model calls in parallel. Poll ${OUTDIR}/results.md for output."

# ── models (id, label, max reasoning) ────────────────────────────────
MODEL_A="claude-opus-4-6"
LABEL_A="Opus 4.6"
RE_A="max"
MODEL_B="gpt-5.4"
LABEL_B="GPT-5.4"
RE_B="xhigh"
MODEL_C="gemini-3.1-pro-preview"
LABEL_C="Gemini 3.1 Pro"
RE_C="high"

# ── fire all three in parallel ───────────────────────────────────────
droid exec -m "$MODEL_A" -r "$RE_A" --auto high -f "$TMPDIR/prompt.md" -o json 2>/dev/null >"$OUTDIR/a.json" &
PID_A=$!
droid exec -m "$MODEL_B" -r "$RE_B" --auto high -f "$TMPDIR/prompt.md" -o json 2>/dev/null >"$OUTDIR/b.json" &
PID_B=$!
droid exec -m "$MODEL_C" -r "$RE_C" --auto high -f "$TMPDIR/prompt.md" -o json 2>/dev/null >"$OUTDIR/c.json" &
PID_C=$!

FAIL=""
wait $PID_A 2>/dev/null || FAIL="${FAIL}${LABEL_A} "
wait $PID_B 2>/dev/null || FAIL="${FAIL}${LABEL_B} "
wait $PID_C 2>/dev/null || FAIL="${FAIL}${LABEL_C} "

# ── extract results ──────────────────────────────────────────────────
extract() {
  local file="$1"
  if [ -s "$file" ]; then
    jq -r '.result // empty' "$file" 2>/dev/null || cat "$file"
  fi
}

RESULT_A=$(extract "$OUTDIR/a.json")
RESULT_B=$(extract "$OUTDIR/b.json")
RESULT_C=$(extract "$OUTDIR/c.json")

# ── write results to persistent file ─────────────────────────────────
SUCCESS=0
[ -n "$RESULT_A" ] && SUCCESS=$((SUCCESS + 1))
[ -n "$RESULT_B" ] && SUCCESS=$((SUCCESS + 1))
[ -n "$RESULT_C" ] && SUCCESS=$((SUCCESS + 1))

{
  echo "# rfspec results"
  echo ""
  echo "User request: ${PROMPT}"
  echo ""

  [ -n "$RESULT_A" ] && printf '## Option A -- %s\n\n%s\n\n' "$LABEL_A" "$RESULT_A"
  [ -n "$RESULT_B" ] && printf '## Option B -- %s\n\n%s\n\n' "$LABEL_B" "$RESULT_B"
  [ -n "$RESULT_C" ] && printf '## Option C -- %s\n\n%s\n\n' "$LABEL_C" "$RESULT_C"

  if [ -n "$FAIL" ]; then
    echo "> **Note:** The following models encountered errors: ${FAIL}"
    echo ""
  fi

  if [ "$SUCCESS" -gt 0 ]; then
    echo "---"
    echo ""
    echo "## Agent Instructions"
    echo ""
    echo "Analyze the specs above. Provide a brief comparison of each model's"
    echo "strengths and weaknesses -- compare them against each other, not individually."
    echo "Then use the AskUser tool to offer:"
    echo "- Use Option A (${LABEL_A}) as-is"
    echo "- Use Option B (${LABEL_B}) as-is"
    echo "- Use Option C (${LABEL_C}) as-is"
    echo "- Synthesize a refined spec combining the best of all three"
    echo "- No -- none of these work (explain why)"
    echo ""
    echo "CRITICAL: Do NOT save the spec directly. After the user picks an option"
    echo "or requests synthesis, use the ExitSpecMode tool to present the final"
    echo "spec content for review. Only save to specs/active/YYYY-MM-DD-<slug>.md"
    echo "AFTER the user approves the spec in spec mode. If rejected, gather"
    echo "feedback and revise."
  fi
} >"$OUTDIR/results.md"

# ── also print to stdout (for cases where timeout is large enough) ───
cat "$OUTDIR/results.md"

if [ "$SUCCESS" -eq 0 ]; then
  echo ""
  echo "Error: All three models failed. Check that your droid CLI is authenticated"
  echo "and the models (${MODEL_A}, ${MODEL_B}, ${MODEL_C}) are available."
  echo "STATUS=failed" >"$OUTDIR/done"
  exit 1
fi

# ── write completion sentinel ────────────────────────────────────────
echo "STATUS=complete" >"$OUTDIR/done"
echo ""
echo "Results written to: ${OUTDIR}/results.md"
