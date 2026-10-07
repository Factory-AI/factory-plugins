# Storage Migration Reference

## Risk Model

Storage changes are high risk when they touch live tenants, PDisks, storage pools, Docker volumes, or bind-mounted data directories. Use read-only discovery first, then present impact and rollback before mutation.

Prefer this order:

1. Inspect current containers, mounts, image tags, tenant names, and ports.
2. Capture YDB config and BSC placement with `ReadStoragePool` and `QueryBaseConfig`.
3. Verify tenant metadata and small table reads before any move.
4. Dump data to a separate path.
5. Rebuild or restore on a copied volume when possible.
6. Switch clients only after metadata reads and placement checks pass.
7. Clean old storage only after final verification.

## Discovery Commands

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -l <user> <host> \
  "docker ps -a --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}'"

ssh -o BatchMode=yes -o ConnectTimeout=10 -l <user> <host> \
  "df -h / /path/to/storage 2>/dev/null || true"

ssh -o BatchMode=yes -o ConnectTimeout=10 -l <user> <host> \
  "docker volume ls --format '{{.Name}}' | grep -E 'ydb|local' || true"

ssh -o BatchMode=yes -o ConnectTimeout=10 -l <user> <host> \
  "docker inspect ydb-local ydb-dyn-example 2>/dev/null"
```

Tenant and BSC checks:

```bash
sudo cat /path/to/root.password | docker exec -i ydb-local bash -lc '
  umask 077
  cat >/tmp/root.password
  /ydb -e grpc://localhost:2136 -d /local/<tenant> \
    --user root \
    --password-file /tmp/root.password \
    scheme ls /local/<tenant>
  /ydbd --server localhost:2136 \
    --user root \
    --password-file /tmp/root.password \
    admin blobstorage config invoke --proto "Command { ReadStoragePool { BoxId: 1 } }"
  /ydbd --server localhost:2136 \
    --user root \
    --password-file /tmp/root.password \
    admin blobstorage config invoke \
      --proto "Command { QueryBaseConfig { RetrieveDevices: true SuppressNodes: true } }"
  rm -f /tmp/root.password
'
```

Treat monitoring/UI `StorageGroups` as advisory only. Use `ReadStoragePool` for pool config and `NumGroups`; use `QueryBaseConfig { RetrieveDevices: true SuppressNodes: true }` for actual `Group -> PDisk` placement.

## Single-Disk Rebuild Pattern

Use this path when the user wants all active YDB storage on one real disk or bind mount. The safe pattern is dump -> fresh single-disk local-ydb -> restore -> verify -> cleanup. Do not try to live-move non-empty tenant groups with `ReassignGroupDisk` for this goal unless the user accepts the risk after rehearsal.

Target topology should use placeholders in reusable docs:

- active static/root node and tenant data in one bind mount: `/path/to/ydb-data:/ydb_data`
- static PDisk path inside YDB: `/ydb_data/pdisks/1`
- tenant: `/local/<tenant>`
- dump path: `/path/to/ydb-dump/<tenant>-<timestamp>`

## Dump

Stop clients before taking consistency-sensitive dumps. Keep secrets in temp files and remove them on exit.

```bash
TS=$(date +%Y%m%d-%H%M%S)
BASE=/path/to/ydb-dump/<tenant>-$TS
sudo install -d -o <user> -g <user> "$BASE"

docker inspect ydb-local ydb-dyn-example >"$BASE/docker-inspect-before.json" 2>/dev/null || true

sudo cat /path/to/root.password | docker run --rm -i \
  --network container:ydb-local \
  -v /path/to/ydb-dump:/dump \
  --entrypoint /bin/bash \
  ghcr.io/ydb-platform/local-ydb:26.1.1.6 \
  -lc '
    set -euo pipefail
    umask 077
    cat >/tmp/root.password
    trap "rm -f /tmp/root.password" EXIT
    /ydb -e grpc://localhost:2137 -d /local/<tenant> --user root --password-file /tmp/root.password \
      tools dump -p . -o /dump/<tenant>-<timestamp>/tenant
  '
```

Adjust endpoint, database, and dump path to the live topology. Whole-tenant dump can be unreliable for some layouts; use table-level dumps when whole-tenant dump is unsupported, too broad, or fails on a rehearsed copy.

## Fresh Rebuild

Create a new static/root data directory or Docker volume, then start a fresh static node with one target data mount:

```bash
docker rm -f ydb-local ydb-dyn-example 2>/dev/null || true
sudo install -d -o root -g root -m 0755 /path/to/ydb-data

