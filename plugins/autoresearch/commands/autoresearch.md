---
description: Start or resume an autonomous experiment loop for optimization research
argument-hint: <optimization goal, e.g. "optimize val_bpb in train.py">
---

Start autoresearch mode. Use the `autoresearch` skill to set up and run an autonomous experiment loop.

Goal: $ARGUMENTS

If `autoresearch.md` already exists in this directory, resume the existing experiment loop — read the file and git log for context, then continue where the last session left off.

If no `autoresearch.md` exists, gather information about the optimization target and set up a new experiment session.

Be careful not to overfit to the benchmarks and do not cheat on the benchmarks.
