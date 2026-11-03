# Deploy: ship code and manage releases

`fly deploy` builds (or takes an image), creates a release, runs the release
command, and replaces the app's Machines according to the strategy. Run it
from the directory holding `fly.toml`, or pass `-a <app>` and `-c <path>`.

## The deploy

```bash
fly deploy                                  # build from Dockerfile, deploy
fly deploy -a <app> -c path/to/fly.toml     # explicit target
fly deploy --image registry.fly.io/<app>:<tag>   # prebuilt image, no build
fly deploy --image ghcr.io/org/app:1.2.3
fly deploy --remote-only                    # build on a Fly remote builder (default when no local Docker)
fly deploy --local-only                     # build with the local Docker daemon
fly deploy --depot=false                    # opt out of the Depot builder if a build hangs
fly deploy --build-only --push              # build and push the image, do not deploy
fly deploy --detach                         # return after the release is created; verify separately
fly deploy --wait-timeout 10m               # wait longer for Machines to start
fly deploy --strategy canary                # see strategies below
fly deploy --ha=false                       # do not add a second Machine on first deploy
fly deploy --regions ord,ams                # deploy to Machines in these regions only
fly deploy --process-groups web             # deploy one process group
fly deploy --only-machines <id>,<id>        # deploy specific Machines
fly deploy --update-only                    # do not create Machines for new process groups
fly deploy --skip-release-command
fly deploy --build-arg KEY=VAL --build-secret KEY=VAL --build-target <stage>
fly deploy -e KEY=VAL                       # env for this deploy (prefer fly.toml or secrets)
fly deploy --yes                            # accept confirmations
```

Image source precedence: `--image`, then `[build] image` in `fly.toml`, then
`[build]` builder settings, then `--dockerfile`, then `Dockerfile` in the
working directory.

Set `NO_COLOR=1` for plain output and `LOG_LEVEL=debug fly deploy` for a
verbose trace when a build or release step fails opaquely.

## First deploy behaviour

- Creates one Machine per process group, plus a second standby for groups
  with services when `--ha` is on (the default). Volumes change this: a
  mounted volume means one Machine per volume.
- Allocates a shared IPv4 and a dedicated IPv6 unless `--no-public-ips` or
  `--flycast` is set.
- Machines created with `fly machine run` are **unmanaged**: `fly deploy`
  leaves them alone. Use `fly machine update` for those, or recreate them
  through the app config.

## Strategies

Set in `[deploy] strategy = "..."` or `--strategy`.

| Strategy | Behaviour | Notes |
|---|---|---|
| `rolling` (default) | Replace Machines one batch at a time, waiting for each to pass checks. `max_unavailable` (default 0.33) sizes the batch. | Safe default. Forced when `--max-per-region 1`. |
| `immediate` | Replace every Machine at once without waiting for checks. | Downtime. Fast for dev apps. |
| `canary` | Boot one new Machine, verify it, then roll the rest. | Not usable with volumes. |
| `bluegreen` | Boot a full new set beside the old one, shift traffic when all pass checks, then destroy the old set. | Needs at least one health check. Not usable with volumes. Briefly doubles Machine count and cost. |

`--max-unavailable <n|fraction>` and `--max-concurrent <n>` tune rolling.

## Release command

`[deploy] release_command = "..."` runs once per deploy in a temporary
Machine built from the new image, before any app Machine changes. It has the
app's env, secrets and private network, but **no volumes**. A non-zero exit
aborts the deploy. Typical use: database migrations. `RELEASE_COMMAND=1` is
set inside it. `release_command_timeout` (default 5m) and
`[deploy.release_command_vm]` (`size`, `memory`) tune it.

When a migration needs a volume, run it from a Machine instead:
`fly ssh console -a <app> -C "<cmd>"` after the deploy, or `fly console`.

## Verifying a deploy

Never report success from exit status alone. Read the end of the deploy
output: it lists each Machine's update result and prints log excerpts when a
Machine fails smoke checks (restarting with a non-zero exit in the first ~10
seconds). Then:

```bash
fly status -a <app>                  # every Machine started, checks passing
fly checks list -a <app>             # health check detail
fly logs -a <app> --no-tail          # startup logs
fly releases -a <app> --image        # the new release and its image tag
curl -sS -o /dev/null -w '%{http_code}\n' https://<app>.fly.dev/
```

After `--detach`, poll `fly status` until every Machine is `started` and
healthy, or report the state you observed. A Machine stuck `stopped` with
a crash loop in `fly logs` is a failed deploy even if the command exited 0.

## Rollback

There is no `fly rollback`. Redeploy a previous image:

```bash
fly releases -a <app> --image        # find the image of the last good release
fly deploy -a <app> --image registry.fly.io/<app>:<tag>
```

`fly image update -a <app>` moves to the newest available image for the
current reference with a rolling restart; `--image <ref>` targets a specific
one. Both honour health checks unless `--skip-health-checks`.

## Builders

- **Depot** (default remote builder): fast, cached. `--depot=false` falls
  back to the classic Fly remote builder app in your org.
- **Remote builder**: a Machine named `fly-builder-*` in the org. `fly deploy
  --recreate-builder` rebuilds it if it is wedged.
- **Local**: `--local-only` uses the local Docker daemon. Build for
  `linux/amd64`.
- Buildpacks and Nixpacks: `[build] builder`/`buildpacks` or `--nixpacks`.

Build secrets (`--build-secret`) are mounted at build time only and never
land in the image. Build args are baked in; do not pass credentials as args.

## Secrets and deploys

`fly secrets set` triggers a deploy of the current image unless `--stage` is
passed; staged secrets land on the next `fly deploy`. Setting several secrets
before a first deploy should use `--stage` to avoid redundant restarts. See
[secrets.md](secrets.md).

## Common failure modes

| Symptom | Likely cause | Where to look |
|---|---|---|
| "app will not be reachable by fly-proxy" warning | App listens on `127.0.0.1` or the wrong port | Bind `0.0.0.0` on `internal_port`; [operate.md](operate.md) |
| Smoke checks failed, Machine restarting | Crash on boot: missing env, bad command, missing migration | Log excerpt in deploy output, `fly logs` |
| Health checks never pass | Check path wrong, redirect (301/302 fails), slow start | Add `grace_period`, dedicated `/healthz`; [configure.md](configure.md) |
| Release command failed | Migration error, DB unreachable | Deploy output prints its logs; run it manually via `fly console` |
| Build hangs | Builder issue | `--depot=false`, `--local-only`, `--recreate-builder` |
| Timed out waiting for Machines | Slow image pull or boot | `--wait-timeout 10m`, check `fly machine status <id>` |
| "could not find a volume" on scale or deploy | Mounts need one volume per Machine per region | [volumes.md](volumes.md) |

Checked against flyctl v0.4.123 and docs.fly.io/launch/deploy on 2026-10-28.
