# Contributing

## What's open to contributions

| Plugin | Who can contribute | Process |
|--------|--------------------|---------|
| `core` | Factory employees only | Internal review |
| Everything else | Anyone | Open an issue first, then PR |

## Before you start

**Open an issue before writing code.** Describe what you want to change and why. This saves everyone time if the direction needs adjustment or the work is already planned.

Small fixes (typos, broken links, clarifying a sentence) can go straight to a PR.

## Standards

**You must be willing to stand behind every line you push.** If you used AI tools to help write code, that's fine, but you are responsible for reviewing and understanding it. PRs with unreviewed generated output will be closed.

**No promotions.** Plugins exist to give Droids useful capabilities, not to advertise products or services. Plugins that primarily serve to promote external software will be rejected.

**Match existing conventions.** Read the plugin you're modifying before changing it. Follow the same file structure, naming, and style. When in doubt, look at how existing plugins handle it.

## Submitting a PR

1. Fork the repo and create a branch from `master`.
2. Make your changes. Keep the diff focused on one thing.
3. Test your changes locally by installing the plugin via the marketplace.
4. Open a PR with a clear description of what changed and why.

## Plugin structure

Every plugin must include a `.factory-plugin/plugin.json` manifest. See the [Factory plugin docs](https://docs.factory.ai/cli/configuration/plugins) for the full spec.

```
my-plugin/
├── .factory-plugin/
│   └── plugin.json
├── skills/
│   └── my-skill/
│       └── SKILL.md
├── commands/
│   └── my-command.md
└── README.md
```

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
