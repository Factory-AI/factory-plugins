# linkbunny

Plan contextual nofollow link exchanges for sites you control with Droid. You review each placement and approve publication through your site's normal workflow. LinkBunny does not edit sites.

A LinkBunny account and API token are required. The first exchange on one site is free with no card. The paid plan covers up to 10 sites.

## Skills

### `linkbunny`

The LinkBunny API workflow: register a site, check in for an assignment, propose a contextual placement within the assignment's inventory, and report the published nofollow link. Use when you ask Droid to build backlinks or use LinkBunny.

## Install

```bash
droid plugin install linkbunny@factory-plugins
```

## Start without an account

Ask Droid:

```text
Use LinkBunny to plan setup for my site. I have no account or API token. Propose the target and inventory URLs first, and wait for my approval before registering the site or checking for an assignment.
```

The Skill plans locally before any LinkBunny API call. Save the API token in a private secret store you approve. Never put it in chat, plugin metadata, or a tracked file.

## Approvals

Account, billing, credential, site, check-in, publication, reporting, and rejection actions each need your approval. An assignment permits consideration within its inventory boundary; it does not authorize publication. Reporting needs separate approval after the published nofollow link has been inspected.

The Skill states these rules in text. This plugin contains only the Skill, its license, and this README: no hooks, droids, MCP server, commands, or automatic runs. Configure Droid's autonomy level and review its actions.

Read the [agent setup guide](https://getlinkbunny.com/docs/agents) for the API workflow and the optional MCP server. The Skill is distributed under MIT-0 from [hboon/linkbunny-agent-skill](https://github.com/hboon/linkbunny-agent-skill).
