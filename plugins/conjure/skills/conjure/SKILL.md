---
name: conjure
description: Create, deploy, and manage static sites and dashboards on the conjure platform, and use the site-scoped platform APIs (db, files, identity, ws, ai, warehouse). Installs and authenticates the conjure CLI when it is missing. Use when asked to build, generate, host or publish a site, page or dashboard with conjure, deploying a folder of HTML/CSS/JS, updating a live site, rolling back a bad deploy, tailing request logs, minting API tokens, or calling a site's /_conjure/ APIs.
---

# Conjure

Conjure hosts static sites: deploy a folder, get a live URL at
`<scheme>://<slug>.<base-domain>/`. Every deploy creates an immutable
version; rollback instantly activates a previous one. Every deployed
site also gets platform APIs (db, files, identity, ws, ai, warehouse)
served from its own origin under `/_conjure/`.

Examples below target Factory's deployment (`https://conjure.factory.ai`,
site slug `my-app`). Against a local dev server pass
`--server http://conjure.localhost:4600` and read the site origin as
`http://my-app.conjure.localhost:4600`; everything else is identical.

## Before anything else

Work through these three in order. Most sessions clear all three in under a
minute, and skipping one produces a failure that looks like a Conjure bug: a
missing binary, a blank page behind a login wall, or a site nobody asked for.

### 1. Know what you are building

If the request already says what to build, go to step 2. If it says what to
build but leaves the data source open, ask only the questions still open --
never invent figures to fill a dashboard. Made-up numbers on something that
looks official are the worst thing this skill can produce. If the user invoked
this skill with no brief, ask before doing anything, because a site is cheap to
build and expensive to guess at. Use a single AskUser with three questions:

- What should the page show?
- Where does the content or data come from? (paste, a file, or an API you can
  already reach)
- What should it be called? This becomes the slug in the URL.

Do not scaffold a placeholder site to "get started". An unwanted deploy takes a
slug permanently: deleted slugs are never reusable.

### 2. Make sure the CLI is installed

Nothing provisions `conjure` automatically. If `command -v conjure` finds
nothing, install it. The downloads bucket serves the binary unauthenticated, so
no credential is involved:

```bash
set -euo pipefail
BASE=https://downloads.factory.ai/conjure
V=$(curl -fsSL "$BASE/LATEST")
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$(uname -m)" in
  x86_64|amd64) A=x64 ;;
  arm64|aarch64) A=arm64 ;;
  *) echo "unsupported arch $(uname -m)" >&2; exit 1 ;;
esac
TMP=$(mktemp -d)
curl -fsSL "$BASE/releases/$V/$OS/$A/conjure" -o "$TMP/conjure"
curl -fsSL "$BASE/releases/$V/$OS/$A/conjure.sha256" -o "$TMP/sum"
if command -v sha256sum >/dev/null 2>&1; then SUM=sha256sum; else SUM="shasum -a 256"; fi
echo "$(cat "$TMP/sum")  $TMP/conjure" | $SUM -c -
mkdir -p "$HOME/.local/bin"
install -m 755 "$TMP/conjure" "$HOME/.local/bin/conjure"
export PATH="$HOME/.local/bin:$PATH"
conjure --version
```

The published `.sha256` holds the bare hash, which is why it is pasted in
front of the path rather than piped straight to `-c`. The checksum tool is
chosen at runtime because macOS ships only `shasum` and minimal Linux images
ship only `sha256sum`. Export the PATH line in every later shell too: each
command runs in a fresh one, so an install that is not on PATH looks exactly
like no install at all, and `$HOME/.local/bin/conjure` is a safe fallback if
the export does not stick. Tell the user you installed it; do not narrate every
step.

### 3. Make sure they are signed in

Run `conjure whoami --json --server https://conjure.factory.ai`. Any non-zero
exit means this host cannot act yet; treat them all the same and sign in. Do
not branch on exit 3 alone.

