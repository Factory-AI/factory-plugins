---
name: distill
description: "Strip designs to their essence by removing clutter, redundancy, and unnecessary complexity. Use when the user asks to simplify, declutter, reduce noise, or make a UI cleaner and more focused."
user-invocable: true
---

Remove unnecessary complexity from the UI, reducing cognitive load while preserving everything essential.

## 1. Identify Clutter

Analyze the target for these common sources of visual and cognitive noise:

- **Redundant headers**: Titles restating what the page already says, breadcrumbs duplicating tabs
- **Too many CTAs**: Multiple competing buttons fighting for attention
- **Nested cards**: Cards inside cards inside cards—unnecessary visual containers
- **Unnecessary chrome**: Borders, dividers, backgrounds, and decorations that don't aid comprehension
- **Information overload**: Everything visible at once with no progressive disclosure
- **Repeated content**: Same information stated multiple ways
- **Dead weight**: Elements that exist "just in case" but no one uses

Use agent-browser to screenshot the current state and annotate what's clutter vs. essential.

## 2. Find the Essence

For each screen or component, answer:

- What is the **one primary goal** the user has here?
- What's the minimum needed to accomplish that goal?
- What can be removed entirely?
- What can be hidden behind progressive disclosure (accordion, "Show more", modal)?
- What can be combined into a single element?

## 3. Apply Progressive Disclosure

Move secondary complexity out of the default view:

- **Collapse advanced options** into expandable sections
- **Move edge-case actions** into overflow menus
- **Defer details** behind "Learn more" or expandable rows
- **Simplify forms** by hiding optional fields until requested
- **Reduce visible choices** to the 2-3 most common options

## 4. Simplify Systematically

### Layout
- Flatten nesting: remove wrapper elements that only add indirection
- Eliminate unnecessary cards—use spacing and alignment instead
- Replace complex multi-column layouts with linear flow where possible
- Use whitespace to create grouping instead of borders and backgrounds

### Visual
- Reduce to 1-2 accent colors plus neutrals
- Remove decorative borders, shadows, and backgrounds that don't serve hierarchy
- Limit to 3-4 type sizes maximum
- Strip gratuitous icons that duplicate adjacent text

### Content
- Cut copy in half. Then cut it in half again
- Remove redundant headings and labels
- One CTA per context—make the next step obvious
- Eliminate marketing fluff from functional UI

### Interaction
- Reduce confirmation dialogs to only destructive actions
- Replace multi-step flows with single-step where possible
- Smart defaults over explicit choices
- Inline editing over modal forms

## 5. Verify with Before/After Screenshots

Use agent-browser to capture before and after:

- Screenshot the original cluttered state
- Screenshot the simplified version
- Compare side-by-side to confirm:
  - All essential functionality is still accessible
  - The primary action is more obvious, not less
  - Users can still accomplish their goals
  - Nothing critical was hidden too aggressively

Test interactive flows to make sure progressive disclosure works correctly—hidden content should be easy to find when needed.

## Rules

- Simplification ≠ removal of features. Everything essential stays, just better organized
- Never sacrifice accessibility (labels, ARIA, contrast must remain)
- If you're unsure whether something is essential, keep it and ask
- Run lint, type-check, and tests after changes
