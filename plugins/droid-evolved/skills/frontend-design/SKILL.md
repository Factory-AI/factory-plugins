---
name: frontend-design
version: 1.0.0
description: |
  Build good-looking web interfaces. Use when:
  - User asks you to build a web app, website, landing page, or HTML page
  - User asks for a one-off tool, utility, or demo app
  - User is starting a new frontend project
  - User wants to improve how something looks
  - User mentions UI, design, styling, or making something look better
  This applies to ANY frontend work, not just "design" tasks. Even simple
  apps benefit from basic design principles.
---

# Frontend design

Practical tactics for designing and building frontend interfaces. This is about making things look good and work well, not about frameworks or tooling.

## Start with the creative vision

Before touching code, understand what you're trying to achieve emotionally and aesthetically.

### Tone

What feeling should this interface convey? Professional and trustworthy? Playful and fun? Calm and minimal? Energetic and bold?

The tone affects every decision: colors, typography, spacing, imagery, micro-interactions.

If there's an existing design language, follow it first. Match the existing tone before introducing new elements. Consistency matters more than novelty.

### Aesthetics

Look at references. What interfaces do you admire that have a similar purpose? What makes them work?

Collect screenshots, note what you like about each. Is it the generous whitespace? The bold typography? The subtle shadows? The color palette?

Don't copy directly, but understand the principles behind what you're drawn to.

## Then add constraints

Constraints make design easier, not harder. They eliminate decision fatigue.

### Spacing scale

Pick a base unit (4px or 8px) and stick to multiples of it.

```
4, 8, 12, 16, 24, 32, 48, 64, 96, 128
```

Every margin, padding, and gap should come from this scale. No magic numbers like 13px or 47px.

### Type scale

Pick a ratio (1.25 or 1.333 are common) and generate your sizes:

```
12, 14, 16, 20, 24, 32, 40, 48
```

Each size has a purpose: body text, subheadings, headings, display text.

### Color palette

Start minimal:

- One primary color (your brand or accent)
- Neutrals: white, black, and 3-4 grays
- One semantic color for errors (red)
- One for success (green)

You can always add more later. Starting with fewer colors forces you to use them intentionally.

### Layout grid

Use a 12-column grid with consistent gutters. Most layouts can be built with 12 columns.

## Design in the browser

Designing directly in code (HTML/CSS) has advantages:

- You see real rendering, real responsiveness
- Faster iteration than design tools for some changes
- No handoff problems
- Version control

Start with mobile, then scale up. It's easier to add space than to remove it.

## Common patterns

### Cards

Cards group related content. Keep them simple:

- Consistent padding (16px or 24px)
- Subtle border or shadow to separate from background
- Clear hierarchy: image, title, description, action

### Forms

- Labels above inputs, not beside
- One column for most forms
- Clear error states (red border, error message below)
- Generous touch targets (44px minimum height on mobile)

### Navigation

- Keep primary nav minimal (5-7 items max)
- Current page should be obvious
- Mobile: hamburger menu or bottom nav
- Breadcrumbs for deep hierarchies

### Empty states

Don't leave empty areas blank. Show:

- What would normally be here
- How to add content
- An illustration if appropriate

### Loading states

- Skeleton screens over spinners when possible
- Show progress for long operations
- Don't block the whole UI if only part is loading

## Avoiding AI-slop aesthetics

Generated designs often look generic. To avoid this:

**Be specific about what you want.** "A modern dashboard" gives you something forgettable. "A dashboard with a dark theme, data visualizations using a blue-to-purple gradient, compact information density, inspired by trading terminals" gives you something distinctive.

**Add constraints.** Limit your color palette. Commit to a specific type scale. Use a consistent spacing system. Constraints create cohesion.

**Look at real references.** Find interfaces you admire. Understand why they work. Borrow principles, not pixels.

**Edit ruthlessly.** Generated designs often have too much going on. Remove decorative elements that don't serve a purpose. Simplify until it feels too simple, then add back one thing.

**Test with real content.** Lorem ipsum hides problems. Use realistic text lengths, real images, actual data.

## Responsive design

Design for mobile first, then add complexity for larger screens.

Breakpoints (common):

- Mobile: up to 640px
- Tablet: 641px to 1024px
- Desktop: 1025px and up

What changes between breakpoints:

- Number of columns
- Font sizes (slightly larger on desktop)
- Navigation pattern
- Amount of content visible

What stays the same:

- Color palette
- Typography hierarchy
- Brand elements
- Core functionality

## Accessibility basics

- Color contrast: 4.5:1 minimum for text
- Focus states: visible focus rings for keyboard navigation
- Alt text: describe images meaningfully
- Semantic HTML: use headings, lists, buttons correctly
- Touch targets: 44x44px minimum

These aren't nice-to-haves. They're requirements for usable interfaces.

## Quick checklist

Before shipping:

- [ ] Consistent spacing from the scale
- [ ] Typography hierarchy is clear
- [ ] Colors meet contrast requirements
- [ ] Works on mobile
- [ ] Focus states are visible
- [ ] Loading and error states exist
- [ ] Empty states are handled
- [ ] Real content has been tested
