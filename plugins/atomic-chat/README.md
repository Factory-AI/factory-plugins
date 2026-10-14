# atomic-chat

Connect [Atomic Chat](https://atomic.chat) to Factory Droid as a **local OpenAI-compatible** model provider.

Atomic Chat runs open-weight models on your machine with a local API at `http://127.0.0.1:1337/v1`. Optional Exa web search runs inside the Atomic Chat app (not this plugin).

## Install

```bash
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
droid plugin install atomic-chat@factory-plugins --scope user
```

Or browse via `/plugins` → **atomic-chat**.

## Quick setup

Run in a Droid session:

```text
/atomic-chat-setup
```

Or ask Droid to load the **atomic-chat-setup** skill and configure `~/.factory/settings.json`.

## Manual configuration

Add to `~/.factory/settings.json`:

```json
{
  "customModels": [
    {
      "model": "YOUR_MODEL_ID",
      "displayName": "Local model [Atomic Chat]",
      "baseUrl": "http://127.0.0.1:1337/v1",
      "apiKey": "atomic-chat-local",
      "provider": "generic-chat-completion-api",
      "maxOutputTokens": 16000
    }
  ]
}
```

Replace `YOUR_MODEL_ID` with an id from `curl http://127.0.0.1:1337/v1/models`.

## Prerequisites

1. Install [Atomic Chat](https://atomic.chat) and download at least one model
2. Enable **Settings → Local API Server** (default port `1337`)
3. Verify: `curl http://127.0.0.1:1337/v1/models`

## Links

- [Atomic Chat](https://atomic.chat)
- [GitHub](https://github.com/AtomicBot-ai/Atomic-Chat)
