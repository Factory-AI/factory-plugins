# Volumes and snapshots

A volume is persistent NVMe storage in one region, attached to at most one
Machine. Fly volumes are not shared or replicated: redundancy means more
Machines with their own volumes and application-level replication or a
database that does it for you. Snapshots are the backup mechanism.

## Inspect

```bash
fly volumes list -a <app> --json                # id, name, region, size, attached Machine, state
fly volumes show <id> -a <app>
fly volumes snapshots list <volume-id> -a <app>
```

MCP: `list_volumes` (hides deleting volumes unless `include_destroyed`; with
`summary: true` the byte and block statistics are omitted, not zero),
`get_volume`, `list_volume_snapshots`.

## Create

```bash
fly volumes create <name> -a <app> -r <region> -s 10            # GB; default 1
fly volumes create <name> -a <app> -r <region> -s 10 -n 2       # two volumes (two Machines)
fly volumes create <name> -a <app> -r <region> --snapshot-id <id>   # restore from a snapshot
fly volumes create <name> -a <app> -r <region> --snapshot-retention 14
fly volumes create <name> -a <app> -r <region> --vm-size performance-2x  # place for that Machine class
```

Flags: `--require-unique-zone` (default true) spreads same-named volumes
across hardware zones; `--scheduled-snapshots` (default true) enables daily
snapshots; `--no-encryption` disables at-rest encryption (do not do this
without being asked). Volumes are billable from creation; confirm size, region
and count.

The `name` must match `[[mounts]] source` in `fly.toml`, and the mount's
`destination` is where it appears in the Machine. `fly launch --volume
name:/path` and `fly deploy` create volumes when `[[mounts]]` names one that
does not exist yet, sized by `initial_size`.

MCP: `create_volume` (billable; does not restore or fork, which need separate
source-resource authorization). After an uncertain response, list volumes
before retrying.

## Resize, fork, update

```bash
fly volumes extend <id> -a <app> -s 20            # grow only; cannot shrink
fly volumes fork <id> -a <app> [--name <n>] [--region <code>]   # new volume with a copy of the data
fly volumes update <id> -a <app> --scheduled-snapshots=true --snapshot-retention 10
```

Extending grows charges and may need a Machine restart for the filesystem to
see the space, depending on the image. Automatic growth is configured in
`fly.toml` under `[[mounts]]` (`auto_extend_size_threshold`, `_increment`,
`_limit`), not on the volume. MCP: `extend_volume` (its result, like
`create_volume`'s, reports zero statistics; read `get_volume` for usage),
`update_volume` (read current settings and preserve the rest).

## Snapshots

```bash
fly volumes snapshots create <volume-id> -a <app>
fly volumes snapshots list <volume-id> -a <app>
fly volumes create <name> -a <app> -r <region> --snapshot-id <snapshot-id>   # restore into a new volume
fly machine clone <id> -a <app> --from-snapshot last                         # new Machine on a restored volume
fly scale count 2 --from-snapshot last -a <app>
```

Snapshot creation is asynchronous; it may return before an ID exists. List
until the snapshot's status is `created`; while `running`, its `created_at`
reads `0001-01-01T00:00:00Z` and sizes are placeholders. Do not call it a
backup until it completes. Daily scheduled snapshots are kept for
`snapshot_retention` days (default 5, max 60). Restoring always creates a new
volume; there is no in-place restore.

MCP: `create_volume_snapshot` (attached, ready volume only), then
`list_volume_snapshots` for the ID and status.

## Destroy

```bash
fly volumes destroy <id> -a <app> [-y]
```

Irreversible. The volume must be detached: stop and destroy the Machine, or
`fly machine update` it without the mount, first. Confirm the exact volume ID,
check `fly volumes snapshots list` for a recent snapshot, and say what data is
lost. MCP: `delete_volume`, which also requires the volume to be detached; if
the detach step is unavailable over MCP, explain the prerequisite rather than
forcing it.

## Scaling with volumes

- One volume per Machine. `fly scale count` reuses unattached volumes in the
  region first, then creates empty ones (`--with-new-volumes` forces new).
- Scaling down detaches; it never deletes.
- `canary` and `bluegreen` deploy strategies cannot be used with mounts.
- Growing to a new region means a new empty volume there; copy data yourself.

## Practical notes

- Mount paths: `/data` by convention; never `/`.
- Databases on volumes: Fly's unmanaged Postgres uses volumes; Managed
  Postgres does not expose them ([postgres.md](postgres.md)).
- A Sprite checkpoint and a volume snapshot are unrelated; neither protects
  the other's data.

Checked against flyctl v0.4.123 and mcp.fly.io on 2026-10-28.
