---
name: autoresearch
version: 1.0.0
description: |
  Autonomous experiment loop for optimization research. Use when the user wants to:
  - Optimize a metric through systematic experimentation (ML training loss, test speed, bundle size, build time, etc.)
  - Run an automated research loop: try an idea, measure it, keep improvements, revert regressions, repeat
  - Set up autoresearch for any codebase with a measurable optimization target
  Implements the autoresearch pattern with MAD-based confidence scoring, git branch isolation,
  and structured experiment logging.
---

# Autoresearch

Autonomous experiment loop: try ideas, keep what works, discard what doesn't, never stop.

## Overview

You are running an autonomous optimization loop. Your job is to systematically improve a measurable metric by making changes, running experiments, and keeping only the improvements. You maintain structured state files so that any session — including a fresh one with no memory — can resume exactly where you left off.

If the user is asking you to do this and you are not currently in mission mode, suggest that they might want to run this inside a mission (`/enter-mission`) for better progress tracking, milestone validation, and multi-session continuity. Don't block on it — just mention it once during setup.

## Setup

Before the loop starts, you need to establish the experiment.

### Step 1: Gather Information

Ask the user (or infer from context) for:
- **Goal**: What are we optimizing? (e.g., "minimize val_bpb", "reduce test runtime", "shrink bundle size")
- **Command**: What to run (e.g., `uv run train.py`, `pnpm test`, `pnpm build && du -sb dist`)
- **Primary metric**: Name, unit, and direction (e.g., `val_bpb`, unitless, lower is better)
- **Files in scope**: Which files may be modified
- **Constraints**: Hard rules (tests must pass, no new deps, etc.)
- **Termination condition**: When to stop. Ask the user — options are:
  - Fixed experiment count (e.g., 20 experiments)
  - Fixed time budget (e.g., 2 hours)
  - Target metric (e.g., val_bpb < 1.0)
  - Run until interrupted (default)

### Step 2: Create Branch and State Files

```bash
git checkout -b autoresearch/<goal>-<date>
```

Read the source files thoroughly. Understand the workload deeply before writing anything.

Create three files:

#### `autoresearch.md`

The living research document. A fresh agent with no context should be able to read this file and run the loop effectively. Invest time making it excellent.

```markdown
# Autoresearch: <goal>

## Objective
<Specific description of what we're optimizing and the workload.>

## Metrics
- **Primary**: <name> (<unit>, lower/higher is better) — the optimization target
- **Secondary**: <name>, <name>, ... — independent tradeoff monitors

## How to Run
`./autoresearch.sh` — outputs `METRIC name=number` lines.

## Files in Scope
<Every file the agent may modify, with a brief note on what it does.>

## Off Limits
<What must NOT be touched.>

## Constraints
<Hard rules: tests must pass, no new deps, etc.>

## Termination
<When to stop: experiment count, time budget, target metric, or run until interrupted.>

## What's Been Tried
<Update this section as experiments accumulate. Note key wins, dead ends,
and architectural insights so the agent doesn't repeat failed approaches.>
```

#### `autoresearch.sh`

Bash script (`set -euo pipefail`) that: pre-checks fast (syntax errors in <1s), runs the benchmark, and outputs structured `METRIC name=value` lines to stdout. Keep the script fast.

For fast, noisy benchmarks (< 5s), run the workload multiple times inside the script and report the median. Slow workloads (ML training, large builds) don't need this.

Example:
```bash
#!/bin/bash
set -euo pipefail

# Pre-check: syntax validation
python3 -c "import ast; ast.parse(open('train.py').read())" 2>&1 || { echo "SYNTAX ERROR"; exit 1; }

# Run the workload
output=$(uv run train.py 2>&1)

# Extract and output metrics
val_bpb=$(echo "$output" | grep -oP 'val_bpb=\K[0-9.]+' | tail -1)
echo "METRIC val_bpb=$val_bpb"
```

#### `autoresearch.checks.sh` (optional)

Only create this when the user's constraints require correctness validation (e.g., "tests must pass", "types must check"). Bash script (`set -euo pipefail`) for backpressure checks.

```bash
#!/bin/bash
set -euo pipefail
pnpm test --run --reporter=dot 2>&1 | tail -50
pnpm typecheck 2>&1 | grep -i error || true
```

### Step 3: Commit State Files

```bash
git add autoresearch.md autoresearch.sh
git commit -m "autoresearch: initialize experiment session"
```

### Step 4: Run Baseline

Run the experiment command and record the baseline result. This is experiment #1 — it establishes the starting point.

## The Experiment Loop

**LOOP FOREVER.** Never ask "should I continue?" — the user expects autonomous work. Only stop when:
- The termination condition from setup is met
- The user interrupts
- You detect you're running low on context (see Context Management below)

