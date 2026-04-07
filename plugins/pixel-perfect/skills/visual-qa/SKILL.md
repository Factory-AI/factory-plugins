---
name: visual-qa
description: "Open a page in a real browser, screenshot at multiple viewports, and produce an annotated visual report covering contrast, touch targets, overflow, and focus indicators. Use when the user asks for visual QA, accessibility audit, responsive check, or wants to verify how a page actually renders."
argument-hint: "<url-or-file>"
user-invocable: true
---

# Visual QA — Real Rendered Inspection

Render a page in a real browser at multiple viewports and produce an annotated visual report. Not code-level guessing — what users actually see.

## 1. Open and Screenshot at Three Viewports

```bash
agent-browser open <url-or-file>   # use file:/// for local files
agent-browser wait --load networkidle

```bash
agent-browser set viewport 375 812
agent-browser screenshot --full mobile.png

agent-browser set viewport 768 1024
agent-browser screenshot --full tablet.png

agent-browser set viewport 1280 800
agent-browser screenshot --full desktop.png
```

## 3. Get Interactive Element Map

```bash
agent-browser snapshot -i   # returns refs (@e1, @e2...) for all interactive elements
```

## 4. Check Contrast Ratios (WCAG AA: 4.5:1 normal, 3:1 large)

```bash
agent-browser eval "
  const results = [];
  document.querySelectorAll('p,span,h1,h2,h3,h4,h5,h6,a,button,label,li').forEach(el => {
    const s = getComputedStyle(el);
    if (s.backgroundColor !== 'rgba(0, 0, 0, 0)') {
      results.push({ tag: el.tagName, text: el.textContent.slice(0,30), color: s.color, bg: s.backgroundColor, size: s.fontSize });
    }
  });
  JSON.stringify(results.slice(0, 40));
"
```

Flag any pairs that look too close. Compute relative luminance if needed.

## 5. Check Touch Target Sizes

Every interactive element must be at least 44×44px:

```bash
agent-browser eval "
  const small = [];
  document.querySelectorAll('a,button,input,select,textarea,[role=button],[tabindex]').forEach(el => {
    const r = el.getBoundingClientRect();
    if (r.width > 0 && (r.width < 44 || r.height < 44)) {
      small.push({ tag: el.tagName, text: el.textContent.slice(0,30), w: Math.round(r.width), h: Math.round(r.height) });
    }
  });
  JSON.stringify(small);
"
```

## 6. Check for Horizontal Overflow

Run at each viewport after setting it:

```bash
agent-browser eval "
  document.documentElement.scrollWidth > document.documentElement.clientWidth
    ? 'OVERFLOW: ' + document.documentElement.scrollWidth + ' > ' + document.documentElement.clientWidth
    : 'OK'
"
```

## 7. Check Focus Indicators

Tab through interactive elements and verify visible focus rings:

```bash
agent-browser press Tab
agent-browser screenshot focus-1.png
agent-browser press Tab
agent-browser screenshot focus-2.png
agent-browser press Tab
agent-browser screenshot focus-3.png
```

Check programmatically:

```bash
agent-browser eval "
  const issues = [];
  document.querySelectorAll('a,button,input,select,textarea').forEach(el => {
    el.focus();
    const s = getComputedStyle(el);
    if (s.outlineStyle === 'none' && s.boxShadow === 'none') {
      issues.push({ tag: el.tagName, text: el.textContent.slice(0,30) });
    }
  });
  JSON.stringify(issues);
"
```

## 8. Produce the Visual Report

```
## Visual QA Report — [page URL]

### Viewport Screenshots
- Mobile (375px): mobile.png
- Tablet (768px): tablet.png
- Desktop (1280px): desktop.png

### Contrast Issues
| Element | Color | Background | Pass? |
|---------|-------|------------|-------|

### Touch Target Violations (<44×44px)
| Element | Size | Text |
|---------|------|------|

### Horizontal Overflow
- Mobile: OK / OVERFLOW
- Tablet: OK / OVERFLOW
- Desktop: OK / OVERFLOW

### Focus Indicator Issues
| Element | Issue |
|---------|-------|

### Summary
- Critical: [count] | Warning: [count] | Pass: [count]
```

## Key Principles

- **Screenshot first, read code second** — what renders is the truth
- **Test every viewport** — mobile issues are invisible on desktop
- **Check real contrast** — computed styles, not design tokens
- **Verify focus is visible** — keyboard users depend on it
- **Flag overflow early** — horizontal scroll on mobile is a UX failure
