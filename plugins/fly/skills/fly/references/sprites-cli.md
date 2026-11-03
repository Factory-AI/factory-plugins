# Sprite command and API reference

Two binaries, two scopes. `sprite-env` acts on the sprite it runs in.
`sprite` acts on a named remote sprite. The REST API sits behind both.

## `sprite-env` (inside a sprite)

Located at `/.sprite/bin/sprite-env`. Every call is a request to the local
socket; `sprite-env curl <path>` is the raw pass-through when no subcommand
exists (`sprite-env curl /v1/checkpoints`, `sprite-env curl /policy/network`).

```
sprite-env info                 # name, id, org, URL, version (JSON)
sprite-env version
sprite-env curl <path>
sprite-env services <sub>
sprite-env checkpoints <sub>
```

| Services | Notes |
|---|---|
| `services list` | All services with state |
| `services get <name>` | One service |
| `services create <name> --cmd <exe> [--args a,b,c] [--env K=v,K2=v2] [--dir <path>] [--needs svc1,svc2] [--http-port <port>] [--duration <time>] [--no-stream]` | `--cmd` is the executable only; one service may hold `--http-port` |
| `services delete <name>` | Removes the definition |
| `services start\|stop <name>` | `stop` sends TERM |
| `services restart <name>` | Atomic; prefer over stop+start |
| `services signal <name> <SIG>` | `TERM`, `HUP`, `KILL`, ... |

| Checkpoints | Notes |
|---|---|
| `checkpoints list [--history <ver>]` | IDs are `v0`, `v1`, ... |
| `checkpoints get <id>` | |
| `checkpoints create [--comment <msg>]` | Always pass a comment |
| `checkpoints restore <id>` | Async; restarts the environment |

Service logs: `/.sprite/logs/services/`. Mounted checkpoints:
`/.sprite/checkpoints/<id>/`. Docs: `/.sprite/llm.txt`,
`/.sprite/docs/agent-context.md`.

## `sprite` (outside a sprite)

Global flags: `-o, --org <name>`, `-s, --sprite <name>`, `-h`.

| Authentication | Notes |
|---|---|
| `sprite org auth [--org <name>]` | Browser flow via Fly.io |
| `sprite org list` | Configured orgs and tokens |
| `sprite org logout [--force]` | Remove tokens |
| `sprite org keyring disable\|enable` | Keyring vs file storage |
| `sprite auth setup --token <t>` | Non-interactive; `org-slug/org-id/token-id/token-value` |
| `sprite logout` | Remove all Sprites configuration |

| Lifecycle | Notes |
|---|---|
| `sprite create [name]` | `--skip-console`, `--label <l>` (repeatable) |
| `sprite list` | `--prefix <p>`, `-w/--watch` |
| `sprite use [name]` | Writes `.sprite` in the directory; `--unset` removes it |
| `sprite destroy [name]` | `--force` skips the prompt. Irreversible |
| `sprite upgrade` | `--check`, `--force`, `--version`, `--channel` |

| Commands and sessions | Notes |
|---|---|
| `sprite exec [flags] -- <cmd> [args]` | `--dir`, `--env "K=v,K2=v2"`, `--tty`, `--file <src:dest>`, `--no-port-forward`, `--http-post` |
| `sprite console` | Interactive shell |
| `sprite sessions list\|attach <id>\|kill <id>` | Non-TTY sessions suspend on detach |

| Checkpoints | Notes |
|---|---|
| `sprite checkpoint create [--comment <t>]` | |
| `sprite checkpoint list` | `--history <ver>`, `--include-auto` |
| `sprite checkpoint info <id>` | |
| `sprite checkpoint delete <id>` | Not available through MCP |
| `sprite restore <id>` | Alias of `checkpoint restore` |

| Files, ports, URL | Notes |
|---|---|
| `sprite file push <local> <remote>` | |
| `sprite file pull <remote> <local>` | |
| `sprite file edit <remote>` | |
| `sprite proxy <port>` or `<local:remote>` | Several ports allowed; `-W/--stdio <[host]:port>` |
| `sprite url` | URL and auth mode |
| `sprite url update --auth public\|sprite` | Who can reach the URL |
| `sprite info` | Newer builds: replaces `sprite url` for reading |
| `sprite config update --url-auth public\|sprite [--private-access admins\|org_users]` | Newer builds: replaces `sprite url update` |
| `sprite api <path> -- [curl options]` | Authenticated raw API call |

Follow the deprecation notice the binary prints when a form has moved.

Environment variables: `SPRITE_TOKEN` (org token; highest priority; stateless
mode), `SPRITE_ORG` (default org), `SPRITE_URL` then `SPRITES_API_URL` (API
base override; default `https://api.sprites.dev`). Config: `~/.sprites/
sprites.json` (token in the keyring by default), per-directory `.sprite`.

Exit codes: `0` success, `1` general error, `2` command not found, `126`
cannot execute, `127` not found inside the sprite, `128+n` signal `n`.

## HTTP API

### Local socket (inside)

```bash
curl -s --unix-socket /.sprite/api.sock -H "Content-Type: application/json" http://sprite/v1/services
```

Unauthenticated; any process in the sprite may call it, which is why network
policy is read-only here. The URL host is ignored.

