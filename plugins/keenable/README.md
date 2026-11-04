# keenable

Official [Keenable](https://keenable.ai) plugin for Droid. It connects Keenable's hosted MCP server for fast web search and clean Markdown page extraction — built for agents that need current, source-grounded information.

## Install

```sh
droid plugin install keenable@factory-plugins
```

The MCP server works **keyless** as soon as the plugin is installed: no OAuth flow, no signup, and no API key in `mcp.json`. Search and page fetch are available immediately through the `keenable` server.

For higher rate limits, add a Keenable API key in Droid MCP settings (`/mcp` → `keenable` → configure credentials). Keys are optional and never belong in this repo or in chat.

## Skills

- `keenable-web` — Search the web and read known URLs with `search_web_pages` and `fetch_page_content`
- `keenable-deep-research` — Multi-source investigations with cited synthesis for reviews, comparisons, and market analysis

## MCP server

`mcp.json` registers one HTTP server, `keenable`, at `https://api.keenable.ai/mcp`. No OAuth and no API key in the config file.

## Resources

- [Keenable MCP documentation](https://docs.keenable.ai/mcp-server)
- [Keenable](https://keenable.ai)

## License

MIT. See [LICENSE](LICENSE).
