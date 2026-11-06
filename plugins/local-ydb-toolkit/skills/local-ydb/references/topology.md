# Topology Reference

## Baseline Docker Topology

Common single-host `local-ydb` topology:

- static/root node container: `ydb-local`
- YDB image: `ghcr.io/ydb-platform/local-ydb:26.1.1.6`
- persistent data volume or bind mount at `/ydb_data`
- local domain: `/local`
- tenant database: `/local/<tenant>`
- dynamic tenant nodes: `ydb-dyn-<tenant>`, optionally more nodes with unique ports
- monitoring backend: `127.0.0.1:8765`

`/local` is the root local domain. It can run a database, but it does not get a GraphShard tablet just because `YDB_FEATURE_FLAGS=enable_graph_shard` is enabled.

Practical tag note:

- `ghcr.io/ydb-platform/local-ydb:26.1.1.6` was pullable and worked for a stable `26.1` local stack.
- `ghcr.io/ydb-platform/local-ydb:26.1` was not a reliable alias in our tests. Use an exact patch tag.

## Static Node

Local development static-node shape for host-side clients:

```bash
docker run -d --name ydb-local \
  --no-healthcheck \
  --network ydb-net \
  --restart unless-stopped \
  -p 127.0.0.1:2136:2136 \
  -p 127.0.0.1:2137:2137 \
  -p 127.0.0.1:8765:8765 \
  -v ydb-local-data:/ydb_data \
  -e GRPC_PORT=2136 \
  -e MON_PORT=8765 \
  -e GRPC_TLS_PORT= \
  -e YDB_GRPC_ENABLE_TLS=0 \
  -e YDB_ANONYMOUS_CREDENTIALS=1 \
  -e YDB_LOCAL_SURVIVE_RESTART=1 \
  -e YDB_FEATURE_FLAGS=enable_graph_shard \
  ghcr.io/ydb-platform/local-ydb:26.1.1.6
```

Notes:

- `2136` is the root/static gRPC endpoint for host apps using `/local`.
- `2137` is useful when dynamic tenant nodes share `ydb-local`'s network namespace and host apps use `/local/<tenant>`.
- In a hardened deployment, keep gRPC ports Docker-internal unless direct host access is explicitly required.
- `8765` should be loopback-only if exposed externally through HTTPS reverse proxy.
- `--no-healthcheck` may be required in non-TLS topology because the image healthcheck can expect `/ydb_certs/ca.pem`.
- For bind mounts, use placeholders such as `/path/to/ydb-data:/ydb_data` in reusable docs.
- For side-by-side rebuilds or rehearsals, use a separate profile with distinct container names, Docker network, Docker volume, and monitoring/gRPC/IC ports rather than mutating a broken stack in place.

## GraphShard Tenant

Create `/local/<tenant>` through the public CMS gRPC API, not SQL. Minimal request shape:

```text
path: /local/<tenant>
resources.storageUnits: [{unitKind: "hdd", count: "1"}]
options: {planResolution: 50, coordinators: 1, mediators: 1}
```

The tenant can remain in `PENDING_RESOURCES` until at least one dynamic node is started for it.

Field-proven behavior on `ghcr.io/ydb-platform/local-ydb:26.1.1.6`:

- `ydbd admin database /local/<tenant> create hdd:1` can return `OK` and `admin database ... status` can report `State: PENDING_RESOURCES`.
- That is not a failure. Start the first dynamic node after `status` succeeds; the tenant can move to `RUNNING` only after the dynamic node registers.

When the documented CMS proto recipe is version-sensitive, pin the upstream `ydb-platform/ydb` commit used for matching proto shapes in project docs.

## Dynamic Nodes

Generated `local-ydb` config may advertise the static node as `localhost:19001`. Dynamic node containers therefore often need to share the `ydb-local` network namespace.

For `ghcr.io/ydb-platform/local-ydb:26.1.1.6`, the generated static-node `config.yaml` can also contain:

```yaml
grpc_config:
  ca: /ydb_certs/ca.pem
  cert: /ydb_certs/cert.pem
  key: /ydb_certs/key.pem
```

The static container has those files; a plain dynamic container usually does not. A field-proven non-TLS dynamic launch is therefore:

```bash
docker run -d --name ydb-dyn-example \
  --no-healthcheck \
  --network container:ydb-local \
  --restart unless-stopped \
  -v ydb-local-data:/ydb_data:ro \
  -e GRPC_PORT=2137 \
  -e MON_PORT=8766 \
  -e GRPC_TLS_PORT= \
  -e YDB_GRPC_ENABLE_TLS=0 \
  --entrypoint /bin/bash \
  ghcr.io/ydb-platform/local-ydb:26.1.1.6 \
  -lc '
    set -euo pipefail
    cfg=/tmp/local-ydb-dynamic-config.yaml
    sed \
      -e "/^  ca: \/ydb_certs\/ca\.pem$/d" \
      -e "/^  cert: \/ydb_certs\/cert\.pem$/d" \
      -e "/^  key: \/ydb_certs\/key\.pem$/d" \
      /ydb_data/cluster/kikimr_configs/config.yaml > "$cfg"
    exec /ydbd server \
      --yaml-config "$cfg" \
      --tcp \
      --node-broker grpc://127.0.0.1:2136 \
      --grpc-port 2137 \
      --mon-port 8766 \
      --ic-port 19002 \
      --tenant /local/<tenant> \
      --node-host 127.0.0.1 \
      --node-address 127.0.0.1 \
      --node-resolve-host 127.0.0.1 \
      --node-domain local
  '
```

Additional nodes need unique `--grpc-port`, `--mon-port`, and `--ic-port` values. Dynamic node IDs are assigned by NodeBroker. Scripts should discover IDs through `/viewer/json/nodelist`, not hardcode observed values.

## Adding Dynamic Nodes After Mandatory Auth

Before adding nodes on a live host, inspect the current shape and copy mount, tenant, port, image, and network patterns from the working dynamic node:

```bash
docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}'

docker inspect ydb-dyn-example \
  --format '{{json .Config.Cmd}} {{json .Mounts}} {{json .HostConfig.NetworkMode}} {{json .Config.Entrypoint}} {{json .Config.Image}}'
```

For a mandatory-auth local-ydb dynamic node, `--auth-token-file` expects an `NKikimrProto.TAuthConfig` text protobuf, not a raw token file. The two startup fields are:

```text
StaffApiUserToken: "<allowed-node-sid>"
NodeRegistrationToken: "<allowed-node-sid>"
```

Start one new node first, verify it reaches `nodelist`, then start the next. Preserve logs from failed attempts until the replacement is healthy.

If a dynamic-node container exists but was created with stale image tags or stale flags, remove it and recreate it instead of using `docker start`. Reusing an old container can preserve the broken launch command.

## Storage Pools

Storage-pool changes should be verified through BSC, not only UI or `admin database ... status`:

```bash
/ydbd --server localhost:2136 --no-password \
  admin blobstorage config invoke --proto 'Command { ReadStoragePool { BoxId: 1 } }'

/ydbd --server localhost:2136 --no-password \
  admin blobstorage config invoke \
    --proto 'Command { QueryBaseConfig { RetrieveDevices: true SuppressNodes: true } }'
```

Use `ReadStoragePool` for pool config and `NumGroups`; use `QueryBaseConfig` for actual `Group -> PDisk` placement. Confirm placement before deleting old disk files or volumes.

## Upstream YDB Lookup

Search upstream code when behavior is unclear. Keep search terms stable: symbol names, proto fields, endpoint names, CLI subcommands, config keys, error strings, or feature flags.

Examples:

```bash
gh api 'search/code?q="CreateDatabaseRequest"+repo:ydb-platform/ydb' \
  --jq '.items[] | [.path, .html_url] | @tsv'

gh api 'search/code?q=GraphShard+repo:ydb-platform/ydb' \
  --jq '.items[] | [.path, .html_url] | @tsv'

gh api 'search/code?q="register_dynamic_node_allowed_sids"+repo:ydb-platform/ydb' \
  --jq '.items[] | [.path, .html_url] | @tsv'

gh api 'search/code?q="GraphShard is not enabled on the database"+repo:ydb-platform/ydb' \
  --jq '.items[] | [.path, .html_url] | @tsv'

gh api repos/ydb-platform/ydb/contents/ydb/public/api/protos/ydb_cms.proto \
  --jq '.content'
```

Read matching files through the GitHub API and decode content locally when needed. Prefer the exact upstream commit referenced by project docs when reproducing old behavior.