| Method + path | Purpose |
|---|---|
| `GET /info` | `{version, sprite_name, sprite_url, sprite_id, org}` |
| `GET /v1/services` · `GET /v1/services/{name}` | List · one |
| `PUT /v1/services/{name}` | Create or update; streams NDJSON |
| `DELETE /v1/services/{name}` | Delete |
| `POST /v1/services/{name}/start\|stop\|restart` | Streams NDJSON |
| `POST /v1/services/signal` | `{"name": ..., "signal": "TERM"}` |
| `POST /v1/checkpoint` | Create; `{"comment": ...}`; streams NDJSON |
| `GET /v1/checkpoints[?history=<ver>]` · `GET /v1/checkpoints/{id}` | List · one |
| `DELETE /v1/checkpoints/{id}` | Delete |
| `POST /v1/checkpoints/{id}/restore` | `202`, then the environment restarts |
| `GET /policy/network` | Read egress policy. **Read-only here by design** |
| `GET /v1/orgs` · `ANY /v1/orgs/{org}/...` | Orgs reachable with injected tokens · authenticated proxy to the remote API |

No service-logs route on the socket; read `/.sprite/logs/services/`.

### Remote API (outside)

Base `https://api.sprites.dev`, header `Authorization: Bearer <org-token>`
(`org-slug/org-id/token-id/token-value`; never log or commit it). `sprite api
<path>` signs requests for you.

| Sprites | |
|---|---|
| `POST /v1/sprites` | `{"name", "config": {ram_mb, cpus, region, storage_gb}, "environment": {}, "labels": []}` |
| `GET /v1/sprites` | `prefix`, `max_results`, `continuation_token` |
| `GET /v1/sprites/{name}` | One |
| `PUT /v1/sprites/{name}` | `url_settings` (`auth`: `public`\|`sprite`, `private_access`: `admins`\|`org_users`), `labels` |
| `DELETE /v1/sprites/{name}` | Destroy. Irreversible |

Everything below is `/v1/sprites/{name}` plus the path.

| Exec | |
|---|---|
| `POST .../exec` | Query `cmd` (repeatable: executable then each arg), `dir`, `path`, `env` (repeatable `K=V`), `stdin=true` with the body as stdin |
| `GET .../exec` | Active sessions |
| `POST .../exec/{session_id}/kill` | `signal` (default `SIGTERM`), `timeout` (default `10s`) |
| `WSS .../exec` | Interactive/TTY |

HTTP exec responses are framed `application/octet-stream`: `0x01` stdout
chunk, `0x02` stderr chunk, `0x03` then one exit-status byte at the end.
Decode the frames. For shell syntax make the shell the executable:
`cmd=bash&cmd=-lc&cmd=<script>`.

| Services | |
|---|---|
| `GET .../services` · `GET .../services/{svc}` | List · one |
| `PUT .../services/{svc}` | `{cmd, args[], env{}, dir, needs[], http_port}`; `cmd` is the executable only |
| `DELETE .../services/{svc}` | Delete |
| `POST .../services/{svc}/start\|stop\|restart` | `duration` (default `5s`) or `timeout` (default `10s`) |
| `GET .../services/{svc}/logs` | `lines`, `duration` to follow |
| `POST .../services/signal` | `{"name", "signal"}` |

| Checkpoints | |
|---|---|
| `POST .../checkpoint` | `{"comment": ...}` |
| `GET .../checkpoints[?history=<ver>]` · `GET .../checkpoints/{id}` | List · one |
| `POST .../checkpoints/{id}/restore` | Destroys later state |

| Policy | |
|---|---|
| `GET .../policy/network` · `POST .../policy/network` | Read · **replace** all rules |
| `GET\|POST\|DELETE .../policy/privileges` | Capabilities and devices |
| `GET\|POST\|DELETE .../policy/resources` | Memory limits |

| Filesystem | |
|---|---|
| `GET .../fs/read` | `path`, `workingDir`; raw bytes |
| `PUT .../fs/write` | `path`, `workingDir`, `mode`, `mkdirParents` (default true), `asRoot`; body raw bytes |
| `GET .../fs/list` | Directory entries |
| `DELETE .../fs/delete` | `path`, `workingDir`, `recursive`, `asRoot` |
| `POST .../fs/rename\|copy\|chmod\|chown` | JSON `source`/`dest` or `path` |
| `WSS .../fs/watch` | Change events |

| Other | |
|---|---|
| `WSS .../proxy` | TCP tunnel; first message `{"host", "port"}`, then raw bytes |
| `WSS .../ports/watch` | Port open/close events |
| `GET /v1/gateway/list` | Third-party API connections (call from inside) |

Streaming responses are newline-delimited JSON: `{"type": "info"|"stdout"|
"stderr"|"error"|"complete", ...}`; service log events use `{"type":
"stdout"|"stderr"|"exit"|"started"|"stopping"|"stopped", ...}` with a Unix
millisecond `timestamp`. Read to the terminating event before declaring
success; a `200` says the operation started.

There is no published OpenAPI document; the CLI and the SDKs (Go, JavaScript,
Python, Elixir) are the authoritative wire format. The sprite's own URL,
`https://<sprite>-<org>.sprites.app/`, is a separate surface: `auth: sprite`
needs an org bearer token, `auth: public` needs nothing.

## Sprite MCP tools

Over the Fly MCP connection (`https://mcp.fly.io`) the same control plane is
exposed as tools; see [tools.md](tools.md) for the full list with
constraints: `list_sprites`, `create_sprite`, `destroy_sprite`,
`search_sprite_mentions`, `open_sprite_inspector`, `get_sprite_info`, `exec`,
`exec_list`, `exec_kill`, `file_read|write|list|delete|rename|copy`,
`service_list|get|create|start|stop|logs`,
`checkpoint_create|list|get|restore`, `policy_network_get|update`.
