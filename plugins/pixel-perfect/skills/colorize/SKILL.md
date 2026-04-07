---
name: colorize
description: "Add strategic color to monochromatic UIs with browser-verified contrast and accessibility. Use when the user mentions the design looking gray, dull, lacking warmth, needing more color, or wanting a more vibrant palette."
user-invocable: true
---

Transform monochrome or gray interfaces with purposeful color, verified in the actual browser.

## 1. Assess the Current Palette

Scan the codebase for the existing color usage:

- Grep for all color values: hex, rgb, hsl, oklch, CSS custom properties
- Map which colors are used where (backgrounds, text, borders, icons, states)
- Identify the dominant palette: pure grayscale? One timid accent? All neutrals?
- Check for existing brand colors or design tokens

Use agent-browser to screenshot the current state—see how monochromatic it actually looks rendered.

## 2. Plan Color Introduction

Define a purposeful palette (2-4 colors max beyond neutrals):

- **Primary color**: The dominant accent (60% of colored elements)—brand identity, primary CTAs
- **Semantic colors**: Success (green), error (red), warning (amber), info (blue)
- **Supporting accent**: A secondary color for variety (30%)
- **Highlight color**: Sparingly used for key moments (10%)

Map where each color will appear:
- Primary CTAs and interactive elements
- Status indicators and badges
- Section backgrounds and card accents
- Icons and data visualization
- Hover/focus/active states

## 3. Verify Contrast via agent-browser

Before applying colors, check contrast ratios on rendered elements:

```javascript
// Use eval in agent-browser to check computed styles
const el = document.querySelector('.button-primary');
const styles = getComputedStyle(el);
const bgColor = styles.backgroundColor;
const textColor = styles.color;
// Calculate contrast ratio
```

- Check every text-on-background combination meets WCAG AA (4.5:1 for text, 3:1 for large text and UI components)
- Verify interactive elements have sufficient contrast against their backgrounds
- Test focus ring visibility on colored backgrounds

## 4. Apply Color Strategically

### State Communication
- **Success states**: Green tones for completed, saved, active
- **Error states**: Red/rose for failures, validation errors, destructive actions
- **Warning states**: Amber/orange for caution, approaching limits
- **Info states**: Blue for informational messages, tips
- Don't rely on color alone—pair with icons or text labels

### Surfaces and Backgrounds
- Replace pure gray backgrounds (`#f5f5f5`) with warm-tinted neutrals
- Add subtle colored section backgrounds to create visual rhythm
- Use colored left-borders on cards for categorization
- Tint empty states with soft brand color

### Interactive Elements
- Color primary buttons with the brand/primary color
- Add hover state color shifts (darken or lighten by one step)
- Color links distinctly from body text
- Ensure focus states use visible, colored rings

### Data and Status
- Color-code badges, tags, and status indicators
- Use color meaningfully in charts and graphs
- Apply consistent color mapping (green = good, red = bad) across all features

## 5. Check Colorblind Accessibility

Use agent-browser eval to simulate color vision deficiency:

```javascript
// Apply CSS filter to simulate color blindness
document.documentElement.style.filter = 'url(#protanopia-filter)';
// Or use SVG filters for deuteranopia, tritanopia
```

- Verify the UI is still usable under protanopia (red-blind) simulation
- Check deuteranopia (green-blind) simulation
- Confirm that color is never the **only** indicator—icons, patterns, or text must accompany it
- Screenshot each simulation for the record

## 6. Visual Verification

Use agent-browser to confirm the final result:

- Screenshot the colorized UI and compare against the monochrome original
- Verify color communicates meaning (users can tell success from error at a glance)
- Check that the palette feels cohesive, not chaotic
- Test at multiple viewport sizes—color should work on mobile too
- Confirm dark mode compatibility if applicable

## Rules

- Max 2-4 colors beyond neutrals—strategic beats saturated
- Never use color as the only indicator (accessibility requirement)
- Don't put gray text on colored backgrounds—use darker shades of the background color
- Avoid pure black (`#000`) and pure white (`#fff`) for large surfaces
- Every color must have a purpose: hierarchy, meaning, or brand identity
- Run lint, type-check, and tests after changes
