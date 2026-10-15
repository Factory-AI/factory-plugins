# firecrawl

Official [Firecrawl](https://firecrawl.dev) plugin for Droid. It connects Firecrawl's hosted MCP server for web search, page reading, structured data from data providers, developer docs and repository search, US government sources, and scientific papers.

## Install

```sh
droid plugin install firecrawl@factory-plugins
```

Then connect your Firecrawl account: open `/mcp`, select the `firecrawl` server, and finish the browser sign-in. No API key goes in the config.

## Skills

- `firecrawl` - Search the web, read a supplied URL, and retrieve structured records from data providers
- `firecrawl-developer-index` - Find documentation, issues, and pull requests for library behavior, API contracts, and error messages
- `firecrawl-research-index` - Find scientific papers, follow citations, and verify claims against paper passages

## MCP server

`mcp.json` registers one HTTP server, `firecrawl`, at `https://mcp.firecrawl.dev/v2/mcp-search`. It uses OAuth: authenticate it from `/mcp`, and use `/mcp` again to clear or redo the sign-in.

## Resources

- [Firecrawl documentation](https://docs.firecrawl.dev)
- [Report a problem](https://github.com/firecrawl/firecrawl-claude-plugin/issues)

## License

MIT. See [LICENSE](LICENSE).
