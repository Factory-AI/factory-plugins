# Fly.io for Factory Droid

Use Fly.io from Droid: deploy apps with flyctl, inspect and operate authorized
resources through Fly MCP, and build or test code in persistent Sprites.

## Install

You need Factory Droid with model access and a Fly.io account. The hosted MCP
connection uses browser OAuth; no local Fly CLI is needed for MCP operations.
Install Python 3 for the confirmation hook. Local deploys and operations not
exposed by MCP need flyctl; the Fly skill includes setup instructions.

```bash
droid plugin marketplace add superfly/fly-droid-plugin
droid plugin marketplace list
droid plugin install fly@fly-droid-plugin --scope project
droid mcp auth
droid mcp list
```

Complete Factory login and project trust in Droid first. Authorize the plugin's
`fly` MCP server for the intended organization. Factory model access and Fly
authorization are separate. An empty list of authorized resources is a valid
result. Use `--scope user` instead for all projects. Pinned marketplaces can have
different names; use the name printed by `marketplace list`.

If you previously installed `fly@skills`, disable it in `/plugins` before enabling
this package in the same project. Do not configure a second standalone `fly`
server when the plugin already supplies it. Your existing Fly resources are
unaffected by changing the local plugin installation.

## Example requests

- "What Fly.io resources can this connection access?"
- "Check the health and recent logs for my Fly app."
- "Use a Sprite for this project's tests and give me a patch with the fix."
- "Resume the same Sprite workspace and verify its private preview."
- "Plan how to attach Managed Postgres to this app. Don't provision it yet."

Droid can select the `fly` skill from these requests. Skill selection depends on
the model; for an explicit invocation, ask it to use the installed `fly` skill.
The `brand` skill supplies Fly.io naming and visual guidance when relevant.

## Remote work

Droid's built-in file and shell tools still act on your local machine. Remote
edits, tests, and service commands must run through Sprite tools. Keep the Sprite
name, ID, and repository path in the handoff so the next session reuses it.

The [Sprite guide](skills/fly/references/sprites.md) covers source transfer,
baseline tests, checkpoints, patch export, supervised previews, and resuming work.
Preview URLs can require authentication. Keep existing access restrictions;
do not make a preview public merely to pass a health check.

## Access and safety

The MCP endpoint is `https://mcp.fly.io`. It can operate only within the OAuth
grant, including organization, app, product, and Sprite restrictions. Begin with
`get_fly_context`. The plugin does not broaden access or supply credentials.

The hook recognizes Droid's `Execute` and `fly___*` tool names and asks
before recognized destructive, disruptive, billable, and checkpoint-sensitive
operations. It is a command-pattern guard, not a shell sandbox. When Python 3
is missing or cannot run, it reports that the hook was skipped; ordinary Droid
permissions and server authorization still apply. Install Python 3 to enable it.

Fly.io resources can incur charges. Creation, deployment, scaling, restoration,
deletion, and changing public access require the corresponding user intent.
Never copy credentials into source files, logs, or preview responses.

## Troubleshooting

- Plugin missing: check `/plugins`, the project's scope, and marketplace name.
- Tools missing: check `droid mcp list`; authorize the bundled server with
  `droid mcp auth`, then start a fresh session.
- Model requests fail: verify Factory model access separately from Fly OAuth.
- Grant denied: explain the missing access; do not switch credentials to bypass it.
- Runtime operation fails: inspect the resource before retrying. The dated
  known-issue notes in the Sprite guide record tested workarounds, not guarantees
  that every runtime has the same limitations.

See the [distribution repository](https://github.com/superfly/fly-droid-plugin)
for validation and the release process. MIT licensed. [Fly.io](https://fly.io)
