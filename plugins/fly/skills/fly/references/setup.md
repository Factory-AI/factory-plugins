# Setup: flyctl, sign-in, tokens, and agent tooling

## Install flyctl

`fly` and `flyctl` are the same binary.

```bash
curl -L https://fly.io/install.sh | sh                    # macOS, Linux (installs to ~/.fly)
brew install flyctl                                        # macOS with Homebrew
pwsh -Command "iwr https://fly.io/install.ps1 -useb | iex" # Windows
```

The curl installer prints the lines to add to the shell rc file so `~/.fly/bin`
is on `PATH`. In a non-interactive shell, call `~/.fly/bin/fly` directly rather
than editing rc files on the user's behalf.

Check and upgrade:

```bash
fly version                  # pin this in reports; the command surface changes between releases
fly version upgrade          # also `fly version update`
```

An "unknown command" error on a documented subcommand usually means an old
binary. Upgrade before concluding the feature is missing.

## Sign in and sign up

```bash
fly auth login               # opens the browser; existing account
fly auth signup              # opens the browser sign-up form
fly auth whoami              # who is signed in (add --json to parse)
fly auth logout
```

Both commands wait for the browser flow to finish. When no browser can open
(SSH session, container, CI runner), the CLI prints a URL and a one-time code
and blocks until the user approves. **Relay the URL to the user immediately.**
Run the command in the background and poll its output, or tell the user a link
is coming before you start. If the waiting command dies, finish the login with
the code the approval page shows:

```bash
fly auth login --code <one-time-code>
```

Do not pass `--email`/`--password`; interactive credentials belong to the
user. Never paste a token the user shares in chat into a command line, and
never log one.

### Organizations

```bash
fly orgs list                # slugs you belong to
fly orgs show <slug>
```

Use the real slug with `-o <slug>` on `fly launch`, `fly apps create`,
`fly mpg create` and similar. Do not write `personal` as a stand-in for an
unknown org; ask or list.

## Tokens for unattended use

flyctl reads `FLY_API_TOKEN`, or `-t <token>` on any command. Prefer narrow
tokens over a user token:

```bash
fly tokens create deploy -a <app> -x 24h -n "ci deploy"   # one app and its resources
fly tokens create org -o <org>                             # org-scoped deploy token
fly tokens create readonly -o <org>                        # read-only org token
fly tokens create machine-exec -a <app>                    # exec into one app's Machines
fly tokens create ssh -a <app>                             # SSH to one app
fly tokens list
fly tokens revoke <id>
```

The default expiry is 20 years; pass `-x` with something shorter. A token is
printed once. Hand it to the user or the CI secret store; do not echo it in
your reply. `fly tokens attenuate` and `fly tokens 3p` narrow an existing token
further; read their help before using them.

A brand-new user with no token and no human present cannot complete sign-up.
There is no headless account-creation path.

## Agent tooling

### Factory Droid plugin

Install the Fly.io plugin for Factory Droid:

```bash
droid plugin marketplace add superfly/fly-droid-plugin
droid plugin marketplace list
droid plugin install fly@fly-droid-plugin --scope project
```

Use the registered marketplace name printed by `marketplace list` rather than
assuming it from a display label. Pinned sources can change that name.
Use `--scope user` instead for all projects. Start Droid in the project and
complete Factory login and folder trust, then run `droid mcp auth` to authorize
Fly separately. `droid mcp list` should show the plugin's `fly` server connected.
Do not add a duplicate standalone MCP server when the plugin already supplies it.

The hook handles Droid's `Execute` shell tool and its `fly___*` MCP
tool names, plus the `mcp__fly__*` form. That form also covers a standalone
`fly` server registered by `fly setup agent` when the plugin is installed. For
remote development, follow the
[Sprite workspace workflow](sprites.md#remote-workspace-workflow); installing
the plugin does not relocate Droid's built-in file or shell tools.

### Fly MCP connection

The hosted server is `https://mcp.fly.io` over HTTP with browser OAuth. The
plugin configures it. Use `droid mcp auth` for authentication and
`droid mcp list` to check the connection.

The first call triggers consent in the browser. The user selects organizations,
apps, Managed Postgres and Sprites permissions, and the grant is per resource.
After that, `get_fly_context` reports exactly what was authorized. Tools and
products absent from the grant are not available; new tools on the server never
widen the grant.

When a call is rejected because authentication is required, send the user to
the client's MCP authentication flow and retry once. When a resource is missing
or a tool is denied, inspect `get_fly_context`: listing organizations does not
grant product access, and Fly app, Managed Postgres and Sprites access are
granted independently. An ungranted product is not a broken connection. Do not
install a second server, copy tokens, or switch to another connection to work
around a grant.

Sprite restrictions in the grant (`name_prefix`, `max_sprites_total`, label
filters) bound what `create_sprite` may do. Read them before creating, and
never destroy someone else's Sprite to make room under a cap.

### flyctl's own MCP commands

`fly mcp` (experimental) wraps stdio MCP servers and registers them with
clients (`fly mcp add --claude|--cursor|--vscode|--windsurf|--zed|--neovim`).
It is not how the hosted Fly MCP is connected; use the HTTP URL above.

## Health check for agent setup

Before non-trivial work, report in one line what is available:

```bash
command -v fly && fly version && fly auth whoami
test -S /.sprite/api.sock && echo inside-sprite
command -v sprite && sprite org list
```

Plus, when MCP is connected, the organizations and products
`get_fly_context` lists. Anything missing routes back to the sections above.

Checked against flyctl v0.4.123 on 2026-10-28; `fly setup agent` on flyctl
v0.5.2, and Droid v1.1.0 plugin installation, OAuth, MCP execution and a
two-session Sprite workflow, on 2026-11-02.
