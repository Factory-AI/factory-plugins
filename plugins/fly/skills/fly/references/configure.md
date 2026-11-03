# Configure: fly.toml and app configuration

`fly.toml` is the app's declarative configuration. `fly deploy` applies it;
`fly launch` writes the first version. Edit the file, validate, deploy.

```bash
fly config show -a <app>                  # what the platform currently has (JSON)
fly config show -a <app> --toml           # same, as TOML
fly config show --local                   # parse the local fly.toml
fly config validate [-c fly.toml]         # schema check against the platform
fly config validate --strict              # also reject unknown keys
fly config save -a <app>                  # write the platform's config to fly.toml
fly config env -a <app>                   # runtime environment variable names
```

Environment variables live in `[env]`. Anything sensitive is a secret
(`fly secrets set`), which overrides an `[env]` key of the same name and is
never readable back. Names cannot start with `FLY_`.

## Top level

| Key | Values | Default | Notes |
|---|---|---|---|
| `app` | string | | First line. Builds `<app>.fly.dev`. |
| `primary_region` | region code | | Where `fly deploy` creates Machines; exported as `PRIMARY_REGION`. |
| `kill_signal` | `SIGINT`, `SIGTERM`, `SIGQUIT`, `SIGUSR1`, `SIGUSR2`, `SIGKILL`, `SIGSTOP` | `SIGINT` | First signal on shutdown. |
| `kill_timeout` | seconds, max 300 | 5 | Grace before `SIGTERM`, then kill. |
| `console_command` | string | | What `fly console` runs. |
| `swap_size_mb` | integer | unset | Adds swap. More memory is usually the better fix. |

Shutdown order on deploy, `fly machine stop` or autostop: `kill_signal`, wait
`kill_timeout`, then `SIGTERM` unconditionally.

## `[build]`

```toml
[build]
  dockerfile = "Dockerfile"        # path or URL; context is the project root
  ignorefile = ".dockerignore"
  build-target = "runtime"         # multistage target; needs CMD or ENTRYPOINT
  # image = "ghcr.io/org/app:tag"  # deploy an image as-is, no build
  # builder = "paketobuildpacks/builder:base"
  # buildpacks = ["..."]
  [build.args]
    NODE_ENV = "production"        # build-time only; --build-arg overrides
```

## `[deploy]`

```toml
[deploy]
  release_command = "bin/rails db:migrate"   # temp Machine, no volumes, non-zero aborts
  release_command_timeout = "5m"
  strategy = "rolling"                       # rolling | immediate | canary | bluegreen
  max_unavailable = 0.33                     # rolling only: count (>=1) or fraction (0-1)
  wait_timeout = "5m"
  [deploy.release_command_vm]
    size = "shared-cpu-2x"
    memory = "1gb"
```

Details and strategy trade-offs in [deploy.md](deploy.md).

## `[env]`

```toml
[env]
  PORT = "8080"
  LOG_LEVEL = "info"
```

String values only. Secrets win over env on name collision.

## `[http_service]`

Shorthand for an HTTP app on 80/443. Use `[[services]]` for other ports or
raw TCP/UDP.

```toml
[http_service]
  internal_port = 8080            # the port the app listens on; must bind 0.0.0.0
  force_https = true
  auto_stop_machines = "stop"     # off | stop | suspend
  auto_start_machines = true
  min_machines_running = 0        # primary region only; applies when auto_stop is on
  processes = ["web"]             # which process groups receive this service

  [http_service.concurrency]
    type = "requests"             # connections (default) | requests (recommended for HTTP)
    soft_limit = 200              # above this, lower routing priority; drives autostop/autostart
    hard_limit = 250              # above this, no new traffic

  [[http_service.checks]]
    grace_period = "10s"
    interval = "30s"
    timeout = "5s"
    method = "GET"
    path = "/healthz"             # must return 2xx; 301/302 counts as failure

  [http_service.http_options]
    idle_timeout = 600
    h2_backend = false            # true for HTTP/2 cleartext backends
    [http_service.http_options.response]
      pristine = false            # true removes Fly's Server/Via/Fly-Request-Id headers
      [http_service.http_options.response.headers]
        X-Frame-Options = "DENY"
        Server = false            # false removes a header

  [http_service.tls_options]
    alpn = ["h2", "http/1.1"]
    versions = ["TLSv1.2", "TLSv1.3"]
    default_self_signed = false
```

Enable `auto_stop_machines` and `auto_start_machines` together or disable
both. Autostop acts on excess capacity judged by `soft_limit`; autostart
wakes a stopped Machine on the first request to a region with none running.
`min_machines_running` keeps that many awake in the primary region only.

`[[http_service.machine_checks]]` (fields `image`, `entrypoint`, `command`,
`kill_signal`, `kill_timeout`) runs a test Machine against each new Machine
during `rolling` and `canary` deploys; it gets `FLY_TEST_MACHINE_IP`.

`replay_cache` rules under `http_options` (`path_prefix`, `ttl_seconds` >= 10,
`type` `cookie`|`header`, `name`, `allow_bypass`) pin repeat requests to the
Machine that first answered.

