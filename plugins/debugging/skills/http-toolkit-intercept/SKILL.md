---
name: http-toolkit-intercept
description: Intercept and debug HTTP traffic from a Bun- or Node-based CLI using HTTP Toolkit. Use when you need to inspect LLM API calls, backend requests, auth flows, or debug network-level issues.
---

# HTTP Toolkit Intercept

Use this skill when you need authoritative evidence of what your CLI sent to a remote API and what it received back while verifying a code change.

The reliable pattern is:

1. Start HTTP Toolkit correctly.
2. Run the CLI through the proxy in a mode that produces a machine-readable log (e.g. `--output-format json`).
3. Export outbound HTTP requests from HTTP Toolkit.
4. Pair the outbound HTTP export with the inbound CLI session log.

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

### 1. Launch your CLI with the proxy env vars set

The canonical Bun-friendly pattern:

```bash
BUN_CONFIG_PROXY="http://127.0.0.1:8000" \
HTTP_PROXY="http://127.0.0.1:8000" \
HTTPS_PROXY="http://127.0.0.1:8000" \
ALL_PROXY="http://127.0.0.1:8000" \
NODE_TLS_REJECT_UNAUTHORIZED=0 \
<your-cli> <args>
```

> `NODE_TLS_REJECT_UNAUTHORIZED=0` is only appropriate for controlled local debugging. See "TLS Safety" below.

### 2. Capture the inbound session log

If your CLI supports a machine-readable output mode (e.g. `--output-format stream-json`, `--json`, `--log-level debug`), pipe it to a file:

```bash
<your-cli> exec --output-format stream-json "your prompt" \
  > /tmp/cli-intercept-stdout.log 2> /tmp/cli-intercept-stderr.log
```

### 3. Export outbound HTTP from HTTP Toolkit

Either:

- Use the HTTP Toolkit GUI export (File → Export → JSON), or
- Hit the admin API directly. The exact endpoint depends on your HTTP Toolkit version; inspect DevTools in the HTTP Toolkit UI to see the requests it makes.

### 4. Cross-reference the two streams

- **Outbound HTTP** (from HTTP Toolkit): authoritative for request bodies, headers, auth tokens, retry timing.
- **Inbound session log** (from the CLI): authoritative for how your code reacted to the responses.

Together they answer: "what did we send?" and "what did we do with the response?"

## What finally worked for payload verification

The critical correct pathways that proved reliable were:

1. **Use a non-interactive / exec mode, not the TUI, when verifying payloads**
   - Interactive TUIs are slower and much harder to analyze.
   - Use whatever your CLI has for scripting (`--output-format`, `--json`, `--headless`).

2. **Treat outbound and inbound as separate evidence sources**
   - HTTP Toolkit gives outbound HTTP requests.
   - The CLI's session log gives inbound assistant/tool behavior.
   - You need both to answer: "what did the model receive?" and "what did it actually do?"

3. **Use Bun's proxy env var explicitly**
   - `BUN_CONFIG_PROXY` is the important one.
   - Setting only `HTTP_PROXY`/`HTTPS_PROXY` is not enough for Bun.

4. **Disable TLS verification only for controlled local debugging when needed**
   - If the HTTP Toolkit CA is not trusted locally, use `NODE_TLS_REJECT_UNAUTHORIZED=0` as an escape hatch.
   - Prefer trusting the CA in your OS / Node trust store instead.
   - Never disable TLS in production repros.

5. **Keep runs bounded**
   - Long tool-heavy prompts can take time.
   - If you only need to prove request shape, export after the LLM request is observed — you do not always need to wait for full completion.

## Key Facts

- **Bun requires `BUN_CONFIG_PROXY`** — treat it as mandatory
- **HTTP_PROXY / HTTPS_PROXY alone are silently ignored by Bun**
- **HTTP Toolkit admin API is request-oriented** — outbound HTTP comes from HTTP Toolkit, inbound behavior comes from the CLI log
- **TLS is verified by default** — `NODE_TLS_REJECT_UNAUTHORIZED=0` is a local-dev escape hatch only

## Bun Proxy Gotchas

| Approach | Works? | Notes |
|----------|--------|-------|
| `BUN_CONFIG_PROXY` | Yes | Bun's official proxy env var |
| `HTTP_PROXY` / `HTTPS_PROXY` | No (Bun) | Silently ignored by bun but respected by some libraries used via Bun |
| `SSL_CERT_FILE` with combined CA | Unreliable | Can break on some Bun builds; verify in your local Bun version |
| `NODE_TLS_REJECT_UNAUTHORIZED=0` | Local dev only | Disables TLS verification |
| Running via `tsx` / Node.js | Depends on your CLI | Some CLIs have native deps (`.dylib`) that require Bun |

### TLS Safety Guardrails

- Keep TLS verification enabled whenever possible.
- Prefer trusting the HTTP Toolkit CA in your local trust store instead of disabling verification.
- Use `NODE_TLS_REJECT_UNAUTHORIZED=0` only for controlled local debugging in development.
- Never disable TLS when intercepting production traffic.

## Inspecting Captured Logs

### Filter to LLM-call events only

If your CLI emits newline-delimited JSON, use `jq`:

```bash
jq -c 'select(.type == "tool_call" or .type == "message")' /tmp/cli-intercept-stdout.log
```

### Match outbound HTTP to inbound events

Sort both streams by timestamp, then interleave them. The sequence usually is:

```
outbound POST /chat/completions   (from HTTP Toolkit)
inbound  message role=assistant    (from CLI log)
inbound  tool_call tool=foo        (from CLI log)
outbound POST /chat/completions   (next turn)
```

If a CLI log shows an outbound request that HTTP Toolkit didn't capture, that's a proxy-config bug.

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| CLI hangs, no events after startup | Proxy env vars not reaching the process, or TLS verification blocking | Re-run with `BUN_CONFIG_PROXY` explicitly set; if necessary set `NODE_TLS_REJECT_UNAUTHORIZED=0` in dev |
| `ECONNRESET` on every request | Using `HTTP_PROXY` not `BUN_CONFIG_PROXY` with Bun | Switch to `BUN_CONFIG_PROXY` |
| TLS cert errors via proxy | MITM CA not trusted | Trust HTTP Toolkit CA locally, or use `NODE_TLS_REJECT_UNAUTHORIZED=0` in dev only |
| `ERR_UNKNOWN_FILE_EXTENSION .dylib` | Running with tsx/Node on a CLI that needs Bun | Run the CLI with `bun` directly |
| HTTP Toolkit API 403s on `/config` | Auth-gated config endpoint | Treat 200/401/403 as reachable; only connection failure means the server is dead |
| Export has outbound data but no matching inbound events | Didn't capture the CLI log | Add `> /tmp/cli.log` redirection to the CLI launch |
| HTTP Toolkit misses the first request | Started capturing after the process launched | Start HTTP Toolkit first, THEN launch the CLI |
