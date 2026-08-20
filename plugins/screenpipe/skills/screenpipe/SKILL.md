---
name: screenpipe
description: Use screenpipe's private local work history to recall prior activity, find decisions and meetings, reconstruct repeated workflows, draft evidence-backed SOPs, and identify automation candidates.
---

# screenpipe workflow memory

Use the screenpipe MCP tools as the evidence layer for questions about the
user's real work. Tool names may be prefixed by Droid; match them by the base
names below.

## Choose the smallest useful query

1. Use `activity-summary` for broad questions about a period, app usage, or what the user was doing.
2. Use `search-content` for specific text, conversations, people, projects, decisions, or audio. Filter by time and app when possible.
3. Use `keyword-search` for fast exact-text lookup.
4. Use `frame-context` or `get-frame-elements` only after a search result identifies a relevant frame.
5. Use `list-meetings` and `get-meeting` for meeting history.

Run `health-check` when screenpipe appears unavailable. Explain the failure
clearly instead of inventing results.

## Runtime and permissions

This plugin launches `screenpipe-mcp@0.19.1` over stdio. The server advertises
retrieval tools and mutation tools for memories, meetings, recording, speakers,
notifications, and scheduled pipes. Treat retrieval as the default; use a
mutation tool only when the user explicitly requests that change.

The MCP authenticates only to the user's local screenpipe API and discovers its
key from the installed app. This plugin sets `SCREENPIPE_DISABLE_TELEMETRY=1`,
disabling the MCP package's outbound usage and error telemetry. Retrieved
content still enters the current Droid model context, so disclose that boundary
when the configured model is remote.

## Answer from evidence

- Preserve timestamps, app names, window titles, speakers, and source identifiers returned by tools.
- Distinguish direct evidence from inference.
- Cite the moments that support conclusions. Never fabricate a frame ID, timestamp, quote, or deep link.
- Prefer one well-filtered query over dumping long unfiltered history into context.
- Treat absence of results as absence of evidence, not proof that something did not happen.

## Reconstruct a workflow

1. Define the task, actor, time range, and intended output.
2. Retrieve a broad activity summary for the period.
3. Search the relevant apps, windows, meetings, and terms.
4. Order the evidence into trigger, inputs, actions, decisions, handoffs, and output.
5. Mark gaps and uncertainty explicitly.
6. Return a concise workflow report with cited moments and repeated patterns.

## Draft an SOP or automation candidate

- Build the SOP from repeated observed runs, not a single anecdote.
- Include prerequisites, ordered steps, decision points, exceptions, validation, and completion evidence.
- Separate stable steps from case-specific details.
- Identify automation candidates only after the workflow is evidenced.
- Recommend a human review point before any irreversible or externally visible action.

## Privacy and mutations

Screenpipe data can contain private messages, customer information,
credentials, and other sensitive material.

- Do not share, upload, or quote sensitive history outside the current task without explicit user approval.
- Do not export video, change recording, edit memories, modify meetings or speakers, run pipes, or send notifications unless the user requested that action.
- Keep retrieval local when possible and disclose when a remote model will receive retrieved context.
