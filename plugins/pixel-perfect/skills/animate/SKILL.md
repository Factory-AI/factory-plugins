---
name: animate
description: "Add purposeful animations, micro-interactions, and celebration moments to a UI — then preview them in a real browser, record demos, and verify performance. Use when the user wants to add motion, transitions, micro-interactions, celebration effects, or make the interface feel alive."
argument-hint: "<url-or-file>"
user-invocable: true
---

# Animate — Motion Design with Live Preview

Add purposeful animations and celebration moments, then verify in a real browser. Record demos, test reduced motion, and confirm 60fps.

## Step 1: Identify Animation Opportunities

```bash
agent-browser open <url>
agent-browser wait --load networkidle
agent-browser snapshot -i
```

Look for:
- **Missing feedback** — buttons with no press response, forms with no validation animation
- **Jarring transitions** — instant show/hide, abrupt page changes
- **Celebration gaps** — success states that feel flat (task completed, milestone hit)
- **Static empty states** — illustrations that could subtly breathe

## Step 2: Implement Animations

### Timing & Easing Reference

| Purpose | Duration | Example |
|---------|----------|---------|
| Instant feedback | 100–150ms | Button press, toggle |
| State change | 200–300ms | Hover, menu open |
| Layout shift | 300–500ms | Accordion, modal |
| Entrance | 500–800ms | Page load, hero |

**Easing — use these, not CSS defaults:**
```css
--ease-out-quart: cubic-bezier(0.25, 1, 0.5, 1);  /* Smooth, refined */
--ease-out-expo: cubic-bezier(0.16, 1, 0.3, 1);   /* Confident, decisive */
```
Avoid bounce/elastic easing — feels dated. Exit animations run at ~75% of entrance duration.

### Celebration & Milestone Moments

- **Task completion** — animated checkmark, confetti burst, gentle scale pulse
- **Streaks** — "5 days in a row!" with particle effects
- **First-time actions** — special entrance animation for first use
- **Progress at 100%** — progress bar celebrates when full

Keep celebrations under 1 second, skippable, and varied.

### GPU Rules

Only animate `transform` and `opacity` — never `width`, `height`, `top`, `left`.

## Step 3: Preview in Browser

```bash
agent-browser reload
agent-browser wait --load networkidle
agent-browser snapshot -i

agent-browser hover @e1
agent-browser screenshot hover-state.png
agent-browser click @e2
agent-browser wait 500
agent-browser screenshot click-feedback.png
```

## Step 4: Record a Demo

```bash
agent-browser record start ./animation-demo.webm
agent-browser click @e1
agent-browser wait 800
agent-browser hover @e3
agent-browser wait 500
agent-browser click @e5
agent-browser wait 1000
agent-browser record stop
```

## Step 5: Test Reduced Motion

```bash
agent-browser set media reduced-motion
agent-browser reload
agent-browser wait --load networkidle
agent-browser screenshot reduced-motion.png
```

Verify: decorative animations disabled, essential transitions instant, content fully accessible.

```bash
agent-browser eval "
  const has = [...document.styleSheets].some(s => {
    try { return [...s.cssRules].some(r => r.conditionText?.includes('reduced-motion')); } catch(e) { return false; }
  });
  has ? 'HAS prefers-reduced-motion support' : 'MISSING — add @media (prefers-reduced-motion: reduce)'
"
```

## Step 6: Verify 60fps

```bash
agent-browser eval "
  let frames=0, last=performance.now(), drops=[];
  function check(now) { frames++; if(now-last>20) drops.push(Math.round(now-last)); last=now; if(frames<120) requestAnimationFrame(check); }
  requestAnimationFrame(check);
  setTimeout(() => document.title='drops:'+JSON.stringify(drops), 2500);
"
agent-browser wait 3000
agent-browser get title
```

Trigger animations while the counter runs. Any frame >20ms is potential jank.

## Quality Checklist

- [ ] Animations use `transform`/`opacity` only
- [ ] Easing is `ease-out-quart` or `ease-out-expo`
- [ ] Durations match the timing table
- [ ] `prefers-reduced-motion` disables decorative motion
- [ ] Celebrations are under 1s and skippable
- [ ] Demo recorded showing smooth motion
- [ ] No frame drops >20ms

## Key Principles

- **Every animation needs a reason** — feedback, transition, or delight
- **One hero moment beats scattered animations** — focus on impact
- **Preview in the browser** — code review can't judge motion
- **Record demos** — stakeholders need to see motion, not read about it
- **Respect reduced motion** — accessibility requirement, not optional
