---
name: http-toolkit-intercept
description: Intercept and debug HTTP traffic from any CLI, service, or script using HTTP Toolkit. Use when you need to inspect LLM API calls, backend requests, auth flows, or debug network-level issues across any language or runtime.
---

# HTTP Toolkit Intercept

Use this skill when you need authoritative evidence of what your program sent to a remote API and what it received back while verifying a code change. Works across any runtime — Node.js, Bun, Deno, Python, Go, Ruby, Java/JVM, .NET, PHP, Rust, shell scripts, etc. — as long as the process respects a proxy.

The reliable pattern is:

1. Start HTTP Toolkit correctly.
2. Run the program through the proxy in a mode that produces a machine-readable log (e.g. `--output-format json`, debug logging, or structured stdout).
3. Export outbound HTTP requests from HTTP Toolkit.
4. Pair the outbound HTTP export with the inbound program session log.

Do not rely on TUI screenshots alone when the question is about request payloads, auth headers, or wire-level behavior.

## Prerequisites / Known-Good Launch

### Start HTTP Toolkit

On Linux/headless environments, plain `httptoolkit` often fails due to sandbox/X11 issues. Prefer:

```bash
xvfb-run --auto-servernum httptoolkit --no-sandbox
```

If a stale server is already running on ports `45456/45457`, stop it first:

```bash
pkill -f "HTTP Toolkit Server|httptoolkit|xvfb-run --auto-servernum httptoolkit" || true
```

### Verify the proxy is reachable

```bash
# HTTP Toolkit's admin API lives on port 45456/45457 by default.
# Treat 200/401/403 as "reachable"; only connection failure means the server is dead.
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:45456/config
```

## Quick Start

### 1. Launch your program with the proxy env vars set

The canonical env vars most runtimes honor:

```bash
HTTP_PROXY="http://127.0.0.1:8000" \
HTTPS_PROXY="http://127.0.0.1:8000" \
ALL_PROXY="http://127.0.0.1:8000" \
NO_PROXY="" \
<your-program> <args>
```

Some runtimes require an extra env var or flag — see the "Runtime proxy matrix" below.

> If the HTTP Toolkit CA is not trusted by your runtime, TLS verification will fail. See "TLS Safety" below. Disabling TLS verification is appropriate only for controlled local debugging.

### 2. Capture the inbound session log

If your program supports a machine-readable output mode (e.g. `--output-format stream-json`, `--json`, `--log-level debug`, structured stdout, or writing to a file), pipe it to a file:

```bash
<your-program> exec --output-format stream-json "your input" \
  > /tmp/session-stdout.log 2> /tmp/session-stderr.log
```

### 3. Export outbound HTTP from HTTP Toolkit

Either:

- Use the HTTP Toolkit GUI export (File → Export → JSON / HAR), or
- Hit the admin API directly. The exact endpoint depends on your HTTP Toolkit version; inspect DevTools in the HTTP Toolkit UI to see the requests it makes.

### 4. Cross-reference the two streams

- **Outbound HTTP** (from HTTP Toolkit): authoritative for request bodies, headers, auth tokens, retry timing.
- **Inbound session log** (from the program): authoritative for how your code reacted to the responses.

Together they answer: "what did we send?" and "what did we do with the response?"

## What finally worked for payload verification

The critical correct pathways that proved reliable were:

1. **Use a non-interactive / exec mode, not the TUI, when verifying payloads**
   - Interactive TUIs are slower and much harder to analyze.
   - Use whatever your program has for scripting (`--output-format`, `--json`, `--headless`, `--batch`).

2. **Treat outbound and inbound as separate evidence sources**
   - HTTP Toolkit gives outbound HTTP requests.
   - The program's session log gives inbound assistant/tool/application behavior.
   - You need both to answer: "what did the remote return?" and "what did the program actually do with it?"

3. **Set the proxy env vars your specific runtime honors**
   - `HTTP_PROXY`/`HTTPS_PROXY` cover most runtimes, but some (e.g. Bun, Java) require extra vars or flags.
   - See the "Runtime proxy matrix" below.

4. **Disable TLS verification only for controlled local debugging when needed**
   - If the HTTP Toolkit CA is not trusted locally, use the runtime-specific escape hatch to skip verification.
   - Prefer trusting the CA in your OS / language trust store instead.
   - Never disable TLS in production repros.

5. **Keep runs bounded**
   - Long network-heavy sessions can take time.
   - If you only need to prove request shape, export after the relevant request is observed — you do not always need to wait for full completion.

## Key Facts

- **Proxy env vars are runtime-specific** — know which ones your runtime honors
- **HTTP Toolkit admin API is request-oriented** — outbound HTTP comes from HTTP Toolkit, inbound behavior comes from the program log
- **TLS is verified by default** — runtime-specific skip flags are local-dev escape hatches only

## Runtime proxy matrix

