# aeon

The operator-facing skill for [Aeon](https://github.com/aeonfun/aeon), an autonomous
agent framework that runs your own skills on a schedule in GitHub Actions. This plugin
ships that one skill so you can drive an Aeon instance straight from your coding agent.

## Skills

- `aeon` - Set up and run an Aeon agent instance: get started from scratch, turn skills
  on or off, schedule or reschedule what runs, edit what a skill does, debug a skill that
  will not fire, set the `STRATEGY.md` north star and `soul/` voice, turn a coding-agent
  chat into a scheduled skill, and mine past coding-agent conversations for recurring work
  worth automating.

The skill drives everything through the [GitHub CLI](https://cli.github.com/) (`gh`) and
the instance's own `./aeon` CLI, so nothing about it is tied to one coding agent beyond
where the skill file is loaded from.

## What it needs

- `gh` authenticated (`gh auth status`) - the skill routes every write through `gh`.
- An Aeon instance repo to operate on. Create one from the
  [template](https://github.com/aeonfun/aeon), then point the skill at it when it asks.

## License

MIT - see [LICENSE](./LICENSE).
