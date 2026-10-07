# Verification Reference

## Tenant and Node State

Use the MCP tool first when available:

```json
{ "tool": "local_ydb_status_report", "arguments": { "profile": "<profile>" } }
{ "tool": "local_ydb_healthcheck", "arguments": { "profile": "<profile>" } }
```

`local_ydb_healthcheck` wraps YDB's built-in `monitoring healthcheck --format json` self-check. Treat `selfCheckResult=GOOD` as the main database-level green signal; for other results, route by issue type before trying ad hoc shell diagnostics.

Unauthenticated local-dev examples:

```bash
curl -sSL 'http://127.0.0.1:8765/viewer/json/tenants?database=/local'

curl -sSL 'http://127.0.0.1:8765/viewer/json/nodelist?database=%2Flocal%2Fexample&enums=true&type=any' \
  | python3 -m json.tool
```

In a hardened deployment, anonymous `viewer/json` should return `401`. Do not present these commands as post-auth verification unless an authenticated session or supported credential mechanism is included.

Authenticated viewer flow that has worked on local-ydb builds:

```bash
PASS=$(sudo cat /path/to/root.password)
DATA=$(printf '{"user":"root","password":"%s"}' "$PASS")

curl -sS -c /tmp/ydb-cookies.txt \
  -H 'Content-Type: application/json' \
  -X POST \
  --data "$DATA" \
  http://127.0.0.1:8765/login

curl -fsSL -b /tmp/ydb-cookies.txt -L \
  'http://127.0.0.1:8765/viewer/json/capabilities?database=%2Flocal%2Fexample'
```

Observed details:

- the working login field may be `user`
- protected viewer endpoints may return `307` after login; use `curl -L`
- cookie-based UI flow can work when generic Bearer-token testing does not
- use the actual monitoring port from the selected profile; do not hardcode `8765` when the stack runs on a different port

Authenticated node-list helper for hardened hosts:

```bash
python3 - <<'PY'
import http.cookiejar
import json
import subprocess
import urllib.parse
import urllib.request

password_file = "/path/to/root.password"
database = "/local/example"

password = subprocess.check_output(["sudo", "cat", password_file], text=True).rstrip("\n")
cookies = http.cookiejar.CookieJar()
opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookies))
opener.open(
    urllib.request.Request(
        "http://127.0.0.1:8765/login",
        data=json.dumps({"user": "root", "password": password}).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    ),
    timeout=10,
)

url = (
    "http://127.0.0.1:8765/viewer/json/nodelist?database="
    + urllib.parse.quote(database, safe="")
    + "&enums=true&type=any"
)
response = opener.open(url, timeout=10)
nodes = json.loads(response.read().decode())
print(json.dumps({
    "count": len(nodes),
    "nodes": [
        {"id": node.get("Id"), "address": node.get("Address"), "port": node.get("Port")}
        for node in nodes
    ],
}, separators=(",", ":")))
PY
```

Container/log checks for dynamic-node additions:

```bash
docker ps -a --filter name=ydb-dyn \
  --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'

docker logs --tail 80 <dynamic-node-container>
```

Treat a node as healthy only after all of these are true:

- the container is `Up`, not `Restarting`
- logs show `Successfully applied dynamic config from YAML`
- logs show `serve as dynamic node`
- authenticated `nodelist` includes the node ID and IC port
- tenant metadata checks still pass

`whoami` reachability checks:

```bash
curl -sS -o /tmp/local-whoami.out -w '%{http_code}\n' \
  'http://127.0.0.1:8765/viewer/json/whoami'

curl -k -sS -o /tmp/public-whoami.out -w '%{http_code}\n' \
  'https://<public-domain>/viewer/json/whoami'
```

Interpretation:

- `401` means the viewer endpoint is reachable and unauthenticated, which is the expected hardened result without a UI session
- `503` from the public domain during maintenance can be just a reverse proxy hitting an upstream while YDB is restarting
- `503` with a stable local `401` points to reverse-proxy/upstream routing rather than YDB auth itself

`admin database ... status` interpretation:

- `NOT_FOUND` or `Unknown tenant` means the tenant still needs to be created
- `PENDING_RESOURCES` can be the expected pre-dynamic state on a fresh `/local/<tenant>` database
- `RUNNING` is the steady-state target after the dynamic node has registered successfully

## Metadata Path

After any storage move, tenant recovery, or replacement-tenant cutover, verify metadata explicitly:

```bash
/ydb -e grpc://localhost:2137 -d /local/<tenant> \
  --user root \
  --password-file /tmp/root.password \
  scheme ls /local/<tenant>

/ydb -e grpc://localhost:2137 -d /local/<tenant> \
  --user root \
  --password-file /tmp/root.password \
  scheme describe /local/<tenant>/<known-table>

/ydb -e grpc://localhost:2137 -d /local/<tenant> \
  --user root \
  --password-file /tmp/root.password \
  sql -s "SELECT COUNT(*) AS rows FROM <known-table>;"
```