Pass `--server` explicitly here. A host that has never signed in has no config
file, so a bare `conjure whoami` targets the local dev default and fails with
`connection_error: cannot reach server http://conjure.localhost:4600` — which
is also just "not signed in", but it is not the message below and you should
not go debugging it. Once login succeeds the server is written to the config
and later bare calls work.

Against the real server, an unauthenticated call does not reach Conjure at
all: Okta answers with its HTML sign-in page and the CLI exits **1** with
`decode server response: invalid character '<'`. That message means "not signed
in", not "Conjure is broken".

The browser flow is the only path that works for a human, and it is a single
click for anyone already signed in to Okta:

```bash
conjure login --server https://conjure.factory.ai
```

It waits up to 5 minutes, so run it in the background and read what it printed:

```bash
export PATH="$HOME/.local/bin:$PATH"   # step 2's PATH did not survive this shell
LOG=$(mktemp "${TMPDIR:-/tmp}/conjure-login.XXXXXX")
chmod 600 "$LOG"
nohup conjure login --server https://conjure.factory.ai </dev/null >"$LOG" 2>&1 &
LOGIN_PID=$!
echo "login log: $LOG (pid $LOGIN_PID)"
sleep 3 && cat "$LOG"
```

The log holds a live approval URL, so it goes to a `mktemp` path at 0600
rather than a predictable one: on a shared host anyone who can read it can
complete the sign-in as themselves.

Note the printed path and pid. Neither variable survives into your next
command, and the log name is random, so if you do not read them out of this
output you cannot reach either again.

Redirect stderr as shown: the approval URL and the confirmation code are
written to **stderr**, not stdout, so a plain `&` with no capture leaves you
with nothing to show the user. Closing stdin matters too, because the command
also offers a "paste the token below" prompt. With stdin closed that prompt
still prints but no longer blocks, so ignore that line in the log.

On a headless host the log will also say `Could not open a browser
automatically`. That is expected and not a failure: the URL printed above it
is live, and that is the one to hand to the user.

Two delivery paths race (a loopback POST and a server-side handoff the CLI
polls), so it completes even when the CLI is on a remote machine and the
browser is on the user's laptop.

Post the approval URL as a **markdown link** in an ordinary message before you
call AskUser — `[Approve Conjure sign-in](<url>)`, never a bare URL in a fenced
code block. A code block can only be copy-pasted, which turns a one-click step
into a chore and sends the user hunting through the transcript. AskUser prompts
render as plain text, so the link must be in the message, not in the prompt.

Then AskUser to hold while they approve, telling them the page must show the
same confirmation code before they click Authorize. Offer approve and resend;
the code expires, so a retry needs a fresh `conjure login`. Kill the previous
attempt first with `kill "$LOGIN_PID"` (the pid printed above), since it keeps
polling for the full 5 minutes. Do not reach for `pkill -f 'conjure login'`:
`-f` matches whole command lines, so it also matches the shell you are running
it from and will kill your own session.

**Confirm with `conjure whoami --json` before continuing.** A user reporting
they approved is a claim, not a result. If it still fails, say so plainly and
re-offer with a fresh link. If it fails again, **stop and tell the user Conjure
is unavailable on this host.** Do not build the site anyway, do not write files
to a directory and imply they were published, and do not invent an alternative
host. Publishing is the whole point of this skill, and a page that was never
posted is worse than a clear stop.

If nobody answers, or you are running unattended, stop the same way. Do not
fall through to the token fallback below: it also needs a human with a
browser, so an empty answer here means the same thing as a failed one.

Fall back to a token only if the browser flow times out or no human is
present. Do not reach for `conjure tokens create` here: that is itself an
authenticated CLI call, so it fails for exactly the host that needs it. Have
the user mint the first token from the Conjure admin UI in a browser already
signed in to Okta, then store it:

```bash
CONJURE_TOKEN=cj_... conjure login --server https://conjure.factory.ai
```

