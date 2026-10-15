# firecrawl

Official [Firecrawl](https://firecrawl.dev) plugin for Droid. It connects Firecrawl's hosted MCP server, which gives the agent the full Firecrawl toolset: web search, page scraping into clean Markdown or structured data, site mapping and crawling, browser interaction on dynamic pages, local document parsing, change monitoring, Alexandria data providers, and dedicated search over developer docs and repositories, US government sources, and scientific papers.

## Install

```sh
droid plugin install firecrawl@factory-plugins
```

Then connect your Firecrawl account: open `/mcp`, select the `firecrawl` server, and finish the browser sign-in. No API key goes in the config.

## Skills

- `firecrawl` - Choose the right Firecrawl operation for a request: search, scrape, map, crawl, interact, parse, monitor, Alexandria providers, and developer, government, and paper research
- `firecrawl-deep-research` - Investigate complex questions across many sources and return a cited answer, report, or table

## MCP server

`mcp.json` registers one HTTP server, `firecrawl`, at `https://mcp.firecrawl.dev/v2/mcp-oauth`. It uses OAuth: authenticate it from `/mcp`, and use `/mcp` again to clear or redo the sign-in.

## Resources

- [Firecrawl documentation](https://docs.firecrawl.dev)
- Support: hello@firecrawl.dev

## License

MIT. See [LICENSE](LICENSE).
