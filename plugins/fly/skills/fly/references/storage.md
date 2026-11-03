# Object storage, Redis, and extensions

Fly provisions several partner-run services through flyctl. Each creates a
billable resource in the partner's system, bound to your org, and usually
sets secrets on an app. Confirm the org, region and plan before creating, and
read each command's `--help`; partner flags change.

## Tigris object storage (`fly storage`)

S3-compatible, globally distributed buckets.

```bash
fly storage list [-o <org>]
fly storage create -n <bucket> -o <org> [-a <app>] [--public]
fly storage status <bucket>
fly storage update <bucket> [--public|--private] ...
fly storage dashboard <bucket>
fly storage destroy <bucket> [-y]              # deletes the bucket and every object
```

`create` with `-a <app>` sets `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`,
`AWS_ENDPOINT_URL_S3`, `AWS_REGION` and `BUCKET_NAME` on the app as secrets.
Any S3 client works against the endpoint. Shadow-bucket flags
(`--shadow-endpoint`, `--shadow-name`, ...) migrate from an existing S3
bucket; they take credentials, so take them from the user's environment
rather than chat. `[[statics]] tigris_bucket` in `fly.toml` serves a bucket
through the proxy ([configure.md](configure.md)). `fly launch` offers a
bucket; `--no-object-storage` skips it.

## Upstash Redis (`fly redis`)

```bash
fly redis plans
fly redis list [-o <org>]
fly redis create -n <name> -o <org> -r <region> [--plan <plan>] [--replica-regions ams,syd] [--no-replicas] [--enable-eviction]
fly redis status <name>
fly redis connect <name>                        # redis-cli
fly redis proxy <name>                          # local port
fly redis update <name> ...
fly redis reset <name>                          # new password; breaks existing connections
fly redis destroy <name>                        # irreversible
fly redis dashboard
```

The connection URL is shown at creation and in `status`; set it on the app as
`REDIS_URL` with `fly secrets set` without echoing it. Read replicas are per
region; the primary region should match the app's `primary_region`.

## Other extensions (`fly extensions`)

```bash
fly extensions sentry create -a <app>           # Sentry project; sets SENTRY_DSN
fly extensions arcjet ...                       # bot protection and rate limiting
fly extensions wafris ...                       # WAF
fly extensions vector ...                       # Upstash Vector index
fly extensions kubernetes ...                   # Fly Kubernetes clusters
fly extensions storage ...                      # alias of fly storage
```

Each has `create`, `list`, `status`, `destroy`, `dashboard` and often
`update`. They may prompt to accept the partner's terms in the browser; let
the user do that.

## Monitoring add-ons

`fly dashboard -a <app>` opens the app in the web UI; `fly dashboard metrics`
opens Grafana. `[[metrics]]` in `fly.toml` exposes a Prometheus endpoint Fly
scrapes. `fly synthetics` manages synthetic monitoring; `fly incidents` lists
platform incidents affecting your apps; `fly platform status` shows platform
health.

## Over MCP

None of these partner resources are exposed through the Fly MCP server. Use
flyctl, and say so if asked.

Checked against flyctl v0.4.123 on 2026-10-28.
