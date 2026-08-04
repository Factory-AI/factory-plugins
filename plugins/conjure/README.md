# conjure

Deploy a folder of static files and get a live site with platform APIs attached.

## Skills

### `conjure`

Drive the `conjure` CLI and the site-scoped platform APIs. Covers the deploy
loop (`deploy`, `versions`, `rollback`, `logs`), API tokens, and the six APIs
each site gets on its own origin under `/_conjure/`: `db` (JSON collections
over Postgres), `files` (content-addressed blobs), `identity` (caller
identity), `ws` (realtime channels), `ai` (streaming LLM proxy), and
`warehouse` (read-only SQL). Includes the browser SDK surface, stable exit
codes for branching, and the server/token resolution order.

## Install

```bash
droid plugin install conjure@factory-plugins
```

## Requires

The `conjure` CLI on your `PATH`, and a deployment to point it at.

## Source

`SKILL.md` is a copy of `internal/cli/scaffold_skill.md` in the conjure repo,
the same file `conjure init` scaffolds into `.factory/skills/conjure/`. Update
it there first, then copy the change here.
