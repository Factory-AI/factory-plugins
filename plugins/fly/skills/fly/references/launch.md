# Launch: create a new app

`fly launch` is the one-command path from a source directory or an image to a
running app. It detects the framework, writes `fly.toml` (and a Dockerfile when
one is needed), creates the app, optionally provisions a database and object
storage, and deploys.

## When to use which

| Situation | Command |
|---|---|
| New app from this directory, deploy now | `fly launch` |
| New app, inspect config before deploying | `fly launch --no-deploy`, review, then `fly deploy` |
| Prebuilt image, no source | `fly launch --image <ref> --name <app> --org <org> --region <code>` |
| App already exists on Fly, no local `fly.toml` | `fly config save -a <app>` then `fly deploy` |
| Only the app record, no config or deploy | `fly apps create <name> -o <org>` |
| Second app from the same directory | `fly launch --copy-config --name <new>` |
| Clone a GitHub repo and launch it | `fly launch --from <repo-url> --into <dir>` |

`fly launch` on a directory that already has `fly.toml` offers to reuse the
settings; `--copy-config` accepts that non-interactively.

## Running it for the user

1. Look at the directory first: Dockerfile, package manifest, framework. Say
   what launch will probably detect.
2. Announce the run. The command prompts for sign-in if needed and opens the
   browser.
3. Prefer interactive mode when a human is present so they see the generated
   settings. Use `--yes` (`-y`) only when the user asked for an unattended run
   or you are supplying every setting on the command line.
4. Read the generated `fly.toml` and report the app name, region, port,
   Machine size and any databases it created before the deploy proceeds.
5. Verify as in [deploy.md](deploy.md): the deploy output, then `fly status`.

Useful flags:

| Flag | Effect |
|---|---|
| `--name <app>` | App name. Omit to be prompted; `--generate-name` always generates one. |
| `-o, --org <slug>` | Organization. |
| `-r, --region <code>` | Primary region, from `fly platform regions`. |
| `--no-deploy` | Write config and create the app, stop before deploying. |
| `--now` | Deploy without the confirmation prompt. |
| `-y, --yes` | Accept every confirmation. |
| `-i, --image <ref>` | Deploy an existing image instead of building. |
| `--dockerfile <path>` | Use a specific Dockerfile. |
| `--internal-port <n>` | Set `internal_port` for the generated services. |
| `--ha` / `--ha=false` | Create a second Machine for redundancy (default true for apps with services). |
| `--vm-size`, `--vm-cpus`, `--vm-memory`, `--vm-cpu-kind` | Initial Machine shape. |
| `--volume <name:/path>` | Mount a volume. `--volume-initial-size <GB>` sizes it. |
| `--db` / `--no-db` | Provision or skip a Postgres database. |
| `--no-redis`, `--no-object-storage` | Skip those prompts. |
| `--no-public-ips` | Do not allocate public IPs (internal apps). |
| `--flycast` | Allocate a private Flycast IPv6 instead. |
| `--no-github-workflow` | Skip generating a GitHub Actions deploy workflow. |
| `-e KEY=VAL` | Set environment variables in `fly.toml`. |
| `--secret KEY=VAL` | Set secrets on the new app (values never echoed). |
| `--build-arg`, `--build-secret`, `--build-target` | Build-time inputs. |
| `--attach` | Attach the new app to the current one. |
| `--path <dir>` | Launch a subdirectory. |
| `--json` / `--yaml` | Generate the config in JSON or YAML instead of TOML. |

Flags after `--` go to the framework scanner.

## What launch generates

- `fly.toml` with `app`, `primary_region`, `[build]` (or a `[build] image`),
  `[http_service]` with `internal_port`, `force_https`,
  `auto_stop_machines`, `auto_start_machines`, `min_machines_running`, and a
  `[[vm]]` section. Read [configure.md](configure.md) for every key.
- A `Dockerfile` and `.dockerignore` when the scanner knows the framework and
  none exists. Review it; it is a starting point.
- Framework-specific tweaks (Rails, Phoenix, Django, Laravel, Node, Go, Rust,
  Python, static sites). Scanners can patch the app to bind `0.0.0.0` and read
  `PORT`; keep those changes.
- Optionally a Managed Postgres cluster attached as `DATABASE_URL`, a Tigris
  bucket, or Upstash Redis, each a separate billable resource.
- Optionally `.github/workflows/fly-deploy.yml` using a deploy token.

## Naming and placement

- App names are globally unique and become `<app>.fly.dev`. Use the repo or
  directory name; `-prod`/`-staging` suffixes are the common convention for
  environments. There is no environment object; each environment is an app.
- Choose the region nearest the users, or nearest the database. The
  primary region is where `fly deploy` creates Machines and what
  `PRIMARY_REGION` reports to the app.

## `fly apps create` without launch

```bash
fly apps create <name> -o <org>        # just the app record
fly apps create --generate-name -o <org> --json
fly apps list -o <org> --json
fly apps open -a <app>                 # open the deployed URL
fly apps destroy <name> -y             # irreversible; confirm first
```

`apps create` writes no `fly.toml`. Pair it with `fly config save -a <name>`
or write the file by hand, then `fly deploy`.

## Launch requirements through MCP

When the hosted MCP is connected, `get_launch_requirements` explains why an
organization cannot launch yet (billing, SSO, provider terms) and returns
browser handoffs. It does not accept terms or change payment settings; the
user does that in the browser. The MCP server does not create apps or deploy.

Checked against flyctl v0.4.123 on 2026-10-28.
