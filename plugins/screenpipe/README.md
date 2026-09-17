# screenpipe for Factory Droid

Use screenpipe as a local-first evidence layer for questions about real work.
The plugin combines the pinned screenpipe MCP server with a skill for searching
private work history, reconstructing repeated tasks, drafting cited SOPs, and
identifying automation candidates.

## Requirements

- screenpipe installed and running locally
- Node.js 18 or newer

The MCP server discovers the local screenpipe API key from the installed app.
This plugin disables the MCP package's outbound usage and error telemetry.

Screen and audio history remains in the user's screenpipe instance. Evidence
retrieved for a task enters the configured Droid model context. The skill treats
retrieval as the default and requires explicit user intent before mutations.

## Install

```bash
droid plugin install screenpipe@factory-plugins --scope user
```

Plugin wrapper files are MIT licensed. The screenpipe application and
`screenpipe-mcp` package retain their respective upstream licenses.
