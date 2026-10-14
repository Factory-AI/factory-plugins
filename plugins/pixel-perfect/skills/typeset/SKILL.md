---
name: typeset
description: "Improve typography by fixing font choices, hierarchy, sizing, and readability with browser-verified rendering. Use when the user mentions fonts, type, readability, text hierarchy, sizing looks off, or wants more polished typography."
user-invocable: true
---

Fix flat, generic, or poorly structured typography using real browser rendering verification.

## 1. Assess Current Typography

Scan the codebase for type-related values:

- Grep for `font-size`, `font-weight`, `font-family`, `line-height`, `letter-spacing`
- List every unique value used—identify how many sizes, weights, and families exist
- Check for a type scale (consistent ratios) vs. arbitrary sizes (14px, 15px, 16px, 17px)
- Look for hard-coded values that should use design tokens

Use agent-browser to screenshot the rendered typography:
- Can you instantly tell headings from body from captions?
- Does the font match the product's personality?
- Is there clear visual hierarchy or does everything look the same weight?

## 2. Check Font Rendering via agent-browser

Verify fonts actually load and render correctly:

```javascript
// Check for FOUT/FOIT via document.fonts API
const fontsReady = await document.fonts.ready;
const loadedFonts = [...document.fonts].filter(f => f.status === 'loaded');
console.log('Loaded fonts:', loadedFonts.map(f => `${f.family} ${f.weight}`));
```

- Detect Flash of Unstyled Text (FOUT) by watching the page load
- Detect Flash of Invisible Text (FOIT) where text disappears during load
- Verify `font-display: swap` or `optional` is set on custom fonts
- Check that fallback fonts are metric-compatible with the primary font

## 3. Verify Line Lengths

Use agent-browser eval to measure actual line lengths:

```javascript
// Check character count per line on text containers
const textEl = document.querySelector('.article-body p');
const text = textEl.textContent;
const lineWidth = textEl.offsetWidth;
const fontSize = parseFloat(getComputedStyle(textEl).fontSize);
const charsPerLine = Math.round(lineWidth / (fontSize * 0.5));
// Ideal: 45-75 characters per line
```

- Measure line lengths on body text containers
- Flag anything outside 45-75 characters as a readability issue
- Check that `max-width` is set using `ch` units on text containers
- Verify line lengths at mobile, tablet, and desktop viewports

## 4. Establish Type Scale

If the current typography is arbitrary, establish a proper scale:

- **5 levels cover most UIs**: caption (12px), secondary (14px), body (16px), subheading (20px), heading (25px)
- Use a consistent ratio between levels (1.25 recommended for UI, 1.333 for content-heavy)
- Use `rem` units, never `px` for font sizes (respects user zoom settings)
- Define sizes as design tokens: `--text-caption`, `--text-body`, `--text-heading`

### Weight Strategy
- **Regular (400)**: Body text, descriptions
- **Medium (500)**: Labels, secondary headings
- **Semibold (600)**: Subheadings, emphasis
- **Bold (700)**: Primary headings only
- Load only the weights you use—each adds to page weight

### Line Height
- **Headings**: 1.1–1.2 (tight, since large text needs less leading)
- **Body text**: 1.5–1.7 (loose, for comfortable reading)
- **Captions/labels**: 1.3–1.4

## 5. Apply Fixes

- Replace arbitrary font sizes with the type scale tokens
- Set `max-width: 65ch` on text containers for readable line lengths
- Fix line-height values per context (headings vs. body)
- Add `font-display: swap` to all `@font-face` declarations
- Use `tabular-nums` for data tables and aligned numbers
- Remove unused font weights from imports
- Ensure body text is at least 16px / 1rem

## 6. Verify Visual Hierarchy in Screenshots

Use agent-browser to confirm the typography improvements:

- Screenshot the page and squint test: can you see the hierarchy with blurred vision?
- Verify headings are clearly distinct from body text (size + weight + spacing)
- Check that the type scale creates rhythm and consistency
- Compare before/after screenshots side-by-side
- Test at multiple viewport sizes—hierarchy should hold on mobile
- Verify no FOUT/FOIT by watching a fresh page load

## Rules

- Body text must be at least 16px / 1rem
- Use `rem` for font sizes, never `px` (accessibility: user zoom)
- Maximum 2-3 font families (usually 1 is enough)
- Maximum 3-4 font weights per family
- Never disable browser zoom (`user-scalable=no`)
- Don't pair fonts that are similar but not identical (two geometric sans-serifs)
- Run lint, type-check, and tests after changes
