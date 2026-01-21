---
name: skill-creation
version: 1.0.0
description: |
  Create, improve, and manage Droid skills. Use when the user wants to:
  - Create new skills from scratch or from session learnings
  - Improve existing skills based on user preferences
  - Analyze sessions to identify patterns worth codifying
  - Understand best practices for agentic skill design
  This is a meta-skill for self-improvement and continuous learning.
---

# Skill creation

This skill helps you build new skills and improve existing ones. Think of it as the skill that teaches you how to learn.

## Why bother with skills?

Every time you solve a problem, that knowledge usually dies with the session. Skills fix that. They're how you turn "I figured this out once" into "I know how to do this."

A few reasons to extract skills:

- You won't have to re-discover the same solution next month
- Other sessions (and other users) benefit from what you learned
- Complex workflows become repeatable instead of fragile

## The basic format

Skills live in a folder with a `SKILL.md` file:

```
.factory/skills/my-skill/
└── SKILL.md
```

The file has YAML frontmatter and markdown content:

```markdown
---
name: my-skill
version: 1.0.0
description: |
  What this skill does.
  When to use it.
---

# My skill

Instructions go here.
```

The `description` matters a lot. It's how the agent decides whether to load your skill for a given task. Be specific about the problems it solves.

## When to create a skill

Not everything deserves to be a skill. Skills are for complex or long workflows that someone might need to repeat or share. If it's a simple one-off task, a skill is overkill.

The right level of specificity matters. A skill for "debugging in a codebase" is useful if there's a lot of common failure modes that agents might encounter. A skill for "debugging the login flow" is probably too narrow. Find the balance between general enough to reuse and specific enough to be helpful.

Ask yourself:

- Did I have to dig around to figure this out?
- Would I be annoyed if I had to solve this again from scratch?
- Is there something here that isn't obvious from the docs?

If yes to any of those, probably worth extracting. If it was straightforward or you just followed a tutorial, skip it.

Skills can also encode preferences and best practices. Maybe a user always logs into a specific platform when doing data analysis. Maybe there's a gotcha the team keeps hitting. If you notice you're consistently doing something the user has to correct or adjust, that's worth including.

## Extracting skills from sessions

Use the `session-navigation` skill to dig through past sessions and find patterns worth extracting. Look for things that came up multiple times, solutions that took real effort to figure out, or workflows you keep repeating.

Once you've found something, generalize it. Replace specific paths with patterns, note the prerequisites, call out assumptions. The skill should work for similar situations, not just the exact case you found.

## Skill design tips

When writing the skill, use the `human-writing` skill. Skill docs that read like marketing copy are harder to follow.

**Start small.** A skill that does one thing well beats a skill that tries to cover everything. You can always compose multiple skills together.

**Include verification.** How do you know the skill worked? Add a check at the end:

```markdown
## Verify it worked

Run `npm test` and make sure nothing broke.
Check that the new file exists at `src/config.ts`.
```

**Document the failures too.** What doesn't work? What should you avoid? This saves future pain:

```markdown
## What not to do

Don't run this on a dirty git working directory.
The `--force` flag will overwrite without asking.
```

**Keep it fresh.** Skills rot. Dependencies change, APIs update, better approaches emerge. If a skill stops working or feels outdated, update it or delete it.

## Where skills live

| Location             | Who sees it                                |
| -------------------- | ------------------------------------------ |
| `.factory/skills/`   | Everyone on the project (commit it to git) |
| `~/.factory/skills/` | Just you, across all projects              |

Project skills are good for team conventions. Personal skills are good for your own workflows.

## Improving existing skills

Signs a skill needs work:

- Users ask follow-up questions after it runs
- It fails on edge cases that keep coming up
- There's a better approach now than when it was written

To find patterns:

```bash
# Which sessions used this skill?
rg -l "skill-name" ~/.factory/sessions/*.jsonl

# Where did things go wrong?
rg "error|failed|retry" ~/.factory/sessions/*.jsonl -C 3
```

When you update a skill, bump the version. If it's a breaking change (different output, different inputs), bump the major version.

## Research notes

Some of this comes from academic work on agents that learn:

**Voyager** showed that agents can build up skill libraries over time, with each skill composed from simpler ones.

**CASCADE** demonstrated that skills can be shared between agents, not just stored for one agent's use.

**SEAgent** found that learning from failures is as valuable as learning from successes.

**Reflexion** showed that verbal feedback (explaining what went wrong in plain language) beats numeric scores for improving agent behavior.

## The loop

1. Work on something
2. Notice when you learn something non-obvious
3. Extract it as a skill
4. Use the skill next time
5. Improve the skill based on how it goes
6. Repeat
