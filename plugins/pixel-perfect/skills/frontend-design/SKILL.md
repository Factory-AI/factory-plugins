---
name: frontend-design
description: "Create distinctive, production-grade frontend interfaces with high design quality. Uses browser automation for visual verification. Generates creative, polished code that avoids generic AI aesthetics."
user-invocable: true
---

This skill guides creation of distinctive, production-grade frontend interfaces that avoid generic "AI slop" aesthetics. Implement real working code with exceptional attention to aesthetic details — then **verify it visually** with browser screenshots.

## Project Configuration

Before starting any design work, check for a `.pixel-perfect.md` file in the project root. This file contains project-specific design context:

1. **Read `.pixel-perfect.md`** from the project root. If it exists, load the design context (target audience, brand personality, tone, palette constraints, typography choices, component library).
2. **If no config exists**, examine the existing codebase for design patterns, then proceed with the design direction below. Note the absence — suggest the user create one after the session.

---

## Design Direction

Commit to a BOLD aesthetic direction:
- **Purpose**: What problem does this interface solve? Who uses it?
- **Tone**: Pick an extreme — brutally minimal, maximalist chaos, retro-futuristic, organic/natural, luxury/refined, playful/toy-like, editorial/magazine, brutalist/raw, art deco/geometric, soft/pastel, industrial/utilitarian.
- **Constraints**: Technical requirements (framework, performance, accessibility).
- **Differentiation**: What makes this UNFORGETTABLE?

Then implement working code that is production-grade, visually striking, cohesive, and meticulously refined.

## Frontend Aesthetics Guidelines

### Typography
→ *Consult [typography reference](reference/typography.md) for scales, pairing, and loading strategies.*

**DO**: Use a modular type scale with fluid sizing (clamp). Vary weights and sizes for clear hierarchy.
**DON'T**: Use overused fonts — Inter, Roboto, Arial, Open Sans, system defaults.
**DON'T**: Use monospace as lazy shorthand for "technical/developer" vibes.
**DON'T**: Put large icons with rounded corners above every heading.

### Color & Theme
→ *Consult [color reference](reference/color-and-contrast.md) for OKLCH, palettes, and dark mode.*

**DO**: Use modern CSS color functions (oklch, color-mix, light-dark). Tint neutrals toward your brand hue.
**DON'T**: Use gray text on colored backgrounds — use a shade of the background color instead.
**DON'T**: Use pure black (#000) or pure white (#fff) — always tint.
**DON'T**: Use the AI color palette: cyan-on-dark, purple-to-blue gradients, neon accents on dark.
**DON'T**: Use gradient text for "impact." Default to dark mode with glowing accents.

### Layout & Space
→ *Consult [spatial reference](reference/spatial-design.md) for grids, rhythm, and container queries.*

**DO**: Create visual rhythm through varied spacing. Use fluid spacing with clamp(). Use asymmetry intentionally.
**DON'T**: Wrap everything in cards. Nest cards inside cards. Use identical card grids endlessly.
**DON'T**: Use the hero metric layout template. Center everything. Use the same spacing everywhere.

### Visual Details
**DON'T**: Use glassmorphism everywhere. Use rounded elements with thick colored border on one side.
**DON'T**: Use sparklines as decoration. Use rounded rectangles with generic drop shadows.
**DON'T**: Use modals unless there's truly no better alternative.

### Motion
→ *Consult [motion reference](reference/motion-design.md) for timing, easing, and reduced motion.*

**DO**: Use motion to convey state changes. Use exponential easing (ease-out-quart/quint/expo).
**DO**: For height animations, use grid-template-rows transitions.
**DON'T**: Animate layout properties (width, height, padding, margin) — use transform and opacity.
**DON'T**: Use bounce or elastic easing — they feel dated.

### Interaction
→ *Consult [interaction reference](reference/interaction-design.md) for forms, focus, and loading patterns.*

**DO**: Use progressive disclosure. Design empty states that teach. Make every interactive surface responsive.
**DON'T**: Repeat the same information. Make every button primary.

### Responsive
→ *Consult [responsive reference](reference/responsive-design.md) for mobile-first, fluid design, and container queries.*

**DO**: Use container queries (@container). Adapt the interface for different contexts.
**DON'T**: Hide critical functionality on mobile.

### UX Writing
→ *Consult [ux-writing reference](reference/ux-writing.md) for labels, errors, and empty states.*

**DO**: Make every word earn its place. **DON'T**: Repeat information users can already see.

---

## The AI Slop Test

If you showed this interface to someone and said "AI made this," would they believe you immediately? If yes, that's the problem. Review the DON'T guidelines above — they are the fingerprints of AI-generated work from 2024-2025.

---

## Browser Verification

After implementing any design work, **visually verify your output** using `agent-browser`:

1. **Open the page** in agent-browser and take a full-page screenshot.
2. **Check visual hierarchy**: Does the eye flow to the most important element? Is the layout balanced?
3. **Verify typography**: Are fonts loading correctly? Is the type scale working? Check for FOUT/FOIT.
4. **Verify color**: Do the colors render as intended? Check dark mode if applicable.
5. **Test responsiveness**: Resize to mobile (375px), tablet (768px), and desktop (1280px). Screenshot each.
6. **Test interactions**: Hover, focus, and click interactive elements. Verify state transitions are smooth.
7. **Compare against intent**: Does the rendered result match the aesthetic direction you chose?

If anything looks wrong in the screenshots, fix it before moving on. Code that looks right in your head but wrong in the browser is wrong.

---

## Implementation Principles

Match implementation complexity to the aesthetic vision. Interpret creatively — make unexpected choices that feel genuinely designed for the context. No two designs should look the same. Vary between light and dark themes, different fonts, different aesthetics. NEVER converge on common choices across generations.

The AI model is capable of extraordinary creative work. Don't hold back — commit fully to a distinctive vision, then verify it renders correctly in the browser.
