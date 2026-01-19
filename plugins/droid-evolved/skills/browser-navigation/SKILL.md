---
name: browser-navigation
version: 1.0.0
description: |
  Automate browser interactions for web testing, form filling, screenshots, and data extraction.
  Use when the user needs to:
  - Navigate websites and interact with web pages
  - Fill forms and click buttons
  - Take screenshots of web content
  - Test web applications
  - Extract information from web pages
  - Debug frontend issues in the browser
  - Monitor console logs and network requests
  - Record browser sessions as video
  This skill uses agent-browser for comprehensive browser automation.
  RECOMMENDATION: Disable Chrome DevTools or Playwright MCP when using this skill to save context.
---

# Browser Automation with agent-browser

Comprehensive browser automation for testing, data extraction, and web interaction.

## Quick Start

```bash
agent-browser open <url>        # Navigate to page
agent-browser snapshot -i       # Get interactive elements with refs
agent-browser click @e1         # Click element by ref
agent-browser fill @e2 "text"   # Fill input by ref
agent-browser close             # Close browser
```

## Core Workflow

1. **Navigate**: `agent-browser open <url>`
2. **Snapshot**: `agent-browser snapshot -i` (returns elements with refs like `@e1`, `@e2`)
3. **Interact** using refs from the snapshot
4. **Re-snapshot** after navigation or significant DOM changes

## Installation

```bash
# Install globally
npm install -g agent-browser

# Or use via npx
npx agent-browser open https://example.com
```

## Commands Reference

### Navigation

```bash
agent-browser open <url>      # Navigate to URL
agent-browser back            # Go back
agent-browser forward         # Go forward
agent-browser reload          # Reload page
agent-browser close           # Close browser
```

### Page Analysis (Snapshot)

```bash
agent-browser snapshot            # Full accessibility tree
agent-browser snapshot -i         # Interactive elements only (RECOMMENDED)
agent-browser snapshot -c         # Compact output
agent-browser snapshot -d 3       # Limit depth to 3
agent-browser snapshot -s "#main" # Scope to CSS selector
```

### Interactions (Use @refs from Snapshot)

```bash
# Clicking
agent-browser click @e1           # Click
agent-browser dblclick @e1        # Double-click
agent-browser hover @e1           # Hover

# Focus
agent-browser focus @e1           # Focus element

# Text Input
agent-browser fill @e2 "text"     # Clear and type
agent-browser type @e2 "text"     # Type without clearing

# Keyboard
agent-browser press Enter         # Press key
agent-browser press Control+a     # Key combination
agent-browser keydown Shift       # Hold key down
agent-browser keyup Shift         # Release key

# Forms
agent-browser check @e1           # Check checkbox
agent-browser uncheck @e1         # Uncheck checkbox
agent-browser select @e1 "value"  # Select dropdown option

# Scrolling
agent-browser scroll down 500     # Scroll page
agent-browser scrollintoview @e1  # Scroll element into view

# Other
agent-browser drag @e1 @e2        # Drag and drop
agent-browser upload @e1 file.pdf # Upload files
```

### Getting Information

```bash
agent-browser get text @e1        # Get element text
agent-browser get html @e1        # Get innerHTML
agent-browser get value @e1       # Get input value
agent-browser get attr @e1 href   # Get attribute
agent-browser get title           # Get page title
agent-browser get url             # Get current URL
agent-browser get count ".item"   # Count matching elements
agent-browser get box @e1         # Get bounding box
```

### Checking State

```bash
agent-browser is visible @e1      # Check if visible
agent-browser is enabled @e1      # Check if enabled
agent-browser is checked @e1      # Check if checked
```

### Screenshots & PDF

```bash
agent-browser screenshot          # Screenshot to stdout
agent-browser screenshot path.png # Save to file
agent-browser screenshot --full   # Full page
agent-browser pdf output.pdf      # Save as PDF
```

### Video Recording

```bash
agent-browser record start ./demo.webm    # Start recording
agent-browser click @e1                   # Perform actions
agent-browser record stop                 # Stop and save video
agent-browser record restart ./take2.webm # Stop current + start new
```

### Waiting

```bash
agent-browser wait @e1                     # Wait for element
agent-browser wait 2000                    # Wait milliseconds
agent-browser wait --text "Success"        # Wait for text
agent-browser wait --url "**/dashboard"    # Wait for URL pattern
agent-browser wait --load networkidle      # Wait for network idle
agent-browser wait --fn "window.ready"     # Wait for JS condition
```

### Cookies & Storage

```bash
agent-browser cookies                     # Get all cookies
agent-browser cookies set name value      # Set cookie
agent-browser cookies clear               # Clear cookies
agent-browser storage local               # Get all localStorage
agent-browser storage local key           # Get specific key
agent-browser storage local set k v       # Set value
agent-browser storage local clear         # Clear all
```

### Network

```bash
agent-browser network route <url>              # Intercept requests
agent-browser network route <url> --abort      # Block requests
agent-browser network route <url> --body '{}'  # Mock response
agent-browser network unroute [url]            # Remove routes
agent-browser network requests                 # View tracked requests
agent-browser network requests --filter api    # Filter requests
```

