---
name: hol-guard
description: Set up HOL Guard for supported coding-agent harnesses, review Guard approvals and receipts, or scan skills, plugins, MCP servers, and agent packages before trust.
version: 1.0.0
tags: [security, agent-security, mcp, supply-chain, approvals]
---

# HOL Guard

Use HOL Guard when a user wants local AI-agent security for a supported harness, needs to review Guard approvals/evidence, or wants to scan an agent package before installing or trusting it.

HOL Guard is an external runtime. This Factory plugin does not claim that Droid itself is a HOL Guard-protected harness and does not replace Factory's own permissions or security controls.

## Hard Rules

- Never read `.env` files or print secrets to diagnose Guard.
- Never bypass a Guard block or unresolved approval.
- Do not claim protection until `hol-guard status` or a harness-specific doctor command proves it.
- Treat `hol-guard command test` as inspection only, not an enforcement wrapper.
- Keep Guard Cloud optional. Local protection and scanning do not require a Cloud connection.
- Preserve user configuration; prefer Guard-owned setup commands over manual harness config edits.

## Install

Check the runtime and scanner separately with commands that work across supported shells:

```bash
hol-guard --version
plugin-scanner --version
```

If either command is unavailable, install only the CLI needed for the requested workflow.

For runtime protection:

```bash
pipx install hol-guard
hol-guard status
hol-guard detect --json
```

For package or skill scanning:

```bash
pipx install plugin-scanner
plugin-scanner lint .
plugin-scanner verify .
```

Do not assume the `hol-guard` package also provides the `plugin-scanner` command.

## Protect A Supported Harness

Treat `hol-guard detect --json` as the source of truth for the harnesses supported by the installed HOL Guard version. Do not maintain or infer a static harness list in this skill.

Use Guard-owned setup and verify before normal work:

```bash
hol-guard detect --json
hol-guard bootstrap
hol-guard install <harness>
hol-guard run <harness> --dry-run
hol-guard run <harness>
hol-guard doctor <harness> --json
hol-guard status
```

For Hermes, prefer its dedicated bootstrap path when applicable:

```bash
hol-guard hermes bootstrap
```

If the current Factory/Droid environment is not reported as a supported harness by `hol-guard detect --json`, do not invent a `droid` target. Use HOL Guard for package scanning, evidence review, or for a separately installed harness that Guard reports as supported.

## Review Approvals And Evidence

When Guard blocks or queues work:

```bash
hol-guard approvals
hol-guard approvals open
hol-guard receipts
hol-guard diff <harness>
```

Only approve after the user understands the risk reason and requested scope:

```bash
hol-guard approvals approve <request-id>
hol-guard approvals deny <request-id>
```

For audit/evidence work:

```bash
hol-guard receipts
hol-guard inventory
hol-guard abom --format json
hol-guard events
hol-guard explain <artifact-id>
```

## Scan Skills, Plugins, MCP Servers, Or Agent Packages

Run the scanner against the package root without executing the target just to scan it:

```bash
plugin-scanner lint <path>
plugin-scanner verify <path>
```

Use the repository root for mixed agent workspaces or marketplaces so local skills, plugins, MCP configuration, and harness artifacts can be discovered together.

If verification reports findings, surface the finding, affected path, and exact next safe action. Do not weaken scanner rules merely to make a package pass.

## Troubleshooting

```bash
hol-guard doctor
hol-guard detect --json
hol-guard settings show
plugin-scanner verify . --json
```

Report what command ran, what Guard found, what remains blocked or risky, and what proof exists. Do not claim approval, protection, or release readiness without command output proving it.

Canonical project: https://hol.org/guard
Source: https://github.com/hashgraph-online/hol-guard
