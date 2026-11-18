# debugging

Tools for inspecting runtime behavior and disk usage: HTTP traffic, process-level traces, and safe retirement of obsolete local worktrees.

## Skills

### `disk-cleanup`

Diagnose low disk space, qualify owned disposable outputs or obsolete worktrees, and perform only explicitly authorized cleanup. Preserves unique work, package caches, shared Git state, and active consumers, with local-first operation receipts and honest before/after measurements.

See [the workflow](skills/disk-cleanup/SKILL.md) and its [safety cases](skills/disk-cleanup/safety-cases.md).

### `http-toolkit-intercept`

Intercept and debug HTTP traffic from any CLI, service, or script using HTTP Toolkit. Works with any runtime (Node.js, Bun, Deno, Python, Go, Ruby, Java, .NET, Rust, shell, etc.) that respects a proxy. Pairs the outbound proxy log with a machine-readable session log so you can verify LLM API calls, backend requests, auth flows, and wire-level behavior.

## Install

```bash
droid plugin install debugging@factory-plugins
```
