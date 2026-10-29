# show-me-your-work

Keep a reviewable decision trail for long-running or unattended work: a TSV log with one row per decision.

**Skills:**

- `show-me-your-work` - Append-only TSV decision log (what, why, evidence, result) with an end-of-run audit and optional cross-model review

Ships `references/decision-log-template.tsv` and a `scripts/log.sh` helper that appends well-formed, spreadsheet-safe rows.

Adapted from the pstack skills by Lauren Tan ([cursor/plugins](https://github.com/cursor/plugins), MIT License), with Cursor-specific tooling removed and Droid conventions applied.
