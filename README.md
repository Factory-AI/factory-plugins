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

**Skills:**

- `review` - Review code changes and identify high-confidence, actionable bugs. Includes systematic analysis patterns for null safety, async/await, security, concurrency, API contracts, and more. Used by both the CLI `/review` command and the CI action.
- `session-navigation` - Search and navigate past Droid sessions
- `init` - Session initialization

### security-engineer

Security review, threat modeling, and vulnerability validation skills.

**Skills:**

- `security-review` - STRIDE-based security analysis
- `threat-model-generation` - Generate threat models for repositories
- `commit-security-scan` - Scan commits/PRs for security vulnerabilities
- `vulnerability-validation` - Validate and confirm security findings

### droid-evolved

Skills for continuous learning and improvement.

**Skills:**

- `session-navigation` - Search and navigate past Droid sessions
- `human-writing` - Remove AI writing patterns, make text sound human
- `skill-creation` - Create and improve Droid skills
- `visual-design` - Image generation (nanobanana CLI) and presentations (Slidev)
- `frontend-design` - Build web apps, websites, HTML pages with good design
- `browser-navigation` - Browser automation with agent-browser

## Plugin Structure

Each plugin follows the Factory plugin format:

```
plugin-name/
├── .factory-plugin/
│   └── plugin.json       # Plugin metadata
├── skills/               # Skill definitions
│   └── skill-name/
│       └── SKILL.md
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