Pass it in the environment, not as `--token`: argv is visible to every other
user on the box via `ps`. Both forms still land in shell history, so when a
human is driving, prefer `read -rs CONJURE_TOKEN && export CONJURE_TOKEN`
and then run `conjure login` on its own.

A pasted token lands in the session transcript, so prefer the browser whenever
a human is there. Once a token exists, `conjure tokens create` works for
minting any further ones.

## Golden path

```bash
conjure deploy . --site my-app --json        # create/update and go live
conjure open my-app --json                   # -> {"slug":"my-app","url":"https://my-app...."}
conjure versions my-app --json               # version history + activation timeline
conjure rollback my-app --json               # reactivate the previously active version
conjure rollback my-app 1 --json             # or an explicit version
conjure logs my-app --json                   # newest request logs (owner only)
conjure delete my-app --json                 # remove the site (slug stays reserved)
```

The first deploy to a new slug creates the site automatically (you become
the owner, visibility `internal`). Only the owner can deploy again.

## Creating a site (design defaults)

Conjure is Factory's internal sites platform: a site you generate is
Factory-branded by default. Before writing any HTML/CSS, pull the
Factory Brand Guide and build on its tokens — do not invent a palette
or typography.

- Source of truth: `github.com/Factory-AI/factory-brand-guide`.
  `public/brand.json` is authoritative for colors, the Tungsten neutral
  ramp, theme tokens, and typography; `public/llms.txt` is the prose
  intro for a first read.

  The repo is private, so `raw.githubusercontent.com` returns 404 even for a
  Factory employee (it carries no GitHub credentials). Read it through `gh`,
  which uses the user's own auth:

  ```bash
  gh api repos/Factory-AI/factory-brand-guide/contents/public/brand.json \
    -H "Accept: application/vnd.github.raw"
  ```

  Check first with `gh auth status >/dev/null 2>&1 || echo "gh not signed in"`
  rather than guessing from a failed call. If `gh` is missing or signed out,
  say so and ask the user rather than inventing colors: the rules below are
  the point of this section.

- Core rules: orange `#FF5A00` is the ONLY accent, used sparingly
  (CTAs, links, active indicators — never headlines, large metrics, or
  the logo); neutrals come from the warm Tungsten ramp, never pure or
  cool greys; Geist (Regular 400 default) for headlines and body,
  Geist Mono for labels/metadata/captions; choose dark or light
  explicitly per surface and use that theme's fixed token values; hold
  WCAG AA contrast (4.5:1 body, 3:1 large text).
- Logos: use the official assets from the brand guide repo
  (`public/logos/`) — white lockup/rotor on dark, black on light,
  never orange, never recreated or AI-generated.
- Write the site against the brand tokens from the start. Do not scaffold
  boilerplate and restyle it afterwards; that is how off-brand defaults
  survive into a deployed page.
- Only skip the brand system when the requester explicitly asks for a
  different brand or design.

## Commands

| Command | Purpose |
| --- | --- |
| `conjure init [dir]` | Scaffold a starter static site (optional; you do not need it to deploy) |
| `conjure login [--server <url>] [--token cj_...]` | Log in; without `--token` it opens the browser and mints a token for you |
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

Sign-in itself is step 3 above; this section covers the mechanics behind it
and what to do when you need to read a gated page.

Some deployments sit behind an identity proxy (Okta, IAP). Bearer tokens
take precedence over proxy-injected identity headers, so once you hold a
`cj_...` token use it for everything and ignore the browser entirely.

Bootstrapping is the exception. Before you have a token, `curl` against the
admin UI or a site returns the provider's sign-in page (a 302 to the IdP),
not content. When a human is present, `conjure login` handles this: it
opens their already-signed-in browser, they click Authorize once, and the
minted token lands in the CLI config automatically. This works even when
the CLI runs on a remote machine (SSH, a droid computer): the human opens
the printed URL in any browser, checks that the confirmation code matches
the one in the terminal, and the server hands the token to the waiting
CLI. Otherwise mint the first token from a browser that is already signed
in, then switch to `--token` / `CONJURE_TOKEN`.

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

