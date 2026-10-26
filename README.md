# Factory Plugins Marketplace

Official Factory plugins marketplace containing curated skills, droids, and tools.

## Installation

Add this marketplace to Factory:

```bash
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
```

Then install plugins:

```bash
droid plugin install security-engineer@factory-plugins
```

Or browse available plugins via the UI:

```
/plugins
```

## Available Plugins

### core

Core skills for essential functionalities and integrations. Pre-installed by the Droid CLI.

**Output styles:**

- `Explanatory` - Explains implementation choices and codebase patterns while completing the task

### droid-control

Terminal, browser, and computer automation for Droids. Record demos, verify behavior claims, and run QA flows.

**Commands:** `/demo`, `/verify`, `/qa-test`

**Skills:** `droid-control`, which routes to its atoms: `terminal-use`, `true-input`, `browser-use`, `desktop-use`, `droid-cli`, `pty-capture`, `capture`, `compose`, `verify`, `showcase`

See [plugins/droid-control/README.md](plugins/droid-control/README.md) for details.

### security-engineer

Security review, threat modeling, vulnerability validation, and patch generation skills.

**Skills:**

- `security-review` - STRIDE-based security analysis
- `threat-model-generation` - Generate threat models for repositories
- `commit-security-scan` - Scan commits/PRs for security vulnerabilities
- `vulnerability-validation` - Validate and confirm security findings

### typescript

Opinionated TypeScript and React patterns for safer, cleaner code.

**Skills:**

- `ban-type-assertions` - Ban `as` casts and replace them with compiler-verified alternatives (zod, control-flow narrowing)
- `no-use-effect` - Five replacement patterns for `useEffect` (derived state, query libraries, event handlers, `useMountEffect`, `key`)
- `fix-knip-unused-exports` - Fix every category of knip "Unused exports" violation

### debugging

Inspect runtime behavior: HTTP interception, traffic capture, and wire-level debugging for CLIs and services.

**Skills:**

- `http-toolkit-intercept` - Intercept and debug HTTP traffic from any CLI, service, or script via HTTP Toolkit (language/runtime agnostic)

### code-review

Pull request lifecycle skills: open, triage, and follow up on PRs with consistent conventions.

**Skills:**

- `create-pr` - Open a PR with Conventional Commits title, templated body, and local verification gates
- `follow-up-on-pr` - Rebase, address reviewer comments, fix CI, and push an existing PR to merge-ready state

### droid-evolved

Skills for continuous learning and improvement.

**Skills:**

- `human-writing` - Remove AI writing patterns, make text sound human
- `skill-creation` - Create and improve Droid skills
- `visual-design` - Image generation (nanobanana CLI) and presentations (Slidev)
- `frontend-design` - Build web apps, websites, HTML pages with good design
- `browser-navigation` - Browser automation with agent-browser

### autoresearch

Autonomous experiment loop for optimization research. Try an idea, measure it, keep what works, discard what doesn't, repeat. Works standalone or as a mission worker.

### apify

[Apify](https://apify.com) plugin, sourced from the `apify/` subdirectory of [apify/apify-factory-plugin](https://github.com/apify/apify-factory-plugin) at a pinned commit. It connects the hosted Apify MCP server (`https://mcp.apify.com/`, OAuth for authenticated tools) to search the Apify Store, run ready-made Actors, and read the Apify docs, and adds workflows for building and publishing your own Actors.

**Droid:** `apify` - Routes Apify requests to the right skill, MCP tool, or CLI path

**Commands:** `/apify`, `/create-actor`

**Skills:**

- `apify-actor-development` - Create, debug, and deploy a new Apify Actor
- `apify-actorization` - Convert an existing JS/TS, Python, or CLI project into an Actor
- `apify-generate-output-schema` - Generate dataset, output, and key-value store schemas for an existing Actor
- `apify-sdk-integration` - Add Actor execution to an existing application via `apify-client`
- `apify-ultimate-scraper` - CLI-driven data extraction with pre-built Actors across 15+ platforms

The CLI-based skills need the `apify` CLI (`npm install -g apify-cli`).

## Plugin Structure

Each plugin follows the Factory plugin format:

```
plugin-name/
├── .factory-plugin/
│   └── plugin.json       # Plugin metadata
├── skills/               # Skill definitions
│   └── skill-name/
│       └── SKILL.md
├── output-styles/        # Output style definitions (optional)
│   └── style-name.md
├── droids/               # Droid definitions (optional)
├── commands/             # Custom commands (optional)
├── mcp.json              # MCP server config (optional)
└── hooks.json            # Hook configurations (optional)
```

## Contributing

1. Fork this repository
2. Add your plugin under `plugins/`
3. Update the marketplace.json
4. Submit a pull request
