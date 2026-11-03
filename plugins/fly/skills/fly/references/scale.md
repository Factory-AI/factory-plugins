# Scale: Machine count, size, and regions

```bash
fly scale show -a <app> --json          # count, size, CPUs, memory, regions per process group
fly platform vm-sizes                   # valid --vm-size names
fly platform regions                    # region codes
fly regions list -a <app>               # regions this app may run in
```

Read `fly scale show` and `fly status` before changing anything; the current
count per region is the baseline the user will ask you about afterwards.

## Horizontal: `fly scale count`

```bash
fly scale count 3 -a <app>                          # total for the default group, spread over current regions
fly scale count web=4 worker=2 -a <app>             # per process group
fly scale count 4 --region ord,ams --max-per-region 2 -a <app> # four total, two in each region
fly scale count 1 --process-group worker --region syd -a <app>
fly scale count 2 --region syd -a <app>             # add a region: scale up there from zero
fly scale count 11 --max-per-region 5 -a <app>      # rebalance
fly scale count 0 -a <app>                          # scale to zero; list every group
fly scale count 2 --vm-size performance-2x -a <app> # size the new Machines as they are created
fly scale count 2 --with-new-volumes -a <app>       # do not reuse unattached volumes
fly scale count 2 --from-snapshot last -a <app>     # new volumes restored from the latest snapshot
```

Behaviour that surprises people:

- With `--region`, the count is the total across the selected regions, not
  the count in each. Use `--max-per-region` to cap each region when balancing.
- Scaling up **clones an existing Machine**. From zero there is nothing to
  clone, so `fly scale count` cannot scale up; run `fly deploy` to seed
  Machines again.
- Scaling down stops and destroys Machines but leaves their volumes
  unattached for reuse. Volumes are never destroyed by scaling.
- Scaling up an app with `[[mounts]]` attaches unattached volumes in the
  target region first, then creates new empty ones. Data is never copied.
  Plan for one volume per Machine per region.
- `fly deploy` with `--ha` (default) adds a standby Machine for groups with
  services. Pass `--ha=false` if a single Machine is intended.
- Count changes are billable immediately. Confirm the final per-region total
  before running.

Manual alternatives:

```bash
fly machine clone <id> -a <app> --region ams        # one more Machine, optionally elsewhere
fly machine stop <id> -a <app> && fly machine destroy <id> -a <app>
```

## Vertical: `fly scale vm` and `fly scale memory`

```bash
fly scale vm shared-cpu-2x -a <app>                 # named size, all groups
fly scale vm performance-2x --vm-memory 8192 -a <app>
fly scale vm shared-cpu-1x --process-group worker -a <app>
fly scale memory 1024 -a <app>                      # MB; keep within the size's allowed range
fly scale memory 2048 --process-group web -a <app>
```

Vertical changes restart every Machine in the group (rolling). If `fly.toml`
has a `[[vm]]` section, the next `fly deploy` resets Machines to it, so update
the file too or the change is lost.

| Size | CPUs | Default memory | Kind |
|---|---|---|---|
| `shared-cpu-1x` | 1 | 256 MB | shared |
| `shared-cpu-2x` | 2 | 512 MB | shared |
| `shared-cpu-4x` | 4 | 1024 MB | shared |
| `shared-cpu-8x` | 8 | 2048 MB | shared |
| `performance-1x` | 1 | 2048 MB | performance |
| `performance-2x` | 2 | 4096 MB | performance |
| `performance-4x` | 4 | 8192 MB | performance |
| `performance-8x` | 8 | 16384 MB | performance |
| `performance-16x` | 16 | 32768 MB | performance |

`fly platform vm-sizes` is authoritative (it also lists 6x, 10x, 12x, 14x).
Memory can be raised above the default with `--vm-memory` within limits
flyctl enforces. An OOM-killed app wants more memory, not swap.

## Regions

```bash
fly platform regions                                # codes and names
fly scale count 2 --region <code> -a <app>          # add Machines there
fly scale count 0 --region <code> -a <app>          # remove a region
fly machine clone <id> --region <code> -a <app>     # one Machine, another region
```

Latency-sensitive reads belong near users; writes belong near the database.
`primary_region` in `fly.toml` decides where `fly deploy` creates Machines
and what `PRIMARY_REGION` reports, and `min_machines_running` applies there
only. Apps with volumes grow one region at a time, with a volume per
Machine.

## Autoscaling without code

Fly's built-in elasticity is autostop/autostart, configured in `fly.toml`
([configure.md](configure.md)):

- `auto_stop_machines = "stop" | "suspend"` stops Machines when load falls
  below `concurrency.soft_limit`; `suspend` resumes faster.
- `auto_start_machines = true` wakes one on demand.
- `min_machines_running = N` keeps N awake in the primary region.

Scale **count** to the peak you want available, and let autostop idle the
rest. Metrics-driven autoscaling (queue depth, CPU) is done with the
`fly-autoscaler` app or the Machines API, not flyctl; mention that boundary.

## Verify

```bash
fly scale show -a <app> --json
fly status -a <app>
fly machine list -a <app> --json | jq '.[] | {id, region, state, "size": .config.guest}'
```

Report the before and after count per region and size per group.

Checked against flyctl v0.4.123 and docs.fly.io/apps/scale-count on 2026-10-28;
multi-region count example rechecked against flyctl v0.5.0 source on 2026-10-29.
