# Sprites

A Sprite is a persistent, hardware-isolated Linux environment from Fly. It
keeps its filesystem between sessions, gets a URL, runs supervised services
that survive disconnects, can be checkpointed and restored in seconds, and
filters its own outbound traffic. Sprites sit beside Machines and volumes as a
Fly product: Machines run your app; Sprites run your builds, tests, agents and
scratch environments.

## Step 1: detect the context

```bash
test -S /.sprite/api.sock && echo inside || echo outside
```

| Result | Meaning | Control plane |
|---|---|---|
| `inside` | The agent is running *in* the sprite | `sprite-env` at `/.sprite/bin/sprite-env`, over the local socket. Acts on **this** sprite only, takes no name. |
| `outside` | Laptop, CI runner, another host | `sprite` CLI (`-s <name>`) against `https://api.sprites.dev`, or the Sprite MCP tools, or the REST API. |

The two are not interchangeable. Do not call `sprite-env` from outside, and
do not use `sprite -s <self>` or the MCP tools from inside to manage the
sprite you are already in. A sprite that holds org credentials can
legitimately manage *other* sprites with `sprite -s <other>`; keep the
distinction explicit in what you report. Cache the answer for the session.

Secondary inside signals: `/.sprite/bin/sprite-env` exists and `sprite-env
info` returns JSON; `/.sprite/docs/agent-context.md` is present.

## Step 2: pick the smallest operation

| Goal | Inside | Outside (CLI) | Outside (MCP tool) |
|---|---|---|---|
| Identify the environment | `sprite-env info` | `sprite list`, `sprite info -s <n>` | `list_sprites`, `get_sprite_info` |
| Run a command | run it in the shell | `sprite exec -s <n> -- <cmd>` | `exec` (argv array) |
| Keep a process alive | `sprite-env services create ...` | `sprite exec -s <n> -- sprite-env services create ...` | `service_create`, `service_start` |
| Expose a preview URL | `--http-port` on the service | same, then `sprite url -s <n>` | `service_create` with `http_port` |
| Snapshot or roll back | `sprite-env checkpoints create` | `sprite checkpoint create -s <n>` | `checkpoint_create`, `checkpoint_restore` |
| Move files | file tools, `git clone` | `sprite file push|pull` | `file_write`, `file_read`, `file_list` |
| Outbound domain rules | read-only | `sprite api .../policy/network` | `policy_network_get`, `policy_network_update` |
| Third-party API | gateway at `api.sprites.dev/v1/gateway` | run the gateway call inside via `exec` | `exec` |
| Create or destroy | not possible | `sprite create` / `sprite destroy` | `create_sprite` / `destroy_sprite` |

Full command and endpoint tables are in [sprites-cli.md](sprites-cli.md).

## Setup from outside

```bash
curl -fsSL https://sprites.dev/install.sh | sh    # installs `sprite` to ~/.local/bin
sprite org auth                                    # browser flow via your Fly.io account
sprite auth setup --token "$SPRITE_TOKEN"          # CI: org-slug/org-id/token-id/token-value
```

`SPRITE_TOKEN` in the environment puts the CLI in stateless mode (no config
written), which is right for CI. `Error: no organizations configured` means
authentication, not a bug: tell the user to run `sprite org auth`. Never
paste a token on their behalf. Mint tokens at `sprites.dev/account`.

Target selection, in order: `-s <name>`, then a `.sprite` file written by
`sprite use <name>` in the working directory (git-ignore it), else the command
fails. Add `-o <org>` or `SPRITE_ORG` for multi-org accounts. Get names from
`sprite list`; never guess.

Over MCP, `get_fly_context` shows the Sprites binding: a `name_prefix`,
`max_sprites_total` and label filters bound what you may list and create.
Check them before `create_sprite`, pass the required prefix, and never destroy
someone else's sprite to free a slot. Sprite-scoped tools take `sprite` (name)
and accept `sprite_id` so a destroyed-and-recreated sprite is rejected; pass
the ID when you have it. Listing follows continuation tokens; a partial page
is not the inventory. The visible inventory is not necessarily the counter
used by the creation limit. Do not infer a creation denial from that count
alone; honor the server's response to an authorized creation request.

