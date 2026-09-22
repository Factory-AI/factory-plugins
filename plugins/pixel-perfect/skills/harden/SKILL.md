---
name: harden
description: "Test interface resilience by actually exercising edge cases in a real browser — extreme inputs, offline mode, slow networks, form validation, keyboard navigation. Use when the user wants to harden, stress-test, handle edge cases, or make a UI production-ready."
argument-hint: "<url-or-file>"
user-invocable: true
---

# Harden — Browser-Driven Edge Case Testing

Open the page in a real browser and throw every edge case at it.

## Step 1: Baseline

```bash
agent-browser open <url>
agent-browser wait --load networkidle
agent-browser snapshot -i
agent-browser screenshot --full baseline.png
```

## Step 2: Test Extreme Text Inputs

Find all text inputs (`agent-browser snapshot -i`) and fill with:

```bash
# Very long text
agent-browser fill @e1 "This is an extremely long input that simulates a user pasting an entire paragraph into a field designed for short names and will likely overflow or break the layout in ways nobody tested"
agent-browser screenshot long-text.png

# Emoji and special characters
agent-browser fill @e1 "🎉🔥💯 Ünïcödé → «special» — "quotes" & <tags>"
agent-browser screenshot emoji-input.png

# RTL text
agent-browser fill @e1 "مرحبا بالعالم هذا نص طويل بالعربية"
agent-browser screenshot rtl-input.png

# Empty submission — clear all fields and submit
agent-browser fill @e1 ""
agent-browser click @submit
agent-browser screenshot empty-submit.png
```

Check: Does text overflow? Are error messages clear and specific?

## Step 3: Test Offline Behavior

```bash
agent-browser set offline on
agent-browser click @e1
agent-browser wait 2000
agent-browser screenshot offline-interaction.png

agent-browser set offline off
agent-browser wait 2000
agent-browser screenshot back-online.png
```

Check: Is there a clear "you're offline" message? Does the app recover gracefully?

## Step 4: Test Slow Network

```bash
agent-browser network route "**/*" --delay 3000
agent-browser reload
agent-browser wait 500
agent-browser screenshot loading-state.png
agent-browser wait 5000
agent-browser network unroute
```

Check: Is there a skeleton/spinner, or does the page sit blank?

## Step 5: Test Form Validation

```bash
# Bad email
agent-browser fill @email "not-an-email"
agent-browser click @submit
agent-browser screenshot bad-email.png

# Injection attempts
agent-browser fill @e1 "'; DROP TABLE users; --"
agent-browser click @submit
agent-browser screenshot sql-injection.png

agent-browser fill @e1 "<script>alert('xss')</script>"
agent-browser click @submit
agent-browser screenshot xss-attempt.png

# Rapid submission (test double-submit prevention)
agent-browser click @submit
agent-browser click @submit
agent-browser click @submit
agent-browser screenshot rapid-submit.png
```

Check: Is input sanitized? Is submit button disabled after first click?

## Step 6: Test Error States

```bash
# Mock a 500 error
agent-browser network route "**/api/**" --status 500 --body '{"error":"Server Error"}'
agent-browser reload
agent-browser wait --load networkidle
agent-browser screenshot server-error.png
agent-browser network unroute

# Mock a 404
agent-browser network route "**/api/**" --status 404 --body '{"error":"Not Found"}'
agent-browser reload
agent-browser wait --load networkidle
agent-browser screenshot not-found.png
agent-browser network unroute
```

Check: Do error screens have retry buttons? Are messages helpful?

## Step 7: Test Keyboard Navigation

```bash
agent-browser eval "document.activeElement.blur()"
agent-browser press Tab
agent-browser screenshot focus-1.png
agent-browser press Tab
agent-browser screenshot focus-2.png
```

Check for missing focus rings programmatically:

```bash
agent-browser eval "
  const focusable = document.querySelectorAll('a[href],button,input,select,textarea,[tabindex]:not([tabindex=\"-1\"])');
  const issues = [];
  focusable.forEach(el => { el.focus(); const s = getComputedStyle(el);
    if (s.outlineStyle==='none' && s.boxShadow==='none') issues.push({tag:el.tagName,text:(el.textContent||'').slice(0,25)});
  });
  JSON.stringify({total:focusable.length, noFocusRing:issues});
"
```

## Key Principles

- **Test with real inputs** — paste actual extreme text, don't imagine it
- **Go offline for real** — `set offline on` is the truth test
- **Screenshot every error state** — if it's not screenshotted, it's not verified
- **Tab through everything** — keyboard users are real users
- **Rapid-click everything** — users double-click, triple-click, panic-click
