---
name: atomic-chat-setup
version: 1.0.0
description: |
  Configure Factory Droid to use Atomic Chat as a local LLM provider (OpenAI-compatible API).
  Use when the user wants to:
  - Run Droid with local models via Atomic Chat
  - Set up BYOK customModels for http://127.0.0.1:1337/v1
  - Troubleshoot Atomic Chat + Factory connectivity
  - Pick a model id from Atomic Chat's /v1/models endpoint
disable-model-invocation: false
---

# Atomic Chat + Factory setup

Atomic Chat exposes an OpenAI-compatible API (default `http://127.0.0.1:1337/v1`). Factory loads custom providers from `~/.factory/settings.json` → `customModels`.

## 1. Verify Atomic Chat is running

```bash
curl -s http://127.0.0.1:1337/v1/models | head -c 2000
```

If this fails:

1. Open Atomic Chat → **Settings → Local API Server** → enable (port `1337`)
2. Download at least one model in the app
3. Retry the curl command

## 2. Pick a model id

From the JSON response, use a model `id` string (e.g. from `data[].id`). If multiple models exist, prefer one the user already loaded; otherwise list options and ask.

## 3. Update Factory settings

Edit `~/.factory/settings.json`. Merge into `customModels` (do not remove existing entries unless the user asks):

```json
{
  "customModels": [
    {
      "model": "<MODEL_ID>",
      "displayName": "<MODEL_ID> [Atomic Chat]",
      "baseUrl": "http://127.0.0.1:1337/v1",
      "apiKey": "atomic-chat-local",
      "provider": "generic-chat-completion-api",
      "maxOutputTokens": 16000
    }
  ]
}
```

Notes:

- `apiKey` can be any non-empty string for default local installs; Atomic Chat is keyless by default
- Use `maxOutputTokens` appropriate for the model context window when known
- Preserve valid JSON; back up the file before editing if it already has content

## 4. Docker / remote Factory

If Droid runs in Docker and Atomic Chat on the host:

- macOS/Windows Docker Desktop: `http://host.docker.internal:1337/v1`
- Linux: host LAN IP instead of `127.0.0.1`

## 5. Verify end-to-end

```bash
curl -s http://127.0.0.1:1337/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer atomic-chat-local" \
  -d '{"model":"<MODEL_ID>","messages":[{"role":"user","content":"ping"}],"max_tokens":16}'
```

Tell the user to restart or start a new Droid session, then select the new **displayName** in the model picker.

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Connection refused on 1337 | Enable Local API Server in Atomic Chat |
| Model not in list | Download model in Atomic Chat first |
| Empty completions | Check model id matches `/v1/models` exactly |
| Factory can't reach host | Use `host.docker.internal` from containers |

## References

- Product: https://atomic.chat
- Source: https://github.com/AtomicBot-ai/Atomic-Chat
- Factory BYOK pattern matches [Ollama local setup](https://docs.factory.ai/cli/byok/ollama) with a different `baseUrl`