docker run -d --name ydb-local \
  --no-healthcheck \
  --network ydb-net \
  --restart unless-stopped \
  -p 127.0.0.1:8765:8765 \
  -v /path/to/ydb-data:/ydb_data \
  -e GRPC_PORT=2136 \
  -e MON_PORT=8765 \
  -e GRPC_TLS_PORT= \
  -e YDB_GRPC_ENABLE_TLS=0 \
  -e YDB_ANONYMOUS_CREDENTIALS=1 \
  -e YDB_LOCAL_SURVIVE_RESTART=1 \
  -e YDB_FEATURE_FLAGS=enable_graph_shard \
  ghcr.io/ydb-platform/local-ydb:26.1.1.6
```

Create the replacement tenant through CMS, start its dynamic node, and wait for `scheme ls /local/<tenant>` to succeed before restore.

## Restore

When using the `local-ydb` image as a helper container, override the image entrypoint to `/bin/bash`:

```bash
sudo cat /path/to/root.password | docker run --rm -i \
  --network container:ydb-local \
  -v /path/to/ydb-dump:/dump \
  --entrypoint /bin/bash \
  ghcr.io/ydb-platform/local-ydb:26.1.1.6 \
  -lc '
    set -euo pipefail
    umask 077
    cat >/tmp/root.password
    trap "rm -f /tmp/root.password" EXIT
    /ydb -e grpc://localhost:2137 -d /local/<tenant> --user root --password-file /tmp/root.password \
      tools restore -p . -i /dump/<tenant>-<timestamp>/tenant
  '
```

After restore, recreate users/grants and auth config if the fresh cluster started without mandatory auth.

## Verification Before Cleanup

Do not delete old storage until all checks pass:

```bash
/ydb -e grpc://localhost:2137 -d /local/<tenant> --user root --password-file /tmp/root.password \
  scheme ls /local/<tenant>

/ydb -e grpc://localhost:2137 -d /local/<tenant> --user root --password-file /tmp/root.password \
  scheme describe /local/<tenant>/<known-table>

/ydb -e grpc://localhost:2137 -d /local/<tenant> --user root --password-file /tmp/root.password \
  sql -s "SELECT COUNT(*) AS c FROM <known-table>;"

/ydbd --server localhost:2136 --user root --password-file /tmp/root.password \
  admin blobstorage config invoke \
    --proto 'Command { QueryBaseConfig { RetrieveDevices: true SuppressNodes: true } }'
```

Also verify:

- tenant is `RUNNING`
- all expected dynamic nodes are `Up`
- `GraphShardExists` is `true` when GraphShard is required
- anonymous monitoring access returns `401` in hardened deployments
- direct public YDB gRPC is not exposed unless that was the approved topology
- old Docker volumes and bind-mounted disk paths are still present until rollback is no longer needed

## Cleanup

Only after verification and a rollback hold period, remove old storage paths and Docker volumes named by the live discovery output. Do not put concrete destructive `rm -rf` paths into reusable docs; write them only in private, host-specific execution notes.

Before declaring old storage deleted, check both bind-mounted paths and Docker volumes:

```bash
docker volume ls --format '{{.Name}}' | grep -E 'ydb|local' || true

docker ps -a --filter volume=ydb-local-data --format 'table {{.Names}}\t{{.Status}}'

sudo find /path/to/storage /var/lib/docker/volumes -maxdepth 4 \
  \( -path '*ydb*pdisks*' -o -path '*ydb-dump*' -o -path '*ydb-data*' -o -path '*ydb-local-data*' \) \
  -print 2>/dev/null | sort
```

## Pitfalls

- `ReassignGroupDisk` success and `OperatingStatus: FULL` do not prove tenant metadata survived.
- Removing an apparently unused PDisk file can break root or scheme state if it still stores shared metadata.
- UI storage counts can disagree with BSC placement. Prefer `QueryBaseConfig` for physical placement.
- Adding a PDisk path is a BSC runtime operation, not only a config-file edit. Capture current `HostConfig`, use the server-accepted `DefineHostConfig` shape, and verify the new PDisk appears before allocating groups to it.
- Changes to `host_configs` or expected slot counts in generated config may not immediately change BSC runtime state for an existing static disk.
- `DecommitGroups` is not a tenant allocation shrink operation.
- `storage_units_to_remove` needs server-side implementation verification before live use.
- Whole-tenant dump/restore and table-level dump/restore have different path semantics; test on a copy first.
