---
name: audit
description: "Run technical quality checks across accessibility, performance, theming, responsive design, and anti-patterns. Uses browser automation for real-device verification. Generates a scored report with P0-P3 severity ratings."
user-invocable: true
---

Run systematic **technical** quality checks with **real browser verification** and generate a comprehensive report. Don't fix issues — document them for other skills to address.

## Preparation

1. Invoke the `frontend-design` skill to load design principles and anti-patterns.
2. Check for `.pixel-perfect.md` in the project root for project-specific design context.

---

## Browser Verification Phase

Before scoring anything in code, **open the actual page** with `agent-browser` and perform real visual checks:

### Multi-Viewport Screenshots
Take screenshots at three breakpoints to assess responsive behavior:
```
agent-browser screenshot at viewport 375x812   → mobile.png
agent-browser screenshot at viewport 768x1024  → tablet.png
agent-browser screenshot at viewport 1280x800  → desktop.png
```

### Contrast Verification
Use `agent-browser eval` to check actual rendered contrast ratios on key text elements:
```js
// Get computed styles on rendered elements
const el = document.querySelector('.hero-heading');
const styles = getComputedStyle(el);
// Log color and background-color for manual contrast check
console.log('color:', styles.color, 'bg:', styles.backgroundColor);
```
Check at least: primary headings, body text, secondary/muted text, text on colored backgrounds, and button labels.

### Accessibility Scan
Use `agent-browser eval` to check for common a11y gaps:
```js
// Missing alt text
document.querySelectorAll('img:not([alt])').length;
// Buttons/links without accessible names
document.querySelectorAll('button:not([aria-label]):empty').length;
// Inputs without labels
document.querySelectorAll('input:not([aria-label]):not([id])').length;
// Missing landmarks
document.querySelectorAll('main, nav, header, footer').length;
```

### Screenshot Problem Areas
When you find a visual issue, screenshot the specific region and annotate what's wrong. This creates a visual paper trail.

---

## Diagnostic Scan

Score each dimension 0–4 using the criteria below.

### 1. Accessibility (A11y)
**Check for**: Contrast ratios < 4.5:1, missing ARIA, keyboard navigation gaps, improper heading hierarchy, missing alt text, unlabeled form inputs.

**Score**: 0=Inaccessible (fails WCAG A), 1=Major gaps, 2=Partial effort with significant gaps, 3=WCAG AA mostly met, 4=WCAG AA fully met.

### 2. Performance
**Check for**: Layout thrashing, expensive animations (animating width/height/top/left), missing lazy loading, unnecessary imports, unnecessary re-renders.

**Score**: 0=Severe issues, 1=Major problems, 2=Partial optimization, 3=Mostly optimized, 4=Fast and lean.

### 3. Theming
**Check for**: Hard-coded colors, broken dark mode, inconsistent token usage, values that don't update on theme change.

**Score**: 0=No theming, 1=Mostly hard-coded, 2=Tokens exist but inconsistent, 3=Tokens used with minor gaps, 4=Full token system, dark mode works.

### 4. Responsive Design
**Check for**: Fixed widths that break on mobile, touch targets < 44x44px, horizontal scroll, text scaling failures, missing breakpoints. **Use the multi-viewport screenshots** from the browser verification phase to validate.

**Score**: 0=Desktop-only, 1=Some breakpoints with many failures, 2=Works on mobile with rough edges, 3=Responsive with minor issues, 4=Fluid across all viewports.

### 5. Anti-Patterns (CRITICAL)
Check against ALL the **DON'T** guidelines in the frontend-design skill. Look for AI slop tells: AI color palette, gradient text, glassmorphism, hero metrics, card grids, generic fonts. **Use the desktop screenshot** — AI slop is obvious visually before it's obvious in code.

**Score**: 0=AI slop gallery (5+ tells), 1=Heavy AI aesthetic (3-4), 2=Some tells (1-2), 3=Mostly clean, 4=Distinctive and intentional.

---

## Generate Report

### Audit Health Score

| # | Dimension | Score | Key Finding |
|---|-----------|-------|-------------|
| 1 | Accessibility | ? | [most critical issue] |
| 2 | Performance | ? | |
| 3 | Responsive Design | ? | |
| 4 | Theming | ? | |
| 5 | Anti-Patterns | ? | |
| **Total** | | **??/20** | **[Rating band]** |

**Rating bands**: 18-20 Excellent, 14-17 Good, 10-13 Acceptable, 6-9 Poor, 0-5 Critical.

### Anti-Patterns Verdict
Pass/fail: Does this look AI-generated? List specific tells. Be brutally honest. Reference the screenshots.

### Executive Summary
- Audit Health Score: **??/20** ([rating band])
- Total issues found (count by severity: P0/P1/P2/P3)
- Top 3-5 critical issues with screenshot evidence
- Recommended next steps

### Detailed Findings by Severity

Tag every issue **P0-P3**:
- **P0 Blocking**: Prevents task completion — fix immediately
- **P1 Major**: Significant difficulty or WCAG AA violation — fix before release
- **P2 Minor**: Annoyance, workaround exists — fix in next pass
- **P3 Polish**: Nice-to-fix — address if time permits

For each issue: **[P?] Issue name**, Location, Category, Impact, Standard violated, Recommendation, and screenshot reference if applicable.

### Patterns & Systemic Issues
Identify recurring problems that indicate systemic gaps rather than one-off mistakes.

### Positive Findings
Note what's working well — good practices to maintain and replicate.

## Recommended Actions

List recommended skills in priority order (P0 first):

1. **[P?] `skill-name`** — Brief description with specific context from findings
2. **[P?] `skill-name`** — Brief description

> Re-run `audit` after fixes to see your score improve.

**NEVER**: Report issues without explaining impact. Provide generic recommendations. Skip positive findings. Report false positives without browser verification.
