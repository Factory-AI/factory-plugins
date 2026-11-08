---
name: normalize
description: "Audits and realigns UI to match design system standards using browser-based visual comparison. Use when the user mentions consistency, design drift, mismatched styles, tokens, or wants to bring components back in line with the system."
user-invocable: true
---

Scan the codebase for design system inconsistencies and fix them with visual verification.

## 1. Scan for Inconsistent Tokens

Search the codebase for hard-coded values that should use design tokens:

- **Colors**: Grep for hex values, `rgb()`, `hsl()` not from the token set
- **Spacing**: Find arbitrary pixel values for margin/padding/gap
- **Border radius**: Look for inconsistent rounding (`4px` here, `8px` there, `6px` elsewhere)
- **Typography**: Spot hard-coded font sizes, weights, or families outside the type scale
- **Shadows**: Identify one-off box-shadow values

Build a list of every deviation grouped by category.

## 2. Visual Comparison via agent-browser

Use agent-browser to capture the current state:

- Screenshot each inconsistent component at its current state
- Navigate to reference components (the "correct" versions) and screenshot those
- Place screenshots side-by-side for comparison
- Document which components deviate and how

```
Navigate to the page containing the component.
Take a screenshot of the component in its current state.
Navigate to the reference/canonical version.
Take a screenshot for comparison.
```

## 3. Generate Consistency Report

Before changing anything, produce a report:

- **Token violations**: List every hard-coded value and what token it should use
- **Pattern deviations**: Components that don't follow established patterns
- **Severity**: Rank by visual impact (major inconsistency vs. minor drift)
- **Affected files**: Exact file paths and line numbers

Present this report and confirm the plan before proceeding.

## 4. Apply Systematic Fixes

Work through the report methodically:

- **Replace hard-coded colors** with design token variables
- **Swap arbitrary spacing** for spacing scale tokens
- **Unify border-radius** to the token set
- **Normalize typography** to use the type scale
- **Consolidate shadows** to defined elevation tokens
- **Replace custom components** with design system equivalents where they exist

Apply fixes file by file. Don't introduce new patterns—use what the system already defines.

## 5. Visual Verification

After each batch of fixes, verify with agent-browser:

- Screenshot the fixed components
- Compare against the reference screenshots from step 2
- Confirm the normalized components still look intentional, not broken
- Check that interactive states (hover, focus, active) remain correct
- Test at multiple viewport sizes to catch responsive regressions

If anything looks wrong, revert and adjust. Normalization should make things more consistent, not less polished.

## Rules

- Never invent new tokens—use what the design system provides
- If no design system exists, identify the most common values as the de facto standard
- Don't normalize intentional variations (e.g., marketing pages vs. app UI)
- Preserve accessibility: contrast ratios, focus states, ARIA attributes
- Run lint, type-check, and tests after changes