Base: `<scheme>://<slug>.<base-domain>/_conjure/` — `https` for
`conjure.factory.ai`, `http` for a local dev server. Access follows the site's
visibility (a private site answers 404 to non-owners).
Errors are always `{"error":{"code":"...","message":"..."}}`.

**The curl examples below are written bare for readability, and bare curl only
works against a dev-mode server.** Against `conjure.factory.ai` every one of
them needs `-H "Authorization: Bearer $CONJURE_TOKEN"`; without it the Okta
proxy returns its HTML sign-in page rather than JSON, and the parse failure
looks like a broken API. Export the token once
(`export CONJURE_TOKEN=cj_...`) and add the header to each call.

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
curl -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/me
```

### Collections DB (JSON documents, site-scoped)

```bash
curl -X POST -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/db/tasks \
  -H 'Content-Type: application/json' -d '{"title":"first","done":false}'
curl -H "Authorization: Bearer $CONJURE_TOKEN" 'https://my-app.conjure.factory.ai/_conjure/api/db/tasks?limit=10'
curl -X POST -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/db/tasks/query \
  -H 'Content-Type: application/json' \
  -d '{"filter":{"done":false},"sort":{"field":"created_at","direction":"desc"}}'
```

Per document: `GET/PATCH/DELETE /_conjure/api/db/tasks/{id}` (PATCH
shallow-merges top-level fields). Live mutations stream as SSE:

```bash
curl -N --max-time 5 -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/db/tasks/subscribe
```

### Files (named uploads, site-scoped)

```bash
echo hello > /tmp/hello.txt
curl -X POST -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/files -F file=@/tmp/hello.txt
curl -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/files
curl -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/files/hello.txt
curl -X DELETE -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/files/hello.txt
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
curl -X POST -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/ai/chat \
  -H 'Content-Type: application/json' \
  -d '{"messages":[{"role":"user","content":"Reply with exactly: pong"}]}'
```

`"stream": true` relays the upstream SSE stream (`data:` chunks ending in
`data: [DONE]`). Models are allow-listed server-side; omitting `model`
uses the default. Per-site rate limit -> `429 rate_limited`.

### Warehouse (read-only SQL against operator-configured sources)

```bash
curl -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/warehouse/sources
curl -X POST -H "Authorization: Bearer $CONJURE_TOKEN" https://my-app.conjure.factory.ai/_conjure/api/warehouse/main/query \
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
  4 not found. Branch on them instead of scraping messages. The exception is
  a request an SSO proxy intercepts before Conjure sees it: the proxy returns
  its HTML sign-in page, so the CLI reports a decode failure and exits 1
  rather than 3. Treat any non-zero exit from `whoami` as not-signed-in.
- Server/token resolution: `--server`/`--token` flags beat
  `CONJURE_SERVER`/`CONJURE_TOKEN` env vars, which beat the config file
  written by `conjure login` (~/.config/conjure/config.json, mode 0600).
- Against a dev-mode server no token is needed. Behind an SSO proxy use
  `conjure login` (step 3) when a human is present; a bearer token is the
  headless fallback. Never script a login around the IdP.
- A path prefix in `--server` (e.g. `http://host/conjure`) requires a
  reverse proxy that strips the prefix — conjured itself always mounts
  at `/`.
- `.conjureignore` in the deployed folder excludes files (gitignore-like
  patterns); .git, .factory, .DS_Store, ._* are always excluded.
- Slugs are DNS labels: lowercase a-z0-9 and hyphens, 1-63 chars;
  `conjure`, `admin`, `api`, `www`, `_conjure` are reserved.
- Deleted slugs are never reusable; pick a fresh slug instead.
