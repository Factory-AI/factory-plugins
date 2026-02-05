---
name: session-navigation
version: 1.1.0
description: |
  Navigate, search, and manage Droid sessions. Use when the user wants to:
  - List recent sessions
  - Search session history for specific topics or patterns
  - Resume a previous session
  - Get details about what was accomplished in a session
  - Find sessions by project, date, or content
---

# Session navigation

Find your way around past Droid sessions. Maybe you want to pick up where you left off, find that thing you did last week, or just see what's been happening in a project.

## Where sessions live

Sessions are in `~/.factory/sessions/`, organized by project folder. Each project gets its own directory with the path encoded (slashes become dashes):

```
~/.factory/sessions/
├── -Users-enoreyes-code-work-myapp/
│   ├── <uuid>.jsonl
│   └── <uuid>.settings.json
├── -Users-enoreyes-code-projects-api/
│   ├── <uuid>.jsonl
│   └── <uuid>.settings.json
└── ...
```

Two files per session:

**The conversation** (`.jsonl`): Each line is a JSON object. First line has metadata (session id, title, working directory). Rest is the back-and-forth: user messages, assistant responses, tool calls.

**The settings** (`.settings.json`): Stats about the session. Which model, how long it ran, token counts, autonomy mode.

## Finding sessions

### List project folders

```bash
# See all project folders with sessions
ls ~/.factory/sessions/

# Find folders for a specific project (partial match)
ls ~/.factory/sessions/ | grep "myapp"
```

### Recent sessions in a project

```bash
# List sessions by date for a project
ls -lt ~/.factory/sessions/-Users-enoreyes-code-work-myapp/

# Get titles of recent sessions
for f in $(ls -t ~/.factory/sessions/-Users-enoreyes-code-work-myapp/*.jsonl | head -10); do
  echo "=== $f ==="
  head -1 "$f" | jq -r '.title // "Untitled"'
done
```

### Search by content

```bash
# Search across ALL sessions
rg "authentication" ~/.factory/sessions/

# Search within a specific project
rg "bug fix" ~/.factory/sessions/-Users-enoreyes-code-work-myapp/

# See matches in context
rg -C 2 "login" ~/.factory/sessions/-Users-enoreyes-code-projects-api/
```

### Find which project has sessions about something

```bash
# Which projects have sessions mentioning "redis"?
rg -l "redis" ~/.factory/sessions/ | cut -d'/' -f1-5 | sort -u
```

## Reading a session

Once you've found a session file:

```bash
# The metadata (title, working directory)
head -1 ~/.factory/sessions/-Users-enoreyes-code-work-myapp/<uuid>.jsonl | jq .

# Session stats (model, tokens, duration)
cat ~/.factory/sessions/-Users-enoreyes-code-work-myapp/<uuid>.settings.json | jq .

# How long was this conversation?
wc -l ~/.factory/sessions/-Users-enoreyes-code-work-myapp/<uuid>.jsonl
```

User messages have `"role": "user"`, assistant responses have `"role": "assistant"`. Tool calls show what commands ran and what files got touched.

## Common situations

**"What did I work on in this project?"**
List that project's session folder, check dates, read through the conversation files.

**"Find that session where we fixed the login bug"**
Search for "login" or "auth" across sessions. Once you find it, read the conversation.

**"Resume what I was doing"**
Find the session, read through what happened, summarize the key decisions before continuing.

**"How much have I been using Droid?"**
The settings files have token counts and active time. Sum across sessions if needed.

## Tips

Use `rg` (ripgrep) instead of grep. It's faster and handles nested folders better.

Project paths have slashes replaced with dashes. `/Users/me/code/app` becomes `-Users-me-code-app`.

The session title isn't always helpful. Sometimes you need to read the conversation to know what it was about.

Sessions can contain sensitive stuff. Be careful about what you surface.