### Browser Settings

```bash
agent-browser set viewport 1920 1080      # Set viewport size
agent-browser set device "iPhone 14"      # Emulate device
agent-browser set geo 37.7749 -122.4194   # Set geolocation
agent-browser set offline on              # Toggle offline mode
agent-browser set headers '{"X-Key":"v"}' # Extra HTTP headers
agent-browser set credentials user pass   # HTTP basic auth
agent-browser set media dark              # Emulate color scheme
```

### Tabs & Windows

```bash
agent-browser tab                 # List tabs
agent-browser tab new [url]       # New tab
agent-browser tab 2               # Switch to tab
agent-browser tab close           # Close tab
agent-browser window new          # New window
```

### Frames

```bash
agent-browser frame "#iframe"     # Switch to iframe
agent-browser frame main          # Back to main frame
```

### Dialogs

```bash
agent-browser dialog accept [text]  # Accept dialog
agent-browser dialog dismiss        # Dismiss dialog
```

### JavaScript Execution

```bash
agent-browser eval "document.title"   # Run JavaScript
```

## Example Workflows

### Form Submission

```bash
agent-browser open https://example.com/form
agent-browser snapshot -i
# Output shows: textbox "Email" [ref=e1], textbox "Password" [ref=e2], button "Submit" [ref=e3]

agent-browser fill @e1 "user@example.com"
agent-browser fill @e2 "password123"
agent-browser click @e3
agent-browser wait --load networkidle
agent-browser snapshot -i  # Check result
```

### Authentication with Saved State

```bash
# Login once
agent-browser open https://app.example.com/login
agent-browser snapshot -i
agent-browser fill @e1 "username"
agent-browser fill @e2 "password"
agent-browser click @e3
agent-browser wait --url "**/dashboard"
agent-browser state save auth.json

# Later sessions: load saved state
agent-browser state load auth.json
agent-browser open https://app.example.com/dashboard
```

### Scraping Data

```bash
agent-browser open https://example.com/products
agent-browser snapshot -i --json > page_structure.json

# Get specific data
agent-browser get text @e5  # Product title
agent-browser get attr @e6 href  # Product link
```

### Taking Full Page Screenshot

```bash
agent-browser open https://example.com
agent-browser wait --load networkidle
agent-browser screenshot --full fullpage.png
```

### Testing Login Flow

```bash
# Navigate to login
agent-browser open https://app.example.com/login

# Take initial snapshot
agent-browser snapshot -i

# Fill credentials
agent-browser fill @e1 "test@example.com"
agent-browser fill @e2 "testpassword"

# Click login
agent-browser click @e3

# Wait for redirect
agent-browser wait --url "**/dashboard"

# Verify logged in
agent-browser get text @e10  # Should show username
```

## Debugging

```bash
agent-browser open example.com --headed              # Show browser window
agent-browser console                                # View console messages
agent-browser console --clear                        # Clear console
agent-browser errors                                 # View page errors
agent-browser errors --clear                         # Clear errors
agent-browser highlight @e1                          # Highlight element
agent-browser trace start                            # Start recording trace
agent-browser trace stop trace.zip                   # Stop and save trace
agent-browser --cdp 9222 snapshot                    # Connect via CDP
```

## Sessions (Parallel Browsers)

```bash
agent-browser --session test1 open site-a.com
agent-browser --session test2 open site-b.com
agent-browser session list
```

## JSON Output

Add `--json` for machine-readable output:

```bash
agent-browser snapshot -i --json
agent-browser get text @e1 --json
```

## Semantic Locators (Alternative to Refs)

```bash
agent-browser find role button click --name "Submit"
agent-browser find text "Sign In" click
agent-browser find label "Email" fill "user@test.com"
agent-browser find first ".item" click
agent-browser find nth 2 "a" text
```

## Best Practices

1. **Always snapshot first**: Get the current page state before interacting
2. **Use interactive mode (-i)**: Shows only clickable/fillable elements
3. **Wait appropriately**: Use `--load networkidle` after navigation
4. **Re-snapshot after changes**: DOM updates invalidate refs
5. **Save authentication state**: Avoid repeated logins
6. **Use --headed for debugging**: See what the browser sees
7. **Check console for errors**: `agent-browser console` reveals issues

## When to Use This Skill

- Testing web applications end-to-end
- Automating repetitive web tasks
- Scraping data from websites
- Debugging frontend issues
- Taking screenshots for documentation
- Recording demo videos
- Verifying UI functionality
- Filling out forms programmatically

## Disable conflicting MCPs

If Chrome DevTools or Playwright MCP is enabled, ask the user if they want to disable it to save context. This skill covers the same functionality.

Ask them to run:
```
/mcp
```

If they see chrome-devtools or playwright listed, suggest they remove it:
```
/mcp remove chrome-devtools
/mcp remove playwright
```

Don't run these commands yourself. Let the user decide whether to disable the MCP.
