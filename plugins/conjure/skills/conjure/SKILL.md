---
name: conjure
description: Deploy and manage static sites on the conjure platform, and use the site-scoped platform APIs (db, files, identity, ws, ai, warehouse). Use when deploying a folder of HTML/CSS/JS, updating a live site, rolling back a bad deploy, tailing request logs, minting API tokens, or calling a site's /_conjure/ APIs.
---

# Conjure

Conjure hosts static sites: deploy a folder, get a live URL at
`http://<slug>.<base-domain>/`. Every deploy creates an immutable
version; rollback instantly activates a previous one. Every deployed
site also gets platform APIs (db, files, identity, ws, ai, warehouse)
served from its own origin under `/_conjure/`.

Examples below use the local dev server (`http://conjure.localhost:4600`,
site slug `my-app`). Against another deployment, substitute its base
domain and your slug; everything else is identical.

## Golden path

```bash
conjure deploy . --site my-app --json        # create/update and go live
conjure open my-app --json                   # -> {"slug":"my-app","url":"http://my-app...."}
conjure versions my-app --json               # version history + activation timeline
conjure rollback my-app --json               # reactivate the previously active version
conjure rollback my-app 1 --json             # or an explicit version
conjure logs my-app --json                   # newest request logs (owner only)
conjure delete my-app --json                 # remove the site (slug stays reserved)
```

The first deploy to a new slug creates the site automatically (you become
the owner, visibility `internal`). Only the owner can deploy again.

## Commands

| Command | Purpose |
| --- | --- |
| `conjure init [dir]` | Scaffold a starter site + this skill |
| `conjure login --server <url> [--token cj_...]` | Verify and store credentials |
| `conjure whoami` | Show the identity the server resolves |
| `conjure deploy [dir] --site <slug>` | Pack (tar.gz, honors .conjureignore) and deploy |
| `conjure list` | List sites visible to you (--json items include the live `url`) |
| `conjure open <slug>` | Print the live URL |
| `conjure versions <slug>` | Immutable version history + timeline |
| `conjure rollback <slug> [version]` | Activate a previous version |
| `conjure delete <slug>` | Delete the site |
| `conjure logs <slug> [--limit n] [--follow]` | Request logs; `--follow --json` streams NDJSON |
| `conjure tokens create --name <name>` | Mint an API token (secret shown once) |
| `conjure tokens list` | Your tokens' metadata (`--include-revoked` adds history) |
| `conjure tokens revoke <name-or-id>` | Revoke immediately |

## API tokens

Tokens (`cj_...`) authenticate the CLI and scripts as your identity via
`Authorization: Bearer`. Required in proxy auth mode; optional in dev mode.

```bash
conjure tokens create --name ci-bot --json   # .token is shown ONCE — store it now
conjure tokens list --json
conjure tokens revoke ci-bot --json          # next use of that token fails (exit 3)
```

## SSO-gated deployments

Some deployments sit behind an identity proxy (Okta, IAP). Bearer tokens
bypass that proxy by design, so once you hold a `cj_...` token use it for
everything and ignore the browser entirely.

Bootstrapping is the exception. Before you have a token, `curl` against the
admin UI or a site returns the provider's sign-in page (a 302 to the IdP),
not content. Mint the first token from a browser that is already signed in,
then switch to `--token` / `CONJURE_TOKEN`.

To read an SSO-gated page during that window, reuse a session someone has
already authenticated rather than scripting a login:

```bash
agent-browser --profile Default open https://<base-domain>/
agent-browser eval "fetch('/SKILL.md',{credentials:'include'}).then(r=>r.text())" --json
```

`--profile <name>` copies that Chrome profile, cookies included, into a
temporary directory and drives a separate window. It inherits an existing
session; it does not create one. If nobody has signed in, it lands on the
sign-in page and a human has to finish it. Attaching to an already-running
browser over CDP is not a substitute: that requires the browser to have been
launched with `--remote-debugging-port`, which an ordinary one is not.

Non-rendered types like `.md` download instead of displaying, so `open`
reports `net::ERR_ABORTED` even when the fetch succeeded. Use the
same-origin `fetch` above to get the bytes.

## Platform APIs (on the site's own origin)

Base: `http://<slug>.<base-domain>/_conjure/`. Access follows the site's
visibility (a private site answers 404 to non-owners). Against a dev-mode
server curl works bare; otherwise add `-H "Authorization: Bearer cj_..."`.
Errors are always `{"error":{"code":"...","message":"..."}}`.

In site JavaScript, load the SDK instead of hand-rolling fetches:

```html
<script src="/_conjure/sdk.js"></script>
```

