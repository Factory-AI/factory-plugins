# Local YDB Toolkit

Operate Docker-based local YDB deployments with reusable safety guidance and a pinned local stdio MCP server.

## Requirements

- Node.js 20.19 or newer
- npm
- Docker for local deployments, or SSH access to a remote Docker host

The MCP server starts from the installed plugin root. Pass an absolute `configPath` or set `LOCAL_YDB_TOOLKIT_CONFIG` when using a toolkit config file.

## Source

The manifest, MCP configuration, skill, and license are synchronized from [astandrik/local-ydb-toolkit](https://github.com/astandrik/local-ydb-toolkit) at the commit recorded in `SOURCE.json`.

Do not edit synchronized files in this directory directly. Regenerate them from a checkout containing the pinned commit:

```bash
python3 scripts/sync-local-ydb-toolkit.py --source /path/to/local-ydb-toolkit
```