## Lifecycle

```bash
sprite list [--prefix dev]                  # names, state, URLs
sprite create my-task                       # create and open a console
sprite create --skip-console my-task [--label <l>]
sprite destroy -s my-task [--force]         # irreversible: filesystem, services, checkpoints, URL
```

Pick a short task-scoped name. If a restricted token demands a prefix, read it
from the error and retry once. Never clean up sprites that merely look stale;
destruction needs explicit intent. Reuse an obvious existing task environment
instead of creating another.

## Running commands

```bash
sprite exec -s dev -- ls -la
sprite exec -s dev --dir /app -- npm test
sprite exec -s dev --env "NODE_ENV=test,CI=1" -- npm test
sprite exec -s dev -- bash -lc 'grep -c TODO src/*.ts | sort'   # shell syntax needs an explicit shell
sprite console -s dev                                            # interactive shell
sprite sessions list | attach <id> | kill <id>
```

Everything after `--` is the command. `--tty` for interactive programs,
`--file <src:dest>` to upload first. Each exec is a session; non-TTY sessions
suspend on detach and resume on attach, so output is not lost.

MCP `exec` takes `cmd` as an argv array (`["python3", "-c", "print(1)"]`);
the string form splits on whitespace and breaks on quoting. It returns
output but **no exit status**; when the status matters, make the command
print it (`...; echo "exit=$?"`) and read it from the output. If `exec` or
a file tool fails with "is not assigned to compute" while `get_sprite_info`
shows the sprite running, the server cannot route to it yet; report the name
and ID, do not recreate.

Anything that must outlive the command is a service, not an exec.

## Services and the URL

A service is a supervised process: restarted after reboot, alive after your
session ends, routed HTTP by the proxy. **Never start a long-running process
with `&`, `nohup`, `screen` or `tmux`**; there is no systemd, and a detached
process is invisible to the supervisor.

```bash
sprite-env services list
sprite-env services create web --cmd /usr/bin/node --args server.js --http-port 3000
sprite-env services create web --cmd python3 --args "-m,http.server,8080" --http-port 8080
sprite-env services get|start|stop|restart|delete <svc>
sprite-env services signal <svc> HUP
```

`--cmd` is the executable **only**; arguments go in `--args` (comma
separated). Other flags: `--env "K=v,K2=v2"`, `--dir`, `--needs "svc1,svc2"`,
`--duration <time>` (how long to stream logs after creation), `--no-stream`.
Use `restart` rather than stop then start. From outside, wrap the same
command in `sprite exec -s <n> --`, or use the MCP tools `service_list`,
`service_get`, `service_create` (create or update), `service_start`,
`service_stop`, `service_logs` (bounded).

HTTP exposure depends on the runtime. Legacy Sprites route the Sprite URL to
port 8080 by default, or to the one service declaring `--http-port`. Newer
provider-backed Sprites support named service endpoints and return per-service
`public_url` and `exposure` fields. Prefer the URL returned by `service_get`
or `sprite-env services get`; `public_url` names an address, not anonymous
access. Check `exposure` before sharing it. A just-woken Sprite can take a few
seconds before its process is ready: probe with a bounded retry and check the
real HTTP status. Get the base URL with `sprite-env info` inside or
`sprite url -s <n>` (or `sprite info`) outside. Legacy URL auth modes are
`sprite` (org members, the default) and `public` (the internet); newer service
endpoints can report `authenticated`. Changing to public is a publication event:
`sprite url update --auth public -s <n>` (newer builds: `sprite config
update --url-auth public`); confirm first and say what becomes reachable.

Whatever the mode, a sprite service must never serve environment dumps,
credentials, key files, arbitrary file browsing, unfiltered logs, or debug and
admin endpoints. The MCP hook asks when `service_create` sets `http_port`.

Workflow: list first, pick a stable role name (`web`, `api`, `worker`,
`preview`), create with explicit `--cmd`/`--args`/`--http-port`, read the
streamed logs, verify with `services get` and an HTTP probe, report name,
state, port, URL and auth mode. When a start fails, read the logs before
editing the definition.

## Checkpoints

