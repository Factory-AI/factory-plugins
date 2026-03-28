---
name: autoresearch-finalize
version: 1.0.0
description: |
  Finalize autoresearch experiments into clean, reviewable branches. Use when:
  - An autoresearch session has accumulated kept experiments on a branch
  - The user wants to extract clean, independent branches from the experiment history
  - The user wants to prepare autoresearch results for code review and merging
  Groups kept experiments by logical change, creates independent branches from
  the merge-base, each reviewable and mergeable on its own.
---

# Autoresearch Finalize

Turn a noisy autoresearch branch into clean, independent branches — one per logical change, each starting from the merge-base.

## When to Use

After an autoresearch session has produced kept experiments, the branch history is a messy sequence of incremental improvements, reverts, and re-tries. This skill extracts the net-positive changes into clean branches that can be reviewed and merged independently.

## Procedure

### Step 1: Analyze the Experiment History

Read `autoresearch.jsonl` to understand what was kept:

```bash
python3 autoresearch_helper.py summary --jsonl autoresearch.jsonl
```

This shows all kept experiments with their descriptions, metrics, and which files were changed.

Also review the git log to see the actual commits:

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

Shall I proceed with this grouping?
```

Wait for user confirmation before proceeding.

### Step 3: Resolve File Conflicts

If groups share files, you must resolve this before creating branches. Options:
- Merge the groups into one (if changes are related)
- Split the file changes more carefully (if they're truly independent modifications to different parts of the file)
- Ask the user which group gets priority for the contested file

Groups **must not share files** — each branch must be independently mergeable.

### Step 4: Create Clean Branches

For each group:

```bash
# Find the merge base
merge_base=$(git merge-base HEAD main)

# Create a clean branch from the merge base
git checkout -b autoresearch/finalize/<group-name> $merge_base

# Apply only the final state of files in this group
# (cherry-pick or manually apply the net changes)
git checkout autoresearch/<session-branch> -- <file1> <file2> ...

# Commit with a descriptive message including metrics
git commit -m "<group description>

Autoresearch results:
- Metric: <name> improved from <baseline> to <best> (<delta>%)
- Confidence: <score>x noise floor
- Experiments: <count> total, <kept> kept

Changes:
- <bullet point summary of what changed and why>"
```

### Step 5: Verify Each Branch

For each finalized branch:
1. Run the benchmark to confirm the metric improvement holds
2. Run any checks (tests, types, lint) if applicable
3. Verify the branch merges cleanly with main

### Step 6: Report Results

Present a summary:

```
Created 2 clean branches from 20 experiments:

  autoresearch/finalize/reduce-depth
    val_bpb: 1.15 -> 1.08 (-6.1%)
    Files: train.py
    Ready for review

  autoresearch/finalize/cosine-schedule
    val_bpb: 1.08 -> 1.05 (-2.8%)
    Files: train.py (different sections)
    Ready for review

Original experiment branch preserved: autoresearch/<session-branch>
```

## Notes

- The original experiment branch is preserved — finalization creates new branches, it doesn't modify the experiment branch.
- Each finalized branch starts from the merge-base, so they can be reviewed and merged independently without ordering dependencies.
- If all changes touch the same file and can't be separated, create a single finalized branch with all improvements combined.
- Include metric improvements in commit messages so reviewers can see the impact.
