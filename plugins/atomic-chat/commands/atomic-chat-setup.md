---
description: Configure Factory to use Atomic Chat local models (OpenAI-compatible API at :1337)
argument-hint: '[optional model id from Atomic Chat]'
---

Load skill: **atomic-chat-setup**.

## Goal

Wire Factory `customModels` to the user's Atomic Chat Local API Server.

## Steps

1. If `$ARGUMENTS` contains a model id, use it. Otherwise call `GET http://127.0.0.1:1337/v1/models` and pick the best match (or ask the user).
2. Follow **atomic-chat-setup** to update `~/.factory/settings.json` without clobbering unrelated `customModels` entries.
3. Run the chat completions smoke test from the skill.
4. Report: model id, base URL, display name, and how to select it in Droid.

If Atomic Chat is not running, give install/enable steps from the skill and stop — do not invent API keys or model names.
