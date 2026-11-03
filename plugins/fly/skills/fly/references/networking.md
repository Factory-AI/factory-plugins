# Networking, domains, and certificates

Every app with `[http_service]` or `[[services]]` is reachable at
`<app>.fly.dev` through Fly Proxy, which terminates TLS and routes to the
nearest healthy Machine. Custom domains add a certificate and DNS records.
Apps also share an org-wide private IPv6 network.

## IP addresses

```bash
fly ips list -a <app>                     # public v4/v6, Flycast, egress
fly ips allocate -a <app>                 # the recommended set (shared v4 + dedicated v6)
fly ips allocate-v4 --shared -a <app>     # free shared IPv4 (default for new apps)
fly ips allocate-v4 -a <app>              # dedicated IPv4, billable monthly
fly ips allocate-v6 -a <app>              # dedicated public IPv6 (free)
fly ips allocate-v6 --private -a <app>    # Flycast: private IPv6 reachable from the org network through the proxy
fly ips allocate-egress -a <app>          # static egress IPs for allow-listing outbound traffic
fly ips release <address> -a <app>        # the app stops answering on it; confirm first
fly ips private -a <app>                  # each Machine's 6PN address
```

A shared IPv4 works for HTTP/HTTPS via SNI. Raw TCP on a custom port, or a
service that needs its own address, needs a dedicated IPv4. Allocation does
not require a deploy. MCP: `list_app_ips` (read-only); allocation and release
are CLI-only.

## Custom domains and TLS

```bash
fly certs add example.com -a <app>              # prints the DNS records to create
fly certs add "*.example.com" -a <app>          # wildcard; needs the _acme-challenge CNAME
fly certs check example.com -a <app>            # trigger validation and show DNS status
fly certs show example.com -a <app>
fly certs setup example.com -a <app>            # setup instructions again
fly certs list -a <app>
fly certs import example.com --fullchain chain.pem --private-key key.pem -a <app>
fly certs remove example.com -a <app>           # TLS for that hostname stops; confirm first
```

DNS the user must create, from the `certs add` output:

| Record | Points to | When |
|---|---|---|
| `A` | the app's IPv4 | apex domains, or any direct connection |
| `AAAA` | the app's IPv6 | apex domains; also one of the ownership proofs |
| `CNAME` | `<app>.fly.dev` | subdomains such as `www`; apex only with CNAME flattening |
| `CNAME _acme-challenge.<host>` | `<host>.<id>.flydns.net` (from the output) | wildcards, or issuing before traffic moves |
| `TXT _fly-ownership.<host>` | value from the output | behind a CDN, or custom certificate import |

Issuance needs at least one of: an AAAA record, the `_acme-challenge` CNAME,
or the `_fly-ownership` TXT. Validation is TLS-ALPN by default, HTTP-01 when
that is unavailable, DNS-01 for wildcards. Let's Encrypt limits apply (50
certificates per registered domain per week, 5 failed validations per
hostname per hour), so do not loop `certs check`; run it after the user
confirms DNS has propagated.

Behind Cloudflare's proxy: add the `_fly-ownership` TXT, use "Full" or "Full
(Strict)" SSL mode ("Flexible" causes redirect loops), and if HTTP-01 stays
blocked import a Cloudflare Origin Certificate. A custom certificate shows
`pending_ownership` until AAAA or the TXT record verifies it; custom and Let's
Encrypt certificates can coexist, custom served first.

MCP: `list_app_certificates`, `get_app_certificate` (read-only). Creating
certificates and changing DNS are outside the MCP server.

Verify with `fly certs check <host>` (status `Ready`/`Issued`) and
`curl -sSI https://<host>`.

## Private networking (6PN)

Every Machine has an IPv6 address on the org's private network and a DNS name
in `.internal`:

| Name | Resolves to |
|---|---|
| `<app>.internal` | all Machines of the app |
| `<region>.<app>.internal` | Machines in one region |
| `<machine-id>.vm.<app>.internal` | one Machine |
| `top1.nearest.of.<app>.internal` | the nearest Machine |
| `<process-group>.process.<app>.internal` | one process group |
| `_apps.internal` TXT | app names in the org |

Traffic between apps in the same org stays private without any service
configuration. Apps on a custom network (`fly apps create --network`) are
isolated from the default network.

```bash
fly dig <app>.internal -a <app>            # query Fly's internal DNS
fly ping <app>.internal -a <app>           # ICMP over WireGuard
```

## Flycast

Flycast is a private IPv6 that goes **through Fly Proxy** (so services,
autostart, load balancing and health checks apply) but is reachable only from
the org network. Use it for internal APIs that should scale to zero:

```bash
fly ips allocate-v6 --private -a <app>
# then connect to http://<app>.flycast:<port> from another app in the org
```

The app still needs `[http_service]`/`[[services]]` for the proxy to route;
drop public IPs (`fly ips release`) to make it private-only.

## Reaching Machines from your laptop

```bash
fly proxy 15432:5432 -a <db-app>                 # local port -> service port on a Machine
fly proxy 8080 -a <app> -s                       # pick the Machine interactively
fly proxy 5432 <machine-id>.vm.<app>.internal -a <app>
fly ssh console -a <app>                         # shell; see operate.md
fly wireguard create <org> <region> <peer-name> wg.conf   # a persistent WireGuard peer
fly wireguard list <org>
fly wireguard remove <org> <peer-name>
fly wireguard websockets enable                 # tunnel WireGuard over WebSockets on restrictive networks
```

`fly proxy` and `fly ssh` set up a temporary WireGuard tunnel through the
background `fly agent`; `fly doctor` diagnoses it. The MCP server exposes no
tunnelling.

## Services and ports

Public ports, handlers (`http`, `tls`, `proxy_proto`, `pg_tls`), concurrency
and health checks are configured in `fly.toml` ([configure.md](configure.md)).
`fly services list -a <app>` shows what is live. The app must listen on
`0.0.0.0:<internal_port>`; binding `127.0.0.1` is the top cause of "not
reachable by fly-proxy".

Checked against flyctl v0.4.123 and docs.fly.io/networking/custom-domain on 2026-10-28.