It exposes a `conjure` global: `conjure.identity.me()`,
`conjure.db.collection(name)` (`create/list/get/patch/delete/query/subscribe`),
`conjure.files` (`upload/list/url/delete`), `conjure.ws.connect(channel, opts)`,
`conjure.ai.chat({messages, stream})`, `conjure.warehouse.query(source, sql)`.

### Identity

```bash
curl http://my-app.conjure.localhost:4600/_conjure/api/me
```

### Collections DB (JSON documents, site-scoped)

```bash
curl -X POST http://my-app.conjure.localhost:4600/_conjure/api/db/tasks \
  -H 'Content-Type: application/json' -d '{"title":"first","done":false}'
curl 'http://my-app.conjure.localhost:4600/_conjure/api/db/tasks?limit=10'
curl -X POST http://my-app.conjure.localhost:4600/_conjure/api/db/tasks/query \
  -H 'Content-Type: application/json' \
  -d '{"filter":{"done":false},"sort":{"field":"created_at","direction":"desc"}}'
```

Per document: `GET/PATCH/DELETE /_conjure/api/db/tasks/{id}` (PATCH
shallow-merges top-level fields). Live mutations stream as SSE:

```bash
curl -N --max-time 5 http://my-app.conjure.localhost:4600/_conjure/api/db/tasks/subscribe
```

### Files (named uploads, site-scoped)

```bash
echo hello > /tmp/hello.txt
curl -X POST http://my-app.conjure.localhost:4600/_conjure/api/files -F file=@/tmp/hello.txt
curl http://my-app.conjure.localhost:4600/_conjure/api/files
curl http://my-app.conjure.localhost:4600/_conjure/files/hello.txt
curl -X DELETE http://my-app.conjure.localhost:4600/_conjure/api/files/hello.txt
```

Uploading an existing name replaces it. Deletes remove the name, not the
underlying content-addressed blob (v1 has no blob GC).

### WebSocket channels

`GET /_conjure/api/ws?channel=<name>` upgrades to a WebSocket; every frame
is a JSON envelope (`join`/`message`/`leave`/`error` with `member_count`
presence). Send `{"type":"message","channel":"<name>","payload":<any JSON>}`;
other members receive it, the sender does not. From site JS:

```js
const conn = conjure.ws.connect("lobby", { onMessage: (env) => console.log(env) });
conn.send({ hello: "world" });
```

### AI proxy (needs OPENAI_API_KEY on the server; always requires auth)

```bash
curl -X POST http://my-app.conjure.localhost:4600/_conjure/api/ai/chat \
  -H 'Content-Type: application/json' \
  -d '{"messages":[{"role":"user","content":"Reply with exactly: pong"}]}'
```

`"stream": true` relays the upstream SSE stream (`data:` chunks ending in
`data: [DONE]`). Models are allow-listed server-side; omitting `model`
uses the default. Per-site rate limit -> `429 rate_limited`.

### Warehouse (read-only SQL against operator-configured sources)

```bash
curl http://my-app.conjure.localhost:4600/_conjure/api/warehouse/sources
curl -X POST http://my-app.conjure.localhost:4600/_conjure/api/warehouse/main/query \
  -H 'Content-Type: application/json' \
  -d '{"sql":"SELECT id, name FROM wh_test WHERE id = $1","params":[1]}'
```

Responds `{"columns":[...],"rows":[[...]],"truncated":false}`. One
`SELECT`/`WITH` statement only, 1000-row cap, 10s timeout; writes are
rejected with `400 read_only_required`.

## Agent rules

- Always pass `--json`: stdout is a single JSON document, all prose
  goes to stderr. Parse with jq. Exception: `logs --follow --json`
  streams NDJSON (one compact JSON object per line).
- Exit codes are stable: 0 ok, 1 error, 2 usage, 3 auth/permission,
  4 not found. Branch on them instead of scraping messages.
- Server/token resolution: `--server`/`--token` flags beat
  `CONJURE_SERVER`/`CONJURE_TOKEN` env vars, which beat the config file
  written by `conjure login` (~/.config/conjure/config.json, mode 0600).
- Against a dev-mode server no token is needed; in proxy mode mint a token
  first (`conjure tokens create`) and pass it. Behind an SSO proxy a bearer
  token is the supported path — do not script a login around the IdP.
- A path prefix in `--server` (e.g. `http://host/conjure`) requires a
  reverse proxy that strips the prefix — conjured itself always mounts
  at `/`.
- `.conjureignore` in the deployed folder excludes files (gitignore-like
  patterns); .git, .factory, .DS_Store, ._* are always excluded.
- Slugs are DNS labels: lowercase a-z0-9 and hyphens, 1-63 chars;
  `conjure`, `admin`, `api`, `www`, `_conjure` are reserved.
- Deleted slugs are never reusable; pick a fresh slug instead.
