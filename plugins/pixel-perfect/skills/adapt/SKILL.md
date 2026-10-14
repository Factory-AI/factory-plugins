---
name: adapt
description: "Adapt designs for responsive layouts using real browser screenshots at each breakpoint. Tests touch targets, overflow, and orientation with actual rendered output. Use when the user mentions responsive design, mobile layout, breakpoints, viewport adaptation, or cross-device compatibility."
argument-hint: "<url-or-file> [target-viewport]"
user-invocable: true
---

# Adapt — Browser-Verified Responsive Design

Adapt designs across screen sizes using real rendered screenshots. Document what's broken at each breakpoint visually, then verify fixes in the browser.

## Step 1: Document Current State at Every Breakpoint

```bash
agent-browser open <url>
agent-browser wait --load networkidle

agent-browser set viewport 375 812
agent-browser screenshot --full current-mobile.png

agent-browser set viewport 768 1024
agent-browser screenshot --full current-tablet.png

agent-browser set viewport 1024 768
agent-browser screenshot --full current-tablet-landscape.png

agent-browser set viewport 1280 800
agent-browser screenshot --full current-desktop.png
```

Review each screenshot. List every issue per viewport before writing any code.

## Step 2: Test Touch Targets

At mobile viewport, check all interactive elements are ≥ 44×44px:

```bash
agent-browser set viewport 375 812
agent-browser eval "
  const violations = [];
  document.querySelectorAll('a,button,input,select,textarea,[role=button]').forEach(el => {
    const r = el.getBoundingClientRect();
    if (r.width > 0 && (r.width < 44 || r.height < 44)) {
      violations.push({ tag: el.tagName, text: el.textContent.slice(0,25), w: Math.round(r.width), h: Math.round(r.height) });
    }
  });
  JSON.stringify(violations);
"
```

Check spacing between adjacent targets (≥ 8px):

```bash
agent-browser eval "
  const els = [...document.querySelectorAll('a,button,[role=button]')];
  const crowded = [];
  for (let i = 0; i < els.length; i++) {
    const a = els[i].getBoundingClientRect();
    for (let j = i+1; j < Math.min(i+5, els.length); j++) {
      const b = els[j].getBoundingClientRect();
      const gap = Math.min(Math.abs(a.right-b.left), Math.abs(b.right-a.left), Math.abs(a.bottom-b.top), Math.abs(b.bottom-a.top));
      if (gap >= 0 && gap < 8) crowded.push({ el1: els[i].textContent.slice(0,15), el2: els[j].textContent.slice(0,15), gap: Math.round(gap) });
    }
  }
  JSON.stringify(crowded.slice(0,15));
"
```

## Step 3: Check Horizontal Overflow

At each viewport, verify no content overflows:

```bash
agent-browser eval "
  const sw = document.documentElement.scrollWidth;
  const cw = document.documentElement.clientWidth;
  sw > cw ? 'OVERFLOW: ' + sw + ' > ' + cw : 'OK'
"
```

Find offending elements:

```bash
agent-browser eval "
  const cw = document.documentElement.clientWidth;
  const wide = [];
  document.querySelectorAll('*').forEach(el => {
    const r = el.getBoundingClientRect();
    if (r.right > cw + 2) wide.push({ tag: el.tagName, class: el.className.toString().slice(0,30), overflow: Math.round(r.right - cw) });
  });
  JSON.stringify(wide.slice(0,10));
"
```

## Step 4: Test Orientation Changes

```bash
agent-browser set viewport 768 1024
agent-browser screenshot --full portrait.png

agent-browser set viewport 1024 768
agent-browser screenshot --full landscape.png

agent-browser set viewport 375 812
agent-browser screenshot --full phone-portrait.png

agent-browser set viewport 812 375
agent-browser screenshot --full phone-landscape.png
```

Check: Does nav collapse correctly? Do images scale? Does content reflow?

## Step 5: Make Changes and Verify

After implementing responsive fixes, re-screenshot and re-run all checks:

```bash
agent-browser reload
agent-browser wait --load networkidle

agent-browser set viewport 375 812
agent-browser screenshot --full fixed-mobile.png

agent-browser set viewport 768 1024
agent-browser screenshot --full fixed-tablet.png

agent-browser set viewport 1280 800
agent-browser screenshot --full fixed-desktop.png
```

Re-run touch target and overflow checks to confirm zero violations.

## Key Principles

- **Screenshot before coding** — document the broken state visually
- **Test orientation** — portrait ≠ landscape, both must work
- **Verify touch targets in the browser** — bounding box math doesn't lie
- **Fix overflow at every width** — horizontal scroll on mobile is unacceptable
- **Re-screenshot after every fix** — confirm changes visually, not just in code