Treat these as stronger than `admin database ... status` or BSC `FULL`.

## GraphShard

Check capability and tablet presence:

```bash
curl -sSL 'http://127.0.0.1:8765/viewer/json/capabilities?database=%2Flocal%2Fexample' \
  | jq '.Settings.Database.GraphShardExists'

curl -sSL 'http://127.0.0.1:8765/viewer/json/tabletinfo?database=%2Flocal%2Fexample&enums=true' \
  | grep -o GraphShard | sort | uniq -c
```

Check metrics backend:

```bash
GRAPH_TABLET_ID=$(curl -fsSL 'http://127.0.0.1:8765/viewer/json/tabletinfo?database=%2Flocal%2Fexample&enums=true' \
  | jq -r '.. | objects | select(.Type? == "GraphShard") | .TabletId' \
  | head -n 1)

curl -fsS "http://127.0.0.1:8765/tablets/app?TabletID=${GRAPH_TABLET_ID}&action=get_settings"
```

Only the `backend` field is stable enough for docs. `metrics_size` changes as metrics are collected.

To switch the backend to `Local`, use only after confirming the tablet ID and intended tenant:

```bash
curl -fsS "http://127.0.0.1:8765/tablets/app?TabletID=${GRAPH_TABLET_ID}&action=change_backend&backend=1"
docker restart <dynamic-node-container>
```

## Graph Data

Use the dynamic-node path for graph data:

```bash
GRAPH_NODE_ID=$(curl -fsSL 'http://127.0.0.1:8765/viewer/json/nodelist?database=%2Flocal%2Fexample&enums=true&type=any' \
  | jq -r '.[0].Id')
NOW=$(date +%s)
FROM=$((NOW - 600))
curl -fsS "http://127.0.0.1:8765/node/${GRAPH_NODE_ID}/viewer/json/graph?database=%2Flocal%2Fexample&target=resources.memory.used_bytes&from=${FROM}&until=${NOW}&maxDataPoints=1000"
```

The root `/viewer/json/graph` endpoint can return `GraphShard is not enabled on the database` even when `GraphShardExists=true` for `/local/<tenant>`.

Field-proven post-auth checks on a stable `26.1.1.6` stack:

- anonymous `GET /viewer/json/whoami` returns `401`
- authenticated `scheme ls /local/<tenant>` succeeds with `root.password`
- authenticated `nodelist` returns the dynamic node and IC port
- authenticated `capabilities` reports `GraphShardExists=true`

## Storage

Check storage allocation:

```bash
curl -sSL 'http://127.0.0.1:8765/viewer/json/tenantinfo?database=%2Flocal%2Fexample&path=%2Flocal%2Fexample&tablets=false&storage=true&memory=true' \
  | python3 -m json.tool \
  | grep -E 'Resources|StorageGroups|StorageAllocatedLimit|DatabaseStorage' -A 20
```

Host-level PDisk-byte checks should use placeholders in reusable docs:

```bash
sudo du -h -d 1 /path/to/ydb-pdisks /var/lib/docker/volumes/ydb-local-data/_data/pdisks 2>/dev/null || true

sudo find /path/to/ydb-pdisks /var/lib/docker/volumes/ydb-local-data/_data/pdisks \
  -maxdepth 1 -type f -printf '%p %s bytes\n' 2>/dev/null | sort
```

Useful leftover checks after moving storage:

```bash
docker volume ls --format '{{.Name}}' | grep -E 'ydb|local' || true

docker ps -a --filter volume=ydb-local-data --format 'table {{.Names}}\t{{.Status}}'

sudo find /path/to/storage /var/lib/docker/volumes -maxdepth 4 \
  \( -path '*ydb*pdisks*' -o -path '*ydb-dump*' -o -path '*ydb-data*' -o -path '*ydb-local-data*' \) \
  -print 2>/dev/null | sort
```

Document both current and historical pool counts carefully. Old notes can become stale after storage expansion.

## Auth and Exposure

Hardened checks:

- tenant `/local/<tenant>` remains `RUNNING`
- `scheme ls /local/<tenant>` succeeds
- `scheme describe /local/<tenant>/<known-table>` succeeds
- a small `SELECT` from a known table succeeds
- `GraphShardExists` is `true` when GraphShard is required
- one `GraphShard` tablet is visible when GraphShard is required
- GraphShard settings contain `"backend":"Local"` when local metrics storage is expected
- anonymous `viewer/json` returns HTTP `401`
- anonymous YDB CLI/API access fails with `CLIENT_UNAUTHENTICATED`
- direct host access to YDB gRPC is not externally reachable
- public monitoring access goes through HTTPS reverse proxy, not a direct public Docker bind
- public `/viewer/json/whoami` returns `401`, not `503`, once reverse proxy and local viewer are both healthy
- stale YDB Docker volumes are absent only after old storage is intentionally removed

Do not include live IPs, domains, or secret-bearing commands in reusable docs unless the user explicitly asks for private host documentation.
