# code-review

Pull request lifecycle skills: open, triage, and follow up on PRs with consistent conventions.

## Skills

### `create-pr`

Open a PR with Conventional Commits title, a templated body, local verification (lint/typecheck/tests), and an optional linked ticket. Use when the user asks to "create a PR," "open a PR," or "put code up for review."

### `follow-up-on-pr`

Take over an existing PR: rebase on the base branch, address reviewer comments, fix CI failures, and push updates to a merge-ready state. Accepts a PR URL or number as input.

## Install

```bash
droid plugin install code-review@factory-plugins
```
