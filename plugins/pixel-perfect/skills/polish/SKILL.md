---
name: polish
description: "Final quality pass fixing alignment, spacing, consistency, and micro-details before shipping. Uses browser screenshots for before/after comparison and interaction state testing via browser automation."
user-invocable: true
---

Perform a meticulous final pass to catch all the small details that separate good work from great work. **Every fix is verified visually** — screenshot before, fix, screenshot after.

## Preparation

1. Invoke the `frontend-design` skill to load design principles and anti-patterns.
2. Check for `.pixel-perfect.md` in the project root for project-specific design context.
3. Confirm the work is **functionally complete** — polish is the last step, not the first.

---

## Baseline Capture

Before touching any code, capture the current state with `agent-browser`:

1. **Full-page screenshot** at desktop (1280px) — this is your "before" reference.
2. **Mobile screenshot** at 375px — capture current responsive state.
3. **Note specific problem areas** — mark what you see before reading the code.

Keep these screenshots. You will compare against them after every fix.

---

## Polish Systematically

Work through these dimensions. After fixing each category, **take a new screenshot and compare** against your baseline to confirm the improvement is visible.

### Visual Alignment & Spacing

- **Pixel-perfect alignment**: Everything lines up to grid.
- **Consistent spacing**: All gaps use the spacing scale — no random 13px gaps.
- **Optical alignment**: Adjust for visual weight (icons may need offset for optical centering).
- **Spacing rhythm**: Tight groupings for related elements (8-12px), generous separation between sections (48-96px). Varied spacing within sections — not every row gets the same gap.
- **Fluid spacing**: Use clamp() so spacing breathes on larger screens.
- **Grid adherence**: Use gap for sibling spacing instead of margins.

**After fixing**: Screenshot and compare. Do the alignments look intentional now?

### Typography Refinement

- **Hierarchy consistency**: Same elements use same sizes/weights throughout.
- **Line length**: 45-75 characters for body text.
- **Widows & orphans**: No single words on last line of important headings.
- **Font loading**: No FOUT/FOIT flashes. Verify in browser — reload the page and watch.

### Color & Contrast

- **Contrast ratios**: All text meets WCAG AA. Use `agent-browser eval` to check computed color values.
- **Consistent tokens**: No hard-coded colors — all use design tokens.
- **Tinted neutrals**: No pure gray or pure black. Add subtle color tint.
- **Gray on color**: Never put gray text on colored backgrounds — use a shade of that color.

### Interaction State Testing

Use `agent-browser` to **actually test every interactive element**:

**For each button, link, and interactive surface:**
1. **Screenshot default state**.
2. **Hover** it with agent-browser — screenshot. Is there visible feedback?
3. **Focus** it (tab to it) — screenshot. Is the focus indicator visible and sufficient contrast?
4. **Click** it — screenshot the active/pressed state. Does it feel responsive?
5. **Check disabled state** if applicable.

Every interactive element needs: default, hover, focus, active, disabled, loading, error, and success states. Missing states create confusion.

### Micro-interactions & Transitions

- **Smooth transitions**: All state changes animated 150-300ms.
- **Consistent easing**: ease-out-quart/quint/expo. Never bounce or elastic.
- **No jank**: Only animate transform and opacity. Verify smoothness in browser.
- **Reduced motion**: Test with `prefers-reduced-motion` via agent-browser eval:
```js
window.matchMedia('(prefers-reduced-motion: reduce)').matches;
```

### Content & Copy

- **Consistent terminology**: Same things called same names throughout.
- **Consistent capitalization**: Title Case vs Sentence case applied uniformly.
- **No typos**: Read every visible string in the screenshots.

### Icons & Images

- **Consistent style**: All icons from same family.
- **Proper alignment**: Icons align with adjacent text optically.
- **Alt text**: All images have descriptive alt text.
- **No layout shift**: Images don't cause CLS. Reload the page in agent-browser and watch.

### Edge Cases

- **Long content**: Enter very long text and screenshot — does it overflow or truncate gracefully?
- **Empty states**: Remove data and screenshot — are empty states helpful?
- **Error states**: Trigger errors and screenshot — are messages helpful and non-blaming?

### Responsive Check

Test at three viewports using agent-browser:
- **375px**: Touch targets ≥ 44x44px, no horizontal scroll, text ≥ 14px.
- **768px**: Layout adapts logically, not just shrinks.
- **1280px**: Generous spacing, content doesn't stretch to uncomfortable widths.

Screenshot each. Compare against baseline mobile screenshot.

---

## Before/After Comparison

After all fixes are applied:

1. Take final full-page screenshots at desktop and mobile.
2. Compare side-by-side with your baseline captures.
3. Verify every fix is visible and nothing regressed.
4. If anything looks worse, revert that specific change.

---

## Polish Checklist

- [ ] Visual alignment perfect at all breakpoints
- [ ] Spacing uses scale consistently
- [ ] Typography hierarchy consistent
- [ ] All interactive states implemented and verified in browser
- [ ] All transitions smooth (verified visually)
- [ ] Copy is consistent and polished
- [ ] Icons consistent and properly sized
- [ ] Contrast ratios meet WCAG AA
- [ ] Keyboard navigation works (tested via agent-browser tab)
- [ ] No layout shift on load
- [ ] Reduced motion preference respected
- [ ] Before/after screenshots confirm improvement

**NEVER**: Polish before it's functionally complete. Introduce bugs while polishing. Perfect one thing while leaving others rough. Skip the browser verification — if you didn't screenshot it, you didn't verify it.

Remember: Polish until it feels effortless, looks intentional, and works flawlessly. The screenshots don't lie.