### For Each Experiment:

#### 1. Choose What to Try

Read `autoresearch.md` (especially "What's Been Tried") and `autoresearch.ideas.md` (if it exists) to pick the next hypothesis. Think about what the data tells you. The best ideas come from deep understanding, not random variations.

#### 2. Make Changes

Edit the files in scope. Keep changes focused — one hypothesis per experiment.

#### 3. Run the Experiment

Execute the benchmark:

```bash
timeout 600 bash autoresearch.sh
```

Capture the full output. Parse `METRIC name=value` lines from the output.

If the run crashes or times out, log it as a crash and revert.

If `autoresearch.checks.sh` exists and the benchmark passed, run it:
```bash
timeout 300 bash autoresearch.checks.sh
```
If checks fail, log as `checks_failed` and revert.

#### 4. Evaluate Results

Compare the primary metric against the baseline using the helper script:

```bash
python3 autoresearch_helper.py evaluate --jsonl autoresearch.jsonl --metric <value> --direction <lower|higher>
```

This outputs whether to keep or discard, the confidence score, and delta from baseline.

Decision rules:
- **Primary metric improved** -> `keep`
- **Primary metric worse or unchanged** -> `discard`
- **Simpler code for equal performance** -> `keep` (removing code for same perf is a win)
- **Ugly complexity for tiny gain** -> probably `discard`
- Secondary metrics rarely affect the keep/discard decision. Only discard a primary improvement if a secondary metric degraded catastrophically.

#### 5. Record Results

**On keep:**

```bash
# Commit the changes
git add -A
git commit -m "<description>

Result: {\"status\": \"keep\", \"<metric_name>\": <value>}"
```

Append to `autoresearch.jsonl`:
```bash
python3 autoresearch_helper.py log --jsonl autoresearch.jsonl \
  --commit $(git rev-parse --short=7 HEAD) \
  --metric <value> \
  --status keep \
  --description "<what was tried>" \
  --direction <lower|higher>
```

**On discard/crash/checks_failed:**

```bash
# Revert changes (preserve autoresearch state files)
git checkout -- .
git clean -fd 2>/dev/null
# Re-stage state files in case they were unstaged
git add autoresearch.jsonl autoresearch.md autoresearch.ideas.md autoresearch.sh autoresearch.checks.sh 2>/dev/null || true
```

Append to `autoresearch.jsonl`:
```bash
python3 autoresearch_helper.py log --jsonl autoresearch.jsonl \
  --commit "0000000" \
  --metric <value_or_0> \
  --status <discard|crash|checks_failed> \
  --description "<what was tried>" \
  --direction <lower|higher>
```

#### 6. Update Research Journal

After every few experiments (or after significant findings), update the "What's Been Tried" section in `autoresearch.md`. Include:
- What worked and why
- What didn't work and why
- Dead ends to avoid
- Current best result and how it was achieved

#### 7. Maintain Ideas Backlog

When you discover promising but deferred optimizations, append them as bullet points to `autoresearch.ideas.md`. Don't let good ideas get lost. Prune stale or tried entries.

#### 8. Loop

Go back to step 1.

## State Files Reference

| File | Format | Purpose |
|------|--------|---------|
| `autoresearch.jsonl` | JSON Lines | Append-only experiment log. One JSON object per line. |
| `autoresearch.md` | Markdown | Living research document. Objective, what's been tried, current best. |
| `autoresearch.ideas.md` | Markdown | Hypothesis backlog. Bullet points of promising ideas to try. |
| `autoresearch.sh` | Bash | Benchmark script. Outputs `METRIC name=value` lines. |
| `autoresearch.checks.sh` | Bash | Optional correctness checks (tests, types, lint). |

### JSONL Schema

Each line in `autoresearch.jsonl` is either a config header or an experiment result:

Config header (first line, or on re-init):
```json
{"type": "config", "name": "...", "metricName": "...", "metricUnit": "...", "bestDirection": "lower|higher"}
```

Experiment result:
```json
{
  "run": 1,
  "commit": "abc1234",
  "metric": 1.234,
  "metrics": {"compile_us": 4200, "render_us": 9800},
  "status": "keep|discard|crash|checks_failed",
  "description": "what was tried",
  "timestamp": 1711600000000,
  "segment": 0,
  "confidence": 2.1,
  "asi": {"hypothesis": "...", "rollback_reason": "...", "next_action_hint": "..."}
}
```

### ASI (Actionable Side Information)

Always record ASI with every experiment. At minimum: `{"hypothesis": "what you tried"}`. On discard/crash, also include `rollback_reason` and `next_action_hint`. Add any other key/value pairs that capture what you learned — dead ends, surprising findings, error details, bottlenecks.

