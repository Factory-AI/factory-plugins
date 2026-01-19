---
name: session-navigation
version: 1.0.0
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

Everything's in `~/.factory/sessions/`. Two types of files per session:

**The conversation** (`.jsonl`): Each line is a JSON object. First line has metadata (session id, title, working directory). Rest is the back-and-forth: user messages, assistant responses, tool calls.

**The settings** (`.settings.json`): Stats about the session. Which model, how long it ran, token counts, autonomy mode.

Sessions are organized by project path. A session from `/Users/me/code/myapp` ends up in:

```
~/.factory/sessions/-Users-me-code-myapp/<uuid>.jsonl
```

Sessions without a specific project just go in the root sessions folder.

## Finding sessions

### Recent stuff

```bash
# What's been happening lately?
ls -lt ~/.factory/sessions/*.jsonl | head -20

# Get the titles
for f in $(ls -t ~/.factory/sessions/*.jsonl | head -10); do
  head -1 "$f" | jq -r '.title // "Untitled"'
done
```

### Search by content

```bash
# Find sessions that mention something
rg -l "authentication" ~/.factory/sessions/*.jsonl

# See the matches in context
rg -C 2 "authentication" ~/.factory/sessions/*.jsonl
```

### For a specific project

```bash
# Just that project's sessions
ls ~/.factory/sessions/-Users-me-code-myapp/

# Search within them
rg "bug fix" ~/.factory/sessions/-Users-me-code-myapp/*.jsonl
```

## Reading a session

Once you've found a session file:

```bash
# The metadata
head -1 ~/.factory/sessions/<uuid>.jsonl | jq .

# Session stats (model, tokens, duration)
cat ~/.factory/sessions/<uuid>.settings.json | jq .

# How long was this conversation?
wc -l ~/.factory/sessions/<uuid>.jsonl
```

To dig into the content, read the `.jsonl` file. User messages have `"role": "user"`, assistant responses have `"role": "assistant"`. Tool calls show what commands ran and what files got touched.

## Common situations

**"What did I do yesterday?"**
List recent sessions, check the timestamps, read the ones from yesterday.

**"Find that session where we fixed the login bug"**
Search for "login" or "auth" or whatever you remember about it. Once you find it, skim the conversation.

**"Resume what I was doing"**
Find the session, read through what happened, and pick up from there. You might want to summarize the key decisions or the state of things before continuing.

**"What's been going on in this project?"**
List that project's session folder, maybe grep for specific features or issues.

**"How much have I been using Droid?"**
The settings files have token counts and active time. You could sum those up across sessions if you wanted.

## Tips

Use `rg` (ripgrep) instead of grep. It's faster and handles the sessions folder better.

Sessions can contain sensitive stuff. Be careful about what you surface.

The session title in the first line of the jsonl isn't always helpful. Sometimes you need to actually read the conversation to know what it was about.

Project paths in folder names have slashes replaced with dashes. `/Users/me/code/app` becomes `-Users-me-code-app`. Takes a second to get used to reading them.
