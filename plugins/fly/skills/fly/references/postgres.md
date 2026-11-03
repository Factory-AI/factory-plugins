# Postgres: Managed Postgres and unmanaged clusters

Fly has two Postgres offerings. **Managed Postgres (MPG)** is a service Fly
operates (backups, HA, failover, monitoring, connection pooling), identified
by a cluster ID and driven with `fly mpg` and the MCP tools. **Unmanaged
Postgres** (`fly postgres`) is a Fly app running Postgres on Machines and
volumes in your org that you operate yourself. New work should use MPG unless
the user asks for the app-based cluster.

## Managed Postgres with flyctl

```bash
fly mpg list [-o <org>]                                    # clusters and IDs
fly mpg status <cluster-id> [--json]                       # state, plan, region, endpoints
fly mpg create -n <name> -o <org> -r <region> --plan Basic --volume-size 10 [--pg-major-version 17] [--enable-postgis-support]
fly mpg attach <cluster-id> -a <app> [--database <db>] [--username <user>] [--variable-name DATABASE_URL]
fly mpg detach <cluster-id> -a <app>                       # removes the secret from the app
fly mpg connect <cluster-id> [-d <db>] [-u <user>]         # psql
fly mpg proxy <cluster-id> [-p 16380]                      # local port for your own client
fly mpg databases list|create <cluster-id>
fly mpg databases extensions list|enable <cluster-id> ...  # read its help for exact args
fly mpg users list|create|delete|rotate-password|set-role <cluster-id> ...
fly mpg backup list|create <cluster-id>
fly mpg restore <cluster-id> --backup-id <id> [-n <name>]  # into a NEW cluster
fly mpg restore <cluster-id> --pitr-time 2026-06-01T12:00:00Z [-n <name>]
fly mpg fork <cluster-id> [-n <name>] [--plan <plan>]      # new cluster from current data
fly mpg destroy <cluster-id> [-y]                          # irreversible
```

Plans: `Basic` (shared-2x, 1 GB), `Starter` (shared-2x, 2 GB), `Launch`
(performance-2x, 8 GB), `Scale` (performance-4x, 32 GB), `Performance`
(performance-8x, 64 GB). Storage is billed per GB. Postgres 16 or 17 at
creation; there is no in-place major upgrade, so an upgrade is a new cluster
plus `pg_dump`/`pg_restore`. `pgvector` is included; PostGIS must be enabled
at creation. Every plan includes HA, backups and pooling.

Workflow for "add a database to my app":

1. `fly mpg list` to check for an existing cluster to reuse.
2. Confirm org, region (next to the app's `primary_region`), plan and size;
   this is a billable resource. Create.
3. Provisioning is asynchronous. Poll `fly mpg status` until ready. An
   accepted request is not a ready database.
4. `fly mpg attach <id> -a <app>` sets `DATABASE_URL` (or `--variable-name`)
   as a secret and redeploys unless the app has no Machines yet.
5. Verify with `fly secrets list -a <app>` (the name and digest, never the
   value) and, after deploy, the app's own health.

Connection pooling is on by default; the attached URL points at the cluster's
private endpoint, reachable from the org network only. For local access use
`fly mpg proxy` or `fly mpg connect`.

### Backups, restore, fork

Backups are automatic with point-in-time recovery inside the retention
window. `fly mpg backup create` takes an extra one; `fly mpg restore` always
provisions a **new** cluster (billed separately) and leaves the source alone,
so a "restore in place" is: restore to a new cluster, verify, re-attach the
app to the new cluster ID, then destroy the old one when the user says so.
`fly mpg fork` is the same mechanism from the current data, for staging
copies and upgrade rehearsals.

### Destroy

`fly mpg destroy` deletes the cluster and all data with no undo. Confirm the
exact cluster ID and name, check `fly mpg backup list`, detach apps first,
and state what is lost.

## Managed Postgres over MCP

Tools must be available and granted; Managed Postgres access is separate from
app access, and listing organizations does not imply it.

- Discover: `list_postgres_clusters` for an authorized org,
  `get_postgres_cluster` (status, and endpoints once ready). Use the IDs it
  returns; never infer a cluster from a similarly named app.
- Create or fork: `get_postgres_plans` (cheapest first, which is not
  permission to pick one), then `create_postgres_cluster` or
  `fork_postgres_cluster` after the user confirms the billable resource.
  Record the returned ID. Poll `get_postgres_cluster` until ready. After a
  timeout, list clusters before retrying; never create duplicates to recover
  an unknown outcome.
- Databases and extensions: `list_postgres_databases` then
  `create_postgres_database`; `list_postgres_extensions` for the exact
  database then `enable_postgres_extension`. PostGIS needs a PostGIS-enabled
  cluster. Verify by listing again.
- Backups: `list_postgres_backups` then `create_postgres_backup`; one at a
  time, asynchronous, not usable until complete. No restore or PITR tool.
- Users: `list_postgres_users` only; no creation, password change or
  password retrieval.
- Diagnostics: `list_postgres_active_queries`, `list_postgres_slow_queries`
  return **full query text**, which can carry credentials and personal data.
  The hook asks before either. Summarize timings and patterns; do not repeat
  literals. Slow-query counters are cumulative totals, not deltas for the
  lookback window.

Not exposed over MCP: deletion, attach/detach, users and passwords, restore,
arbitrary SQL. Say so and use `fly mpg` for those steps; do not simulate them.

## Unmanaged Postgres (`fly postgres`)

An app named by you, running Postgres on Machines with volumes, managed by
`repmgr` (`--flex`, default).

```bash
fly postgres create -n <name> -o <org> -r <region> [--initial-cluster-size 1|3] [--vm-size shared-cpu-1x] [--volume-size 10] [--enable-backups]
fly postgres attach <pg-app> -a <app> [--database-name <db>] [--database-user <user>] [--variable-name DATABASE_URL]
fly postgres detach <pg-app> -a <app>
fly postgres connect -a <pg-app>                 # psql on the primary
fly postgres list
fly postgres db list|create -a <pg-app>
fly postgres users list|create -a <pg-app>
fly postgres config show|update -a <pg-app>
fly postgres failover -a <pg-app>                # interrupts connections
fly postgres restart -a <pg-app>
fly postgres backup list|create|restore -a <pg-app>   # with --enable-backups (Tigris WAL)
fly postgres import <source-uri> -a <pg-app>
fly postgres events list -a <pg-app>
```

Because it is an ordinary app, everything in [machines.md](machines.md),
[volumes.md](volumes.md) and [scale.md](scale.md) applies: `fly status -a
<pg-app>`, `fly volumes list`, `fly logs`, `fly ssh console`. You are
responsible for upgrades, disk space (`fly volumes extend`), replicas and
snapshots. Point that out when a user picks it for production.

Checked against flyctl v0.4.123, docs.fly.io/mpg and mcp.fly.io on 2026-10-28.
