---
name: screenshot-diff
description: "Take before/after screenshots and produce a pixel diff to verify visual changes and catch unintended regressions. Use when the user wants to compare visual states, verify CSS changes didn't break anything, or diff two pages side by side."
argument-hint: "<url-or-file> [--compare <second-url>]"
user-invocable: true
---

# Screenshot Diff — Visual Regression Detection

Capture baseline screenshots, make changes, capture again, and produce a pixel diff. Catch every unintended visual regression before it ships.

## Workflow A: Before/After Diff (Same Page)

### 1. Take Baseline Screenshots

Before making any changes, capture the current state at all viewports:

```bash
agent-browser open <url>
agent-browser wait --load networkidle

# Desktop baseline
agent-browser set viewport 1280 800
agent-browser screenshot --full baseline-desktop.png

# Tablet baseline
agent-browser set viewport 768 1024
agent-browser screenshot --full baseline-tablet.png

# Mobile baseline
agent-browser set viewport 375 812
agent-browser screenshot --full baseline-mobile.png
```

### 2. Make Code Changes

Apply the CSS, HTML, or component changes the user requested.

### 3. Take After Screenshots

Reload the page (or re-open if serving locally) and capture again:

```bash
agent-browser reload
agent-browser wait --load networkidle

# Desktop after
agent-browser set viewport 1280 800
agent-browser screenshot --full after-desktop.png

# Tablet after
agent-browser set viewport 768 1024
agent-browser screenshot --full after-tablet.png

# Mobile after
agent-browser set viewport 375 812
agent-browser screenshot --full after-mobile.png
```

### 4. Produce Pixel Diff

Use `agent-browser diff screenshot` to compare:

```bash
agent-browser diff screenshot baseline-desktop.png after-desktop.png diff-desktop.png
agent-browser diff screenshot baseline-tablet.png after-tablet.png diff-tablet.png
agent-browser diff screenshot baseline-mobile.png after-mobile.png diff-mobile.png
```

The diff image highlights changed pixels in red. Review each diff and report:
- **Expected changes** — the areas the user intended to modify
- **Unexpected changes** — regressions in unrelated areas

## Workflow B: Side-by-Side URL Comparison

Compare two different URLs (e.g. staging vs production, old vs new design):

```bash
agent-browser diff url "https://staging.example.com" "https://production.example.com" diff-output.png
```

Or manually for more control:

```bash
# Page A
agent-browser open "https://staging.example.com"
agent-browser wait --load networkidle
agent-browser set viewport 1280 800
agent-browser screenshot --full page-a.png

# Page B — use a new tab
agent-browser tab new "https://production.example.com"
agent-browser wait --load networkidle
agent-browser set viewport 1280 800
agent-browser screenshot --full page-b.png

# Diff
agent-browser diff screenshot page-a.png page-b.png diff-comparison.png
```

## Multi-Viewport Sweep

For thorough regression testing, test at these widths:

| Viewport | Width | Height | Use Case |
|----------|-------|--------|----------|
| Small mobile | 320 | 568 | iPhone SE |
| Mobile | 375 | 812 | iPhone 14 |
| Large mobile | 428 | 926 | iPhone 14 Pro Max |
| Tablet portrait | 768 | 1024 | iPad |
| Tablet landscape | 1024 | 768 | iPad landscape |
| Desktop | 1280 | 800 | Standard laptop |
| Wide desktop | 1920 | 1080 | Full HD monitor |

## Reading the Diff Report

Summarize findings as:

```
## Screenshot Diff Report

### Desktop (1280px)
- diff-desktop.png: [description of changes]
- Expected: ✅ Header color changed
- Unexpected: ❌ Footer spacing shifted by 4px

### Tablet (768px)
- diff-tablet.png: [description of changes]
- Expected: ✅ Nav collapsed to hamburger
- Unexpected: None

### Mobile (375px)
- diff-mobile.png: [description of changes]
- Expected: ✅ Card layout stacked
- Unexpected: ❌ Button text truncated

### Verdict
- Regressions found: [count]
- Action needed: [yes/no]
```

## Key Principles

- **Always baseline before changing** — you can't diff without a before
- **Test at least 3 viewports** — regressions hide at breakpoints
- **Review every diff image** — automated pixel count isn't enough
- **Zero unexpected changes is the goal** — any surprise pixel is a bug
- **Re-diff after fixes** — confirm regressions are resolved
