# rfspec

Request for Spec -- fan out a prompt to multiple AI models in parallel and choose or synthesize the best result.

## What it does

`/rfspec` sends your prompt to three models simultaneously (Claude Opus 4.6, GPT-5.4, Gemini 3.1 Pro), each at its maximum reasoning tier. The results are presented side-by-side so you can pick the strongest one or synthesize a combination. No prescriptive system prompt is injected -- the models bring their own reasoning to your request.

## Usage

```
/rfspec <describe what you want to build>
```

Example:

```
/rfspec add a dark mode toggle to the settings page with persistent user preference
```

The command will:

1. Send the prompt to all three models in parallel via `droid exec`, each at its maximum reasoning tier (Opus: max, GPT-5.4: xhigh, Gemini: high)
2. Collect and display each model's response (Options A, B, C)
3. Ask you to pick one as-is or synthesize the best parts
4. Save the chosen result to `specs/active/YYYY-MM-DD-<slug>.md`

## Requirements

- **droid CLI** -- must be installed and authenticated
- **jq** -- for JSON parsing (`brew install jq` on macOS)
- Access to at least one of the three models. Models that fail are skipped gracefully; the command only errors if all three fail.
