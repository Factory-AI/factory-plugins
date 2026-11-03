# Operate: status, logs, access, and troubleshooting

Diagnose from evidence. Collect the status, the logs and the Machine events
before changing anything, and quote what you saw.

## Status and health

```bash
fly status -a <app> [--json]              # Machines by region: state, checks, last update
fly status -a <app> --watch               # refresh every 5s
fly checks list -a <app> [--check-name <n>] --json
fly machine status <id> -a <app>          # state, events (including OOM kills and exit codes), checks
fly machine list -a <app> --json
fly releases -a <app> --image             # recent deploys and their images
fly services list -a <app>                # live ports and handlers
fly ips list -a <app>
fly incidents                             # platform incidents touching your apps
fly platform status
```

MCP reads for the same picture: `get_app`, `list_machines`, `get_machine`,
`list_machine_events`, `wait_for_machine_state`, `list_app_ips`,
`list_app_certificates`. Use them when the connection is present and no shell
is needed; they do not wake Machines.

Interpretation:

- `fly status` shows no `started` Machine: users get 503. Autostop may have
  stopped them (fine) or they crashed (not fine). `fly logs` tells which.
- Health check `critical`/`failing`: the proxy stops routing to that Machine.
  Failures such as "connection refused" may **not** appear in `fly logs`, so
  always read `fly checks list` too.
- `fly machine status` events: `exit` with a non-zero code and `oom_killed`
  are the two common ones.

## Logs

```bash
fly logs -a <app>                          # stream (Ctrl-C to stop); run in the background when you need to keep working
fly logs -a <app> --no-tail                # the buffered recent lines, then exit
fly logs -a <app> -m <machine-id>          # one Machine
fly logs -a <app> -r <region>
fly logs -a <app> --json | jq -r '.message'
```

Each line carries the region, Machine ID and process group. Deploy-time
failures also print log excerpts in `fly deploy` output. For history beyond
the buffer, the user needs a log shipper (the `fly-log-shipper` app) or the
dashboard's log search.

## Getting inside

```bash
fly ssh console -a <app>                   # root shell on a Machine (prompts if several)
fly ssh console -a <app> -s                # pick the Machine
fly ssh console -a <app> --machine <id> -C "ps aux"    # run one command
fly ssh console -a <app> -g worker -r ams  # by process group and region
fly ssh console -a <app> -u <user>
fly ssh issue --agent                      # longer-lived SSH cert into your agent
fly console -a <app>                       # NEW Machine from the current image running console_command (e.g. rails console)
fly console -a <app> --machine <id>        # same, on an existing Machine
fly sftp shell -a <app>                    # interactive file transfer
fly sftp get /path/in/machine ./local -a <app>
fly sftp put ./local /path/in/machine -a <app>
fly sftp find /app -a <app>
fly machine exec <id> "<command>" -a <app> # no shell session needed
fly proxy 15432:5432 -a <db-app>           # tunnel a port to localhost
```

`fly ssh` and `fly proxy` need WireGuard through the local `fly agent`; if
they hang, run `fly doctor`, then `fly agent restart`, and on hostile networks
`fly wireguard websockets enable`. The Machine must be `started`; autostop
may have stopped it, in which case `fly machine start <id>` first.

Anything you run inside a Machine can read or change production data. Confirm
before writes, and keep output with credentials or personal data out of the
reply.

## Local tooling

```bash
fly doctor [-a <app>] [-v]                 # auth, WireGuard, DNS, Docker, agent
fly doctor diag                            # send diagnostics to Fly support
fly version; fly version upgrade
fly agent restart
```

## Troubleshooting playbooks

### 502 or 503 from `<app>.fly.dev`

1. `fly status -a <app>`: are any Machines `started` and passing checks?
2. None started and `fly logs --no-tail` shows a crash loop: read the stack
   trace. Common: missing secret or env, migration not run, bad command.
3. Started but checks failing: wrong `internal_port`, app bound to
   `127.0.0.1`, health check path redirecting or requiring auth, slow boot
   (raise `grace_period`).
4. Started and healthy but 502s in logs with `could not proxy`: the app
   closes connections or exceeds `hard_limit`; check concurrency settings.
5. Intermittent after deploy: `fly releases`, then compare with
   [deploy.md](deploy.md) rollback.

### "app will not be reachable by fly-proxy" on deploy

The app is not listening on `0.0.0.0:<internal_port>`. Rails, Django, Next.js,
Express and others default to localhost or another port. Set the host and
port explicitly (often `PORT` from `[env]`) and match `internal_port`.

### Machine keeps restarting

`fly machine status <id>` shows exit codes and `oom_killed`. OOM: `fly scale
memory 1024 -a <app>` (and update `[[vm]]`), or fix the leak. Non-zero exit
on boot: run the start command by hand in `fly ssh console` or `fly console`
to see the error. `[[restart]] policy` controls retries.

### Deploy hangs or times out

Build hang: `fly deploy --depot=false` or `--local-only`; `--recreate-builder`
for a wedged remote builder. Machines slow to start: `--wait-timeout 10m`,
check `fly machine status` for image pull errors (private registry auth,
wrong architecture: build for `linux/amd64`).

### Volume or disk errors

`fly volumes list` for attachment and size; `df -h` via `fly ssh console`;
`fly volumes extend`. A new Machine in a region without an unattached volume
needs one created first ([volumes.md](volumes.md)).

### Database unreachable

Confirm the app and database are in the same org and network. `fly mpg status`
or `fly status -a <pg-app>`. `DATABASE_URL` present in `fly secrets list`?
From inside a Machine: `fly ssh console -C "nc -zv <host> 5432"`.

### Stuck Machine

In order: `fly machine restart <id>`, `fly machine update <id> --image <same
image>` to force a replace, `fly machine destroy <id> --force` then
`fly scale count` or `fly deploy` to recreate. Each step is more disruptive;
confirm.

### Certificate not issuing

`fly certs check <host>` shows which DNS record is missing. Do not re-run it
in a loop (rate limits); see [networking.md](networking.md).

### I cannot reach a Machine with `fly ssh`

`fly doctor`; `fly agent restart`; the Machine must be started; on corporate
networks `fly wireguard websockets enable`; as a last resort
`fly machine exec` works without WireGuard.

## Report

State the app, what you observed (quote the status line or log line that
matters), the cause you concluded, what you changed, and the verification you
ran afterwards. Say plainly when something remains unverified.

Checked against flyctl v0.4.123 and docs.fly.io/getting-started/troubleshooting on 2026-10-28.
