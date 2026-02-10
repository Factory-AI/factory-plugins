---
name: tuistory
description: |
  Playwright for terminal user interfaces.
---

# TUI Automation with tuistory

A Playwright-like testing framework for terminal user interfaces.

## Setup

Ensure tuistory is installed:
```bash
# Check if installed
which tuistory || (bun add -g tuistory || npm install -g tuistory)
```

## CLI Usage

tuistory provides a CLI for interacting with terminal sessions from the command line or shell scripts.

### Quick Start

```bash
# Launch Claude Code
tuistory launch "claude" -s claude --cols 100 --rows 30

# Wait for it to load
tuistory -s claude wait "Claude Code" --timeout 15000

# Type a prompt
tuistory -s claude type "what is 2+2? reply with just the number"
tuistory -s claude press enter

# Wait for Claude's response using regex (matches any digit)
tuistory -s claude wait "/[0-9]+/" --timeout 30000

# Get terminal snapshot
tuistory -s claude snapshot --trim
# Output:
# ╭─────────────────────────────────────────────────────────────────────────────────╮
# │ ● Claude Code                                                                   │
# ╰─────────────────────────────────────────────────────────────────────────────────╯
#
# > what is 2+2? reply with just the number
#
# 4
#
# ────────────────────────────────────────────────────────────────────────────────────
# > 

# Close the session
tuistory -s claude close
```

### Debugging Node.js with tuistory

```bash
# Launch Node.js debugger (assuming app.js has a debugger statement)
tuistory launch "node inspect app.js" -s debug --cols 80

# Wait for debugger to start and continue to breakpoint
tuistory -s debug wait "Break on start"
tuistory -s debug type "cont"
tuistory -s debug press enter
tuistory -s debug wait "break in"
tuistory -s debug snapshot --trim

# Print stack trace with bt (backtrace)
tuistory -s debug type "bt"
tuistory -s debug press enter
tuistory -s debug snapshot --trim
# Output:
# #0 getPrice app.js:13:2
# #1 calculateTotal app.js:7:15
# #2 processOrder app.js:2:16

# Enter REPL mode to inspect variables
tuistory -s debug type "repl"
tuistory -s debug press enter
tuistory -s debug type "item"
tuistory -s debug press enter
tuistory -s debug snapshot --trim
# Output:
# > item
# { name: 'Widget', price: 25, quantity: 2 }

# Clean up
tuistory -s debug close
```

### CLI Commands Reference

```bash
tuistory launch <command>     # Start a terminal session
tuistory snapshot             # Get terminal text content
tuistory type <text>          # Type text character by character
tuistory press <key> [keys]   # Press key(s): enter, ctrl c, alt f4
tuistory click <pattern>      # Click on text matching pattern
tuistory wait <pattern>       # Wait for text (supports /regex/)
tuistory wait-idle            # Wait for terminal to stabilize
tuistory scroll <up|down>     # Scroll the terminal
tuistory resize <cols> <rows> # Resize terminal
tuistory close                # Close a session
tuistory sessions             # List active sessions
tuistory daemon-stop          # Stop the background daemon
```

### Options

```bash
-s, --session <name>  # Session name (required for most commands)
--cols <n>            # Terminal columns (default: 80)
--rows <n>            # Terminal rows (default: 24)
--env <key=value>     # Environment variable (repeatable)
--timeout <ms>        # Wait timeout in milliseconds
--trim                # Trim whitespace from snapshot
--json                # Output as JSON
```

### Tips for Successful Automation

**Run `snapshot` after every command** - Terminal applications are stateful and may show dialogs, prompts, or errors. Always check the current state:

```bash
tuistory -s mysession press enter
tuistory -s mysession snapshot --trim  # See what happened
```

**Handle interactive dialogs** - Many CLI applications show first-run dialogs (trust prompts, terms acceptance, login screens). You need to navigate these before your automation can proceed:

```bash
# Example: Claude Code may show trust/terms dialogs on first run
tuistory launch "claude" -s claude
tuistory -s claude snapshot --trim          # Check current state
tuistory -s claude press enter              # Accept dialog
tuistory -s claude snapshot --trim          # Verify it worked
```

**Ensure applications are authenticated** - Some CLIs require login. Run authentication commands first:

```bash
tuistory -s claude type "/login"
tuistory -s claude press enter
tuistory -s claude snapshot --trim          # Follow login prompts
```

**Use `wait` for async operations** - Don't assume commands complete instantly:

```bash
tuistory -s mysession type "long-running-command"
tuistory -s mysession press enter
tuistory -s mysession wait "Done" --timeout 60000  # Wait for completion
tuistory -s mysession snapshot --trim
```

**Debug with frequent snapshots** - When automation fails, add snapshots between each step to see where it went wrong.