ASI is the only structured memory that survives reverts. Without it, future iterations waste time re-discovering the same dead ends.

## Confidence Scoring

After 3+ experiments, the helper script computes a confidence score using Median Absolute Deviation (MAD):

| Confidence | Meaning |
|-----------|---------|
| >= 2.0x | Improvement is likely real |
| 1.0-2.0x | Above noise but marginal |
| < 1.0x | Within noise — consider re-running to confirm |

The score is advisory — it never auto-discards. If confidence is below 1.0x, consider re-running the same experiment to confirm before keeping.

## Context Management

Droid sessions have finite context. To handle this gracefully:

1. **Track experiment count** in the current session. After ~15 experiments, context is getting heavy.
2. **Save state proactively** — all state lives in files (jsonl, md), so a new session can resume immediately.
3. **When context is getting exhausted**: update `autoresearch.md` with current findings, commit state files, and stop. The next session reads the files and continues.
4. **On resume**: read `autoresearch.md`, `autoresearch.jsonl`, and `git log --oneline -20` to understand where things stand.

## Loop Rules Summary

- **LOOP FOREVER.** Never ask "should I continue?"
- **Primary metric is king.** Improved -> keep. Worse/equal -> discard.
- **Annotate every run with ASI.** Record what you learned, not just what you did.
- **Watch the confidence score.** < 1.0x means within noise — re-run to confirm.
- **Simpler is better.** Removing code for equal perf = keep.
- **Don't thrash.** Repeatedly reverting the same idea? Try something structurally different.
- **Crashes:** fix if trivial, otherwise log and move on.
- **Think longer when stuck.** Re-read source files, study the data, reason about what's actually happening. The best ideas come from deep understanding.
- **Resuming:** read autoresearch.md + git log, continue looping.

## Finalization

When the experiment loop ends (termination condition met, user interrupts, or context exhausted), finalize the results into clean, reviewable branches. This is the last phase of an autoresearch session.

### Step 1: Summarize Results

```bash
python3 autoresearch_helper.py summary --jsonl autoresearch.jsonl
```

Review the git log for actual commits:

```bash
git log --oneline --stat $(git merge-base HEAD main)..HEAD
```

### Step 2: Group Changes

Group kept experiments into **logical changesets**. Each group should:
- Represent a single coherent optimization or change
- Not share modified files with other groups (so branches can merge independently)
- Have a clear description of what it achieves and the metric improvement

Present the proposed grouping to the user for approval:

```
Group 1: "Reduce model depth from 8 to 6"
  Files: train.py (DEPTH, HEAD_DIM, N_EMBED)
  Metric improvement: val_bpb 1.15 -> 1.08 (-6.1%)
  Experiments: #3, #7, #12

Group 2: "Switch to cosine LR schedule"
  Files: train.py (lr_schedule, warmup_steps)
  Metric improvement: val_bpb 1.08 -> 1.05 (-2.8%)
  Experiments: #15, #18
```

Wait for user confirmation before proceeding.

### Step 3: Resolve File Conflicts

If groups share files, resolve before creating branches:
- Merge the groups into one (if changes are related)
- Split the file changes more carefully (if they're truly independent modifications to different parts)
- Ask the user which group gets priority

Groups **must not share files** — each branch must be independently mergeable. If all changes touch the same file and can't be separated, create a single finalized branch with all improvements combined.

### Step 4: Create Clean Branches

For each group:

```bash
merge_base=$(git merge-base HEAD main)
git checkout -b autoresearch/finalize/<group-name> $merge_base
git checkout autoresearch/<session-branch> -- <file1> <file2> ...
git commit -m "<group description>

Autoresearch results:
- Metric: <name> improved from <baseline> to <best> (<delta>%)
- Confidence: <score>x noise floor
- Experiments: <count> total, <kept> kept"
```

### Step 5: Verify and Report

For each finalized branch, run the benchmark to confirm the improvement holds, run any checks if applicable, and verify it merges cleanly with main.

Present a summary to the user:

```
Created 2 clean branches from 20 experiments:

  autoresearch/finalize/reduce-depth
    val_bpb: 1.15 -> 1.08 (-6.1%)
    Ready for review

  autoresearch/finalize/cosine-schedule
    val_bpb: 1.08 -> 1.05 (-2.8%)
    Ready for review

Original experiment branch preserved: autoresearch/<session-branch>
```

The original experiment branch is always preserved — finalization creates new branches.

## Mission Worker Mode

When running as a mission worker, the feature description specifies the optimization goal, termination condition, files in scope, and constraints. Read it carefully, follow the same loop procedure above, and respect the termination condition. When the condition is met, run finalization and report results in the handoff.