## `[[services]]`

```toml
[[services]]
  protocol = "tcp"                # tcp | udp
  internal_port = 5432
  processes = ["db"]
  auto_stop_machines = "off"
  auto_start_machines = true
  min_machines_running = 1

  [[services.ports]]
    port = 5432
    handlers = ["pg_tls"]         # http, tls, proxy_proto, pg_tls; omit for udp
    # start_port = 7000; end_port = 7100   # a range instead of port
    # force_https = true (with http handler)

  [services.concurrency]
    type = "connections"
    soft_limit = 100
    hard_limit = 200

  [[services.tcp_checks]]
    grace_period = "1s"
    interval = "15s"
    timeout = "2s"

  [[services.http_checks]]
    interval = "10s"
    grace_period = "5s"
    method = "get"
    path = "/health"
    protocol = "http"
    timeout = "2s"
    tls_skip_verify = false
    [services.http_checks.headers]
      Host = "example.com"
```

No `[http_service]` and no `[[services]]` means no public or Flycast routing
at all; the app is reachable only on the private network.

## `[processes]`

```toml
[processes]
  web = "bin/rails server"
  worker = "bundle exec sidekiq"
```

Each key is a process group with its own Machines. Services, mounts, vm
sections, restarts, checks, metrics and files target groups via `processes`.
`fly scale count web=2 worker=1` sizes them. An app with no `[processes]` has
one implicit group named `app`.

## `[[vm]]`

```toml
[[vm]]
  size = "shared-cpu-1x"          # lowest precedence; fly platform vm-sizes
  memory = "512mb"                # or integer MB
  cpus = 1                        # 1, 2, 4, 8, 16 depending on cpu_kind
  cpu_kind = "shared"             # shared | performance
  processes = ["web"]
  # persist_rootfs = "never"      # never | restart | always
  # host_dedication_id = "..."
  # kernel_args = [...]
```

If `[[vm]]` exists, `fly deploy` resets Machines to it and overrides any
`fly scale vm`/`memory` change. Keep them in agreement.

## `[[mounts]]`

```toml
[[mounts]]
  source = "data"                 # volume name
  destination = "/data"           # not /
  processes = ["web"]
  initial_size = "10gb"           # only when launch/deploy creates the volume
  snapshot_retention = 5          # days, 1-60
  scheduled_snapshots = true
  auto_extend_size_threshold = 80 # percent used
  auto_extend_size_increment = "1GB"
  auto_extend_size_limit = "50GB"
```

One volume per Machine. See [volumes.md](volumes.md).

## `[[restart]]`

```toml
[[restart]]
  policy = "on-failure"           # always | never | on-failure
  retries = 10
  processes = ["worker"]
```

## `[checks]`

Top-level health checks independent of routing:

```toml
[checks]
  [checks.alive]
    type = "http"                 # tcp | http
    port = 8080
    grace_period = "10s"
    interval = "15s"
    timeout = "2s"
    method = "GET"
    path = "/healthz"
    processes = ["web"]
    [checks.alive.headers]
      Authorization = "Bearer ..."  # do not put real secrets here; use a dedicated unauthenticated path
```

Durations default to milliseconds when unitless; always write units.

## `[[metrics]]`

```toml
[[metrics]]
  port = 9091
  path = "/metrics"
  processes = ["web"]
```

Fly scrapes Prometheus metrics from here; see them in `fly dashboard metrics`.

## `[[statics]]`

```toml
[[statics]]
  guest_path = "/app/public"
  url_prefix = "/static"
  # tigris_bucket = "my-bucket"; index_document = "index.html"
```

Up to 10 mappings, non-overlapping prefixes, served by the proxy from a
running Machine (autostart applies). No custom headers, no symlinks.

## `[[files]]`

```toml
[[files]]
  guest_path = "/etc/app/config.yml"
  local_path = "config/production.yml"    # or raw_value = "<base64>", or secret_name = "CONFIG_B64"
  processes = ["web"]
```

Exactly one source. An empty source removes the file on the next deploy.
`secret_name` requires the secret value to be base64.

## `[experimental]`

```toml
[experimental]
  cmd = ["bin/server"]            # overrides CMD
  entrypoint = ["/entrypoint.sh"] # overrides ENTRYPOINT
  # exec = [...]                  # overrides both
```

## Patterns

- **Web + worker**: `[processes]` with both; `[http_service] processes =
  ["web"]`; `[[vm]]` per group; `fly scale count web=2 worker=1`.
- **Scale to zero dev app**: `auto_stop_machines = "stop"`,
  `auto_start_machines = true`, `min_machines_running = 0`, `--ha=false`.
- **Always-on API**: `auto_stop_machines = "off"` or `min_machines_running =
  1`, two Machines in the primary region, `type = "requests"` concurrency.
- **Internal service**: no `[http_service]`; reach it on
  `<app>.internal` or allocate Flycast ([networking.md](networking.md)).

Checked against docs.fly.io/reference/configuration on 2026-10-28.