| Runtime | Proxy env vars / flags | TLS bypass (local dev only) |
|---------|------------------------|------------------------------|
| **Node.js** | `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY` (most libraries); some HTTP clients need `--proxy` flags or explicit `agent:` option | `NODE_TLS_REJECT_UNAUTHORIZED=0` or `NODE_EXTRA_CA_CERTS=/path/to/ca.pem` |
| **Bun** | `BUN_CONFIG_PROXY` (mandatory — plain `HTTP_PROXY`/`HTTPS_PROXY` are silently ignored for Bun's own fetch) | `NODE_TLS_REJECT_UNAUTHORIZED=0` |
| **Deno** | `HTTP_PROXY`, `HTTPS_PROXY` | `--unsafely-ignore-certificate-errors` |
| **Python (`requests`, `httpx`)** | `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` | `REQUESTS_CA_BUNDLE` / `SSL_CERT_FILE` pointing at the HTTP Toolkit CA, or `verify=False` in code |
| **Python (`urllib`)** | Same env vars | `SSL_CERT_FILE` or disable context verification |
| **Go (`net/http`)** | `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY` (honored via `http.ProxyFromEnvironment`) | `SSL_CERT_FILE` or `InsecureSkipVerify: true` in the transport |
| **Ruby** | `HTTP_PROXY`, `HTTPS_PROXY` | `SSL_CERT_FILE`, or `OpenSSL::SSL::VERIFY_NONE` |
| **Java / JVM** | `-Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=8000 -Dhttps.proxyHost=127.0.0.1 -Dhttps.proxyPort=8000`; env vars are **not** honored by the JVM | Import CA into a truststore and pass `-Djavax.net.ssl.trustStore=...` |
| **.NET / C#** | `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` (on recent runtimes) or configure `HttpClient` explicitly | Trust CA in OS store, or `HttpClientHandler.ServerCertificateCustomValidationCallback` |
| **PHP (curl / cli)** | `HTTP_PROXY`, `HTTPS_PROXY` (or per-call `CURLOPT_PROXY`) | `CURLOPT_SSL_VERIFYPEER => false` |
| **Rust (`reqwest`)** | `HTTP_PROXY`, `HTTPS_PROXY` | `danger_accept_invalid_certs(true)` on the client |
| **curl / shell** | `HTTP_PROXY`, `HTTPS_PROXY` env vars or `-x`/`--proxy` flag | `-k` / `--insecure` |
| **Docker containers** | Pass env vars through with `-e HTTP_PROXY=...`; use `host.docker.internal` (Mac/Win) or `--network=host` (Linux) so the container can reach the proxy | Mount the CA into the image and install it, or use runtime-specific bypass flags |

Prefer trusting the HTTP Toolkit CA in your runtime/OS trust store over disabling verification. Bypass flags should be local-dev only.

### TLS Safety Guardrails

- Keep TLS verification enabled whenever possible.
- Prefer trusting the HTTP Toolkit CA in your local trust store instead of disabling verification.
- Use runtime-specific TLS-bypass flags only for controlled local debugging in development.
- Never disable TLS when intercepting production traffic.

## Inspecting Captured Logs

### Filter to relevant events only

If your program emits newline-delimited JSON, use `jq`:

```bash
jq -c 'select(.type == "tool_call" or .type == "message")' /tmp/session-stdout.log
```

Adjust the filter to match your program's event schema. For plain-text logs, use `rg`/`grep` patterns.

### Match outbound HTTP to inbound events

Sort both streams by timestamp, then interleave them. The sequence usually looks like:

```
outbound POST /api/endpoint        (from HTTP Toolkit)
inbound  event received             (from program log)
inbound  follow-up action           (from program log)
outbound POST /api/endpoint        (next request)
```

If a program log shows an outbound request that HTTP Toolkit didn't capture, that's a proxy-config bug.

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| Program hangs, no events after startup | Proxy env vars not reaching the process, or TLS verification blocking | Re-run with the right env vars for your runtime (see matrix); if necessary, enable the runtime's TLS bypass in dev |
| `ECONNRESET` / `connection reset` on every request | Runtime-specific proxy env var missing (e.g. `BUN_CONFIG_PROXY` for Bun, JVM `-D` flags for Java) | Use the correct runtime-specific proxy config |
| TLS cert errors via proxy | MITM CA not trusted by this runtime | Trust HTTP Toolkit CA in the runtime / OS store, or enable the runtime's TLS bypass in dev only |
| HTTP Toolkit API 403s on `/config` | Auth-gated config endpoint | Treat 200/401/403 as reachable; only connection failure means the server is dead |
| Export has outbound data but no matching inbound events | Didn't capture the program log | Add `> /tmp/session.log` redirection to the launch |
| HTTP Toolkit misses the first request | Started capturing after the process launched | Start HTTP Toolkit first, THEN launch the program |
| Container / VM can't reach `127.0.0.1:8000` | Loopback is container-local | Use `host.docker.internal` (Docker Desktop) or `--network=host` (Linux) |
| Program ignores env vars entirely | Runtime doesn't honor env vars (e.g. JVM) | Use runtime-specific flags (`-Dhttp.proxyHost=...` for JVM, etc.) |
