# Shipvela for Factory Droid

Publish supported websites with your own Shipvela account. Includes the remote OAuth MCP server and a publishing skill. No hooks, executable scripts, model provider keys or hosting credentials are included.

Add the repository as a marketplace with `droid plugin marketplace add stefanautomateed/shipvela-codex`. Run `droid plugin marketplace list` to obtain its registered name, then install `shipvela@<registered-name>`. Authenticate through `/mcp` and retain tool confirmations.

For a manual connection, use `droid mcp add shipvela https://shipvela.com/mcp --type http`. No static key is required. A public repository is direct distribution; it is not approval in Factory's official catalog.

Ask: "Use Shipvela to check my hosting allowance." Publishing requires target confirmation; small static uploads additionally require the owner to confirm the manifest in Shipvela. Only an exact successful deployment is a verified live URL.

Requires a Shipvela account. Connect GitHub in Shipvela for repository publishing. Supported static sites and supported Next.js SSR have different plan/runtime requirements; see [docs](https://shipvela.com/docs).

[Privacy](https://shipvela.com/privacy) · [Terms](https://shipvela.com/terms) · [Support](https://shipvela.com/support) · [Revoke access](https://shipvela.com/settings#coding-assistants). Support: hello@shipvela.com. Grants expire after 90 days unless revoked earlier.
