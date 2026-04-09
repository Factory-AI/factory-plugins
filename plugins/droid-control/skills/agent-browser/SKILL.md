---
name: agent-browser
description: Background knowledge for droid-control workflows -- not invoked directly. Agent-browser driver mechanics for web page and Electron desktop app automation.
user-invocable: false
---

# Agent-Browser Driver

The orchestrator routed you here. Use these mechanics to execute your plan.

Control web pages and Electron desktop apps via the `agent-browser` CLI. Uses Playwright under the hood with a headless Chromium instance managed by a background daemon.

## When to use

- Automating web app flows (login, form fill, data extraction, visual QA)
- Driving Electron apps (VS Code, Slack, Discord, Figma, Notion, Spotify)
- Visual verification -- screenshots and annotated element overlays
- DOM-level assertions where terminal snapshots are irrelevant

If the target is a terminal TUI, use **tuistory** or **true-input** instead.

## Prerequisites

```bash
agent-browser install   # one-time: downloads bundled Chromium
```

For Electron apps, the target app must be launched with `--remote-debugging-port=<port>`.

## Core workflow

Every interaction follows the same loop:

```bash
agent-browser open <url>
agent-browser snapshot -i          # interactive elements only -> refs like @e1, @e2
agent-browser click @e3            # interact using refs
agent-browser snapshot -i          # re-snapshot (refs invalidate after navigation/DOM changes)
agent-browser close                # always close when done
```

## Command chaining

Commands share a persistent daemon, so `&&` chaining is safe:

```bash
agent-browser open https://example.com && agent-browser wait --load networkidle && agent-browser snapshot -i
```

Chain when you don't need intermediate output. Run separately when you need to parse refs before acting.

## Command reference

### Navigation

| Command | Purpose |
|---|---|
| `open <url>` | Navigate (auto-prepends `https://` if no protocol) |
| `back` / `forward` / `reload` | History navigation |
| `close` | Shut down browser session |
| `connect <port>` | Attach to a running browser/Electron app via CDP |

### Snapshot (page analysis)

| Command | Purpose |
|---|---|
| `snapshot` | Full accessibility tree |
| `snapshot -i` | Interactive elements only (recommended default) |
| `snapshot -i -C` | Include cursor-interactive elements (onclick divs) |
| `snapshot -c` | Compact output |
| `snapshot -d <n>` | Limit tree depth |
| `snapshot -s "<selector>"` | Scope to CSS selector |

### Interactions (use @refs from snapshot)

| Command | Purpose |
|---|---|
| `click @e1` | Click (`dblclick` for double-click) |
| `fill @e2 "text"` | Clear field and type |
| `type @e2 "text"` | Type without clearing |
| `press Enter` | Press key (combos: `Control+a`) |
| `keyboard type "text"` | Type at current focus (no ref needed) |
| `keyboard inserttext "text"` | Insert without key events (Electron custom inputs) |
| `hover @e1` | Hover |
| `check @e1` / `uncheck @e1` | Toggle checkbox |
| `select @e1 "value"` | Select dropdown option |
| `scroll down 500` | Scroll page (`--selector` for containers) |
| `scrollintoview @e1` | Scroll element into view |
| `drag @e1 @e2` | Drag and drop |
| `upload @e1 file.pdf` | Upload file |

### Semantic locators (when refs are unreliable)

```bash
agent-browser find role button click --name "Submit"
agent-browser find text "Sign In" click
agent-browser find label "Email" fill "user@test.com"
agent-browser find testid "submit-btn" click
```

### Get information

| Command | Purpose |
|---|---|
| `get text @e1` | Element text (`get text body > page.txt` for full page) |
| `get html @e1` | innerHTML |
| `get value @e1` | Input value |
| `get attr @e1 href` | Element attribute |
| `get title` / `get url` | Page title / URL |
| `get count ".item"` | Count matching elements |

### Check state

```bash
agent-browser is visible @e1
agent-browser is enabled @e1
agent-browser is checked @e1
```

### Wait

| Command | Purpose |
|---|---|
| `wait @e1` | Wait for element |
| `wait 2000` | Wait milliseconds |
| `wait --text "Success"` | Wait for text |
| `wait --url "**/dashboard"` | Wait for URL pattern |
| `wait --load networkidle` | Wait for network idle (best for slow pages) |
| `wait --fn "window.ready"` | Wait for JS condition |

### JavaScript (eval)

```bash
agent-browser eval 'document.title'

# Complex JS -- use --stdin to avoid shell quoting issues
agent-browser eval --stdin <<'EVALEOF'
JSON.stringify(Array.from(document.querySelectorAll("a")).map(a => a.href))
EVALEOF
```

### Diff (compare page states)

```bash
agent-browser diff snapshot                          # current vs last snapshot
agent-browser diff snapshot --baseline before.txt    # current vs saved file
agent-browser diff screenshot --baseline before.png  # visual pixel diff
agent-browser diff url <url1> <url2>                 # compare two pages
```

### Dialogs

```bash
agent-browser dialog accept [text]  # accept alert/confirm/prompt
agent-browser dialog dismiss        # dismiss dialog
```

### Tabs & frames

```bash
agent-browser tab                 # list tabs
agent-browser tab new [url]       # new tab
agent-browser tab 2               # switch to tab by index
agent-browser tab close           # close current tab
agent-browser frame "#iframe"     # switch to iframe
agent-browser frame main          # back to main frame
```