A checkpoint snapshots the writable filesystem overlay. Creating one is cheap
and fast; restoring rewinds and discards everything written after it.
Versions are `v0`, `v1`, ...; the last five are mounted read-only at
`/.sprite/checkpoints/` inside the sprite.

```bash
sprite-env checkpoints create --comment "auth flow passing tests"
sprite-env checkpoints list [--history <ver>]
sprite-env checkpoints get v3
sprite-env checkpoints restore v3                       # async; restarts the environment
sprite checkpoint create|list|info|delete -s <n>        # from outside
sprite restore v3 -s <n>
diff /.sprite/checkpoints/v3/home/sprite/app/config.toml /home/sprite/app/config.toml   # read an old file without restoring
```

Create one before package upgrades, migrations, bulk rewrites, running
generated code, or a service or policy change, and **whenever the user says
something works**. Always pass `--comment` with the reason. Do not ask
permission to create one.

Restore is destructive: list, name the exact target with its comment and
time, say that every later change is discarded (uncommitted work, installed
packages), offer to checkpoint the current state first, get approval, then
verify files and `services list` afterwards. Only the overlay is captured:
not the base image, not external databases, not state outside the sprite.
Individual checkpoints cannot be deleted through MCP. Checkpoints are not a
substitute for committing and pushing code.

MCP: `checkpoint_create`, `checkpoint_list`, `checkpoint_get`,
`checkpoint_restore` (confirmation required).

## Files

Ranked by reliability:

1. `git clone` inside the sprite. Fastest, and the sprite gets a real
   repository.
2. `sprite file push <local> <remote>` / `sprite file pull <remote> <local>`
   / `sprite file edit <remote>` from outside; MCP `file_write` (up to 16 MiB,
   `encoding` `utf8` or `base64`, `mkdir` for parents), `file_read`,
   `file_list`, `file_delete` (explicit `recursive`, `asRoot`), `file_rename`,
   `file_copy`.
3. The `fs` HTTP endpoints when driving the API directly.
4. A base64 payload decoded inside the sprite for small generated files; for
   several files, one encoded archive, not a dozen writes.
5. Ask the user for an authorized path for private code. Never invent
   credentials.

Never sync `.git/` (clone instead), dependency and build caches
(`node_modules/`, `.venv/`, `target/`, `_build/`, `deps/`, `dist/`, `.next/`),
secrets (`.env`, key files, token caches, cloud credential directories, shell
history), large datasets unless they are the payload, or editor and OS state.
Supply credentials as service or exec environment values at run time rather
than writing them to disk; a checkpoint keeps every secret written to the
filesystem. Verify what landed (`wc -c`, checksum, `du`) and report what you
copied and skipped.

## Network policy

Outbound traffic is filtered by a DNS-based allow/deny list, which is what
makes a sprite safe for untrusted or generated code. Read first, always:

```bash
curl -s --unix-socket /.sprite/api.sock http://sprite/policy/network   # inside: read-only by design
sprite api "/v1/sprites/<n>/policy/network"                            # outside
```

Rules look like `{"rules": [{"include": "defaults"}, {"domain":
"registry.npmjs.org", "action": "allow"}]}`. **Update replaces the whole set**
(`POST` to the same path, or MCP `policy_network_update`): read, build the
complete intended set, show it, get approval, write, read back. Code inside
a sprite cannot widen its own egress. Widening egress is a security decision:
add the narrowest domain that unblocks the work, never a wildcard or a raw IP.

Egress denial shows up as DNS failure or a hung connection. Reproduce with one
`curl -sS -o /dev/null -w '%{http_code}\n' https://<host>/` inside, check the
exact host against the rules (a rule for `example.com` does not cover
`cdn.example.com`), and resolve the host the tool actually contacts. Sibling
policies at `.../policy/privileges` and `.../policy/resources` follow the
same read-modify-write rule.

## API gateway

The gateway proxies outbound API calls and injects the user's connected
credentials, so a sprite can call GitHub, Slack, Linear and others without
ever holding a key. Calls must originate inside the sprite (from outside, run
them via `sprite exec`).

```bash
curl -s https://api.sprites.dev/v1/gateway/list            # connections (live) and available (not yet connected)
curl -s https://api.sprites.dev/v1/gateway/github/user/repos   # append the provider path to gateway_base_url
```

