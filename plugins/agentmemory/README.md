# AgentMemory

AgentMemory saves useful project context and retrieves it in later coding sessions. The plugin includes 17 skills, lifecycle hooks, and MCP tools for saving, searching, and managing memory.

## Setup

Requires Node.js 20 or newer, npm, Git, and a running AgentMemory server. Start the server in a separate terminal:

```bash
npx -y @agentmemory/agentmemory@0.9.31
```

Then install the plugin:

```bash
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
droid plugin install agentmemory@factory-plugins
```

Start a new Droid session. Ask it to save a project preference, then retrieve that preference in another session using the memory tools.

The default server address is `http://localhost:3111`. Set `AGENTMEMORY_URL` before starting Droid to use another server. Loopback connections discover the local daemon credential automatically. For a remote server, use HTTPS and supply `AGENTMEMORY_SECRET` through the environment. Never put credentials in the plugin manifest.

## What is installed

This catalog references the published AgentMemory 0.9.31 plugin at a fixed Git commit. Droid loads its skills, hooks, and MCP configuration through Claude Code plugin compatibility. The prebuilt scripts are included upstream, so installation requires no build step.

The plugin does not start the memory server. With the server running, the MCP bridge exposes the complete tool set. If the server is unavailable, memory tool calls report that failure; this bridge does not use a fallback store.

## Data and support

Hooks send prompts and tool activity to the configured AgentMemory server. Local capture queues may retain events while that server is unavailable. Use the memory governance tools to inspect or delete saved data. Review [AgentMemory's documentation](https://github.com/rohitg00/agentmemory#readme) before enabling capture.

[Source and license](https://github.com/rohitg00/agentmemory) · [Support](https://github.com/rohitg00/agentmemory/issues)