## Screenshots & recording

```bash
agent-browser screenshot                      # save to temp directory
agent-browser screenshot path.png             # save to specific path
agent-browser screenshot --full               # full-page screenshot
agent-browser screenshot --annotate           # annotated with numbered element labels
agent-browser pdf output.pdf                  # save as PDF
```

`--annotate` overlays numbered labels on interactive elements. Each label `[N]` maps to ref `@eN`, enabling both visual verification and immediate interaction.

Video recording:

```bash
agent-browser record start ./demo.webm
# ... perform actions ...
agent-browser record stop
agent-browser record restart ./take2.webm     # stop current + start new
```

Recording creates a fresh context but preserves cookies/storage. Explore first, then start recording for smooth demos.

## Ref lifecycle

Refs (`@e1`, `@e2`, ...) are invalidated whenever the page changes. Always re-snapshot after:

- Clicking links/buttons that navigate
- Form submissions
- Dynamic content loading (dropdowns, modals)

```bash
agent-browser click @e5           # navigates
agent-browser snapshot -i         # MUST re-snapshot
agent-browser click @e1           # use new refs
```

## Electron app automation

Any Electron app supports `--remote-debugging-port` since it's built on Chromium.

### Launch and connect

```bash
# macOS
open -a "Slack" --args --remote-debugging-port=9222

# Linux
slack --remote-debugging-port=9222

# Then connect
sleep 3
agent-browser connect 9222
agent-browser snapshot -i
```

**The app must be quit first** if already running -- the flag only takes effect at launch.

### Tab management in Electron

Electron apps often have multiple windows/webviews:

```bash
agent-browser tab                        # list targets
agent-browser tab 2                      # switch by index
agent-browser tab --url "*settings*"     # switch by URL pattern
```

### Electron troubleshooting

| Problem | Fix |
|---|---|
| "Connection refused" | Ensure app was launched with `--remote-debugging-port`; quit and relaunch if already running |
| Connect fails after launch | `sleep 3` before connecting; app needs time to initialize |
| Elements missing from snapshot | Try `snapshot -i -C`; use `tab` to switch to the correct webview |
| Cannot type in fields | Use `keyboard type "text"` or `keyboard inserttext "text"` for custom input components |
| Dark mode lost | Set `AGENT_BROWSER_COLOR_SCHEME=dark` or use `--color-scheme dark` |

## State persistence

Save and restore cookies/localStorage across sessions:

```bash
agent-browser open https://app.example.com/login
# ... login flow ...
agent-browser state save auth.json

# Later: load saved state
agent-browser state load auth.json
agent-browser open https://app.example.com/dashboard
```

Auto-save/restore with named sessions:

```bash
agent-browser --session-name myapp open https://app.example.com
# state auto-saved on close, auto-loaded on next launch with same --session-name
```

## Sessions

The browser persists via a background daemon. One session is the default.

```bash
agent-browser --session test1 open site-a.com
agent-browser --session test2 open site-b.com
agent-browser session list
agent-browser --session test1 close
agent-browser --session test2 close
```

Each `--session` spawns a separate Chromium process (~300 MB). Prefer navigating within a single session. Exception: controlling multiple Electron apps on different CDP ports.

## Global options

| Flag | Purpose |
|---|---|
| `--session <name>` | Isolated browser session |
| `--headed` | Show browser window |
| `--cdp <port>` | Connect via CDP |
| `--auto-connect` | Auto-discover running Chrome |
| `--proxy <url>` | Use proxy server |
| `--color-scheme dark` | Force dark/light mode |
| `--ignore-https-errors` | Accept self-signed certs |
| `--allow-file-access` | Enable `file://` URLs |
| `--json` | JSON output for parsing |

## Debugging

```bash
agent-browser --headed open example.com   # visible browser
agent-browser console                     # view console messages
agent-browser errors                      # view page errors
agent-browser highlight @e1               # highlight element
```

## Gotchas

- **Invisible-to-snapshot elements.** `contenteditable` divs and custom components may not appear in accessibility snapshots. Use `eval` to interact:
  ```bash
  agent-browser eval --stdin <<'EVALEOF'
  const el = document.querySelector("[contenteditable]");
  el.focus();
  el.textContent = "hello";
  el.dispatchEvent(new Event('input', { bubbles: true }));
  EVALEOF
  ```
- **Unstable class names.** Never hardcode CSS-in-JS class names (`sc-*`, `css-*`). Find elements by text content, `cursor: pointer` style, or `testid` instead.
- **SPA loading delays.** Single-page apps may take 5-10s to render after navigation. Double-wait: `wait --load networkidle` then `wait 5000`.
- **Flag ordering.** Global flags (`--headers`, `--session`, `--cdp`) must come **before** the subcommand: `agent-browser --headers '{}' open <url>`.

## Critical rules

1. **Always take screenshots for visual QA.** Text snapshots miss layout, styling, alignment, and z-index issues. Use `screenshot --annotate` when you need both visual proof and element refs.
2. **One session by default.** Navigate between pages with `open <url>` instead of creating new sessions.
3. **Always close when done.** `agent-browser close` frees the Chromium process.
4. **Re-snapshot after every navigation.** Refs are invalidated.
