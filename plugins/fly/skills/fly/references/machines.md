# Machines

A Machine is one Firecracker VM running one image, in one region, in one
process group. `fly deploy` manages app Machines as a group; `fly machine`
commands and the Machine MCP tools act on individual Machines. Both paths need
the exact Machine ID, from `fly machine list` or `list_machines`.

## Inspect

```bash
fly machine list -a <app> --json                    # IDs, state, region, image, size
fly machine list -a <app> -q                        # IDs only
fly machine status <id> -a <app>                    # state, events, checks
fly machine status <id> -a <app> --display-config   # full config as JSON (may include env values)
fly logs -a <app> -m <id>                           # one Machine's logs
```

MCP: `list_machines`, `get_machine` (lifecycle summary, no config),
`list_machine_events`, `wait_for_machine_state`. Configuration, history,
processes and metadata (`get_machine_configuration`, `list_machine_versions`,
`list_machine_processes`, `get_machine_metadata*`) need the
`fly:machines:sensitive:read` grant because env, files and arguments can hold
secrets; request them only when the task needs them and summarize
selectively. `list_machine_processes` includes kernel threads with empty
commands; report the named processes.

Machine states: `created`, `starting`, `started`, `stopping`, `stopped`,
`suspending`, `suspended`, `replacing`, `destroying`, `destroyed`. A service
with autostop may have stopped a Machine minutes after it started; check the
state before assuming a fault.

## Lifecycle

```bash
fly machine start <id> [<id>...] -a <app>
fly machine stop <id> -a <app> [--signal SIGTERM --timeout 30]
fly machine restart <id> -a <app> [--skip-health-checks] [--force]
fly machine suspend <id> -a <app>                   # snapshot memory; resumes faster than stop
fly machine kill <id> -a <app>                      # SIGKILL, no cleanup
fly machine cordon <id> -a <app>                    # pull from proxy routing, keep running
fly machine uncordon <id> -a <app>
fly machine wait <id> -a <app> --state started      # block until a state
fly machine destroy <id> -a <app>                   # must be stopped or suspended unless --force
fly machine destroy --image <hash> -a <app>         # every Machine running that image
```

MCP: `start_machine`, `stop_machine`, `restart_machine`, `suspend_machine`,
`signal_machine`, `cordon_machine`, `uncordon_machine`, `delete_machine`
(never enables `force` implicitly). Stopping a Machine whose config has
`auto_destroy` set destroys it. Explain disruption before any of these on a
production app; the plugin hook asks.

`delete_app` / `fly apps destroy` is broader than one Machine: it removes
every Machine, volume, IP and certificate the app owns. Inspect first.

## Create and change

```bash
fly machine run <image> [command] -a <app> -r <region> \
    --vm-size shared-cpu-1x -e KEY=VAL -p 443:8080/tcp:http:tls \
    --volume <vol>:/data --autostop stop --autostart --name <name>
fly machine run <image> --rm -a <app> -- <one-off command>   # ephemeral task Machine
fly machine update <id> -a <app> --image <ref>               # new image, same config
fly machine update <id> -a <app> --vm-memory 1024 --autostop suspend
fly machine update <id> -a <app> -C "<command>" --entrypoint <path>
fly machine update <id> -a <app> --file-local /etc/app.yml=./app.yml
fly machine update <id> -a <app> --metadata key=value
fly machine update <id> -a <app> --schedule hourly            # cron-like Machines
fly machine clone <id> -a <app> [--region <code>] [--name <n>] [--attach-volume <vol>:/data] [--from-snapshot last]
fly machine clone <id> -a <app> --standby-for <id>            # standby that starts if the source fails
fly machine place -a <app> --region <code> --count 3          # dry-run placement
```

Machines made with `fly machine run` are **unmanaged by `fly deploy`**. They
suit cron jobs, one-off tasks, standbys and experiments. For app Machines,
change `fly.toml` and deploy instead, so the config and the Machines agree.

MCP: `create_machine` (billable; starts unless `skip_launch`; if the response
is lost, look for the Machine before retrying), `update_machine_metadata`
(merges keys, needs `machine_version`; stale version is a conflict, so reread
and reassess rather than loop), `set_machine_metadata_key`,
`delete_machine_metadata_key`, `get_machine_memory`,
`set_machine_memory_limit`, `reclaim_machine_memory`. General Machine
configuration updates are not exposed over MCP; use `fly machine update`.

## Execute a command

```bash
fly machine exec <id> "<command>" -a <app> [--timeout 60] [--container <name>]
fly ssh console -a <app> --machine <id> -C "<command>"      # when exec needs a shell or TTY
```

MCP `exec_machine_command` takes `app_name`, `machine_id` and `body.command`
as an argv array, for example `{"command": ["uname", "-a"]}`. There is no
implicit shell; pass `["sh", "-c", "..."]` only when shell syntax is needed,
and never serialize an argv into one string. The Machine must be `started`;
exec against a stopped Machine currently reports a `conflict` saying the
resource "changed or is leased", so check `get_machine` before suspecting a
lease. The result carries `exit_code`, `stdout` and `stderr`; report the exit
code. A command can read or change production data however diagnostic it
looks; confirm first and keep sensitive output out of the reply.

## Leases

A lease is an exclusive write lock on a Machine, used by deploy tooling.

```bash
fly machine leases view <id> -a <app>
fly machine leases clear <id> -a <app>              # only when you know the holder is gone
```

MCP: `get_machine_lease` (answers `Resource not found` when no lease is held;
confirm the Machine exists with `get_machine`), `create_machine_lease`
(returns a nonce; keep it private, renew with the same nonce),
`release_machine_lease` (exact nonce). While a lease is held, Machine changes
without its `lease_nonce` are refused; the server currently reports that as
`outcome_unknown` even though nothing was applied. Inspect, then repeat with
the nonce if you hold it. Never release another operator's lease.

## Volumes and Machines

A volume attaches to one Machine. `fly machine clone` creates a new empty
volume for a Machine that has one (or restores from a snapshot with
`--from-snapshot`). `fly machine destroy` leaves the volume behind. See
[volumes.md](volumes.md).

## Verify and report

After any change: `fly machine status <id>` or `get_machine`, then the
relevant runtime check (`fly logs -m <id>`, an HTTP probe, `fly checks list`).
Report the Machine ID, region, state, and what you observed. After a timeout,
inspect before retrying: the original call may have succeeded.

Checked against flyctl v0.4.123 and mcp.fly.io on 2026-10-28.
