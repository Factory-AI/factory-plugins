# rfspec

Request for Spec -- fan out a prompt to multiple AI models in parallel and choose or synthesize the best implementation spec.

## What it does

`/rfspec` sends your prompt to three models simultaneously (Claude Opus 4.6, GPT-5.4, Gemini 3.1 Pro), each generating a structured implementation spec. The results are presented side-by-side so you can pick the strongest one or synthesize a combination.

## Usage

```
/rfspec <describe what you want to build>
```

Example:

```
/rfspec add a dark mode toggle to the settings page with persistent user preference
```

The command will:

1. Send the prompt to all three models in parallel via `droid exec`
2. Collect and display each model's spec (Options A, B, C)
3. Ask you to pick one as-is or synthesize the best parts into a final spec
4. Save the chosen spec to `specs/active/YYYY-MM-DD-<slug>.md`

## Requirements

- **droid CLI** -- must be installed and authenticated
- **jq** -- for JSON parsing (`brew install jq` on macOS)
- Access to at least one of the three models. Models that fail are skipped gracefully; the command only errors if all three fail.

## Spec skeleton

Every generated spec follows a consistent structure:

- Purpose / Big Picture
- Context and Orientation
- Plan of Work (with named files and concrete steps)
- Progress checklist
- Validation and Acceptance criteria
- Key Decisions table
- Risks & Mitigations
- Surprises & Discoveries
- Outcomes & Retrospective