Send **no** `Authorization` header or key of your own; adding one is the
common failure and yields a confusing 401. Match on `provider`, use its
`gateway_base_url`, adapt `usage_snippet`. Missing scope: name the scopes and
send the user to `request_scopes_url`; do not retry in a loop. No connection:
share the provider's `setup_url` from `available`, then re-list. Unsupported
provider: offer `custom_api` rather than a raw key on disk. Keep gateway
responses out of anything the sprite serves over HTTP.

## Remote workspace workflow

When the agent runs outside a Sprite, its built-in shell, editor and Git tools
still act locally. Keep one explicit remote repository path and use Sprite
exec and file operations for every remote edit, build and test. Do not fix a
local file and report remote test results as validation of that change.

1. Discover the authorized organization and Sprite restrictions. Reuse an
   explicitly selected workspace or create a task-specific Sprite. Record its
   name, ID and repository path for follow-up sessions.
2. Clone the repository in the Sprite, or transfer only the task's source
   files for a local-only fixture. Establish the baseline commit and run the
   failing test remotely before editing. Follow the file exclusions above.
3. Checkpoint the baseline, make the remote change, and rerun the same test.
   Record the command, working directory and result. If an exec response lacks
   an exit status, capture the test process status explicitly in its output;
   successful transport alone does not mean the test passed.
4. Inspect the remote Git diff and save a patch or requested artifacts locally.
   A local patch is a deliverable, not evidence that the local checkout was
   changed. Checkpoint the working state and retain its ID.
5. For a preview, use a supervised service with an explicit working directory
   and HTTP port. Preserve the existing URL authentication mode, probe the
   service, and report how the user can access it. Do not change a private URL
   to public merely to make a probe pass.
6. In a follow-up session, select the same Sprite identity, inspect the existing
   repository and service, and rerun a test before continuing. Do not create a
   replacement Sprite or overwrite the workspace simply because the agent's
   local working directory is empty.

At handoff, report the Sprite name/ID, repository path and commit, test result,
patch location, checkpoint IDs, and preview URL/auth mode. Leave retained
resources explicit; cleanup needs the user's intent.

## Known runtime issues

Seen on Sprite runtime 0.1.12 with sprite-env 0.1.0 through mcp.fly.io on
2026-11-02. These are defects, not design. Re-check before relying on a
workaround, and drop an entry once the runtime no longer shows the problem.

- `exec` output can carry raw stream-framing bytes. Do not save it directly
  as a source file or patch; extract the payload and verify it.
- `file_read` can answer that the runtime supports writes but not reads. Use
  `exec` in the same sprite instead. To export a file, base64-encode it, take
  only the encoded payload, decode locally, and check the byte count and
  SHA-256 against the remote file. Fail on a mismatch; never strip bytes from
  a patch or treat an error response as file content.
- `file_write` can create root-owned files while `exec` runs as `sprite`.
  After uploading a workspace, check `id` and ownership before Git or build
  commands. If authorized, fix ownership on the new task directory only.
- `service_create` can return `missing_service_definition` on a runtime whose
  service schema the MCP adapter does not support yet. Run `sprite-env
  services help` through `exec` and create the service with the guest CLI,
  passing `--dir` for the repository, then read it back. This uses the same
  authorized sprite; it does not bypass a denial, change URL exposure, or
  justify detached shell processes.

## Golden path

1. Detect inside or outside. Choose the control plane once.
2. Resolve the exact sprite by name (and ID over MCP). Never guess.
3. Inspect before you mutate: services, logs, checkpoints.
4. Checkpoint valuable state before risky work.
5. Do the work: `exec` for bounded commands, a service for anything that
   must outlive the call.
6. Verify with real output: exit status, service state, logs, an HTTP probe.
7. Report the sprite name, the URL and its auth mode, and any checkpoint IDs.
   A remote result the user cannot locate afterwards is not a finished task.

Checked against sprites.dev CLIs and mcp.fly.io on 2026-10-28; remote workflow
verified with Droid 1.1.0, Sprite runtime 0.1.12 and sprite-env 0.1.0 on
2026-11-02; "Known runtime issues" lists what that test hit.
