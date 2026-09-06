---
name: critique
description: "Evaluate design from a UX perspective using browser automation. Assesses visual hierarchy, information architecture, emotional resonance, and cognitive load with quantitative scoring and real interaction testing."
user-invocable: true
---

Conduct a holistic design critique by **actually looking at and interacting with the interface** in a real browser. Think like a design director who opens the product, uses it, and gives feedback based on what they see — not what they imagine from reading code.

## Preparation

1. Invoke the `frontend-design` skill to load design principles and anti-patterns.
2. Check for `.pixel-perfect.md` in the project root for project-specific design context.
3. Identify the URL or file path to open the interface.

---

## Phase 1: Visual Capture & Flow Testing

Before writing a single word of critique, **use the interface**:

### Screenshot the Current State
Open the page with `agent-browser` and take a full-page screenshot. This is your primary input — study the visual hierarchy, composition, and overall impression before reading any code.

### Test the Primary User Flow
Use `agent-browser` to walk through the main user journey:
1. Identify the primary action the interface is designed for.
2. Click through it step by step — every button, every form field, every transition.
3. Screenshot each major state change.
4. Note where you hesitate, get confused, or feel friction. Those are your findings.

### Test Interaction States
Use `agent-browser` to verify hover, focus, and active states on key interactive elements:
- Hover over buttons and links — is there feedback?
- Tab through the page — are focus indicators visible and logical?
- Click interactive elements — does the active state feel responsive?

---

## Phase 2: Design Critique

Evaluate the interface across these dimensions, using your screenshots and interaction experience as primary evidence:

### 1. AI Slop Detection (CRITICAL)
Look at the screenshot. Does this look like every other AI-generated interface? Check against ALL the **DON'T** guidelines in the frontend-design skill. The test: would someone immediately believe "AI made this"?

### 2. Visual Hierarchy
Does the eye flow to the most important element first? Is there a clear primary action visible in 2 seconds? Do size, color, and position communicate importance correctly?

### 3. Information Architecture & Cognitive Load
Is the structure intuitive? Are there too many choices at once (>4 visible options at a decision point)? Is complexity revealed progressively or dumped upfront?

### 4. Emotional Journey
What emotion does this interface evoke? Does it match the brand personality? Does the experience end well (confirmation, celebration, clear next step)?

### 5. Discoverability & Affordance
Are interactive elements obviously interactive? Would a user know what to do without instructions? Do hover/focus states provide useful feedback? (You tested these in Phase 1.)

### 6. Composition & Balance
Does the layout feel balanced? Is whitespace intentional? Is there visual rhythm? Does asymmetry feel designed or accidental?

### 7. Typography as Communication
Does the type hierarchy signal what to read first, second, third? Is body text comfortable (45-75 char line length)? Do font choices reinforce the tone?

### 8. Color with Purpose
Is color used to communicate, not just decorate? Does the palette feel cohesive? Do accent colors draw attention to the right things?

### 9. States & Edge Cases
Empty states: do they guide users? Loading states: do they reduce perceived wait? Error states: are they helpful and non-blaming?

### 10. Microcopy & Voice
Is the writing clear and concise? Does it sound human? Are labels unambiguous?

---

## Phase 3: Persona Testing via Browser

Select 2-3 personas most relevant to this interface type and **simulate their experience** using `agent-browser`:

### First-Timer Test
Navigate the interface as if seeing it for the first time:
- Can you complete the primary action without prior knowledge?
- Is there visible help or onboarding?
- Screenshot the exact point where a newcomer would get stuck.

### Power User Test
Try to accomplish tasks efficiently:
- Are there keyboard shortcuts? Try common ones (Cmd+K, /, Escape).
- How many clicks for the primary action? Count them.
- Is there a way to skip steps or use advanced features?

### Accessibility User Test
Navigate with keyboard only (use agent-browser tab/enter/escape):
- Can you reach all interactive elements?
- Is the focus order logical?
- Are focus indicators visible?

For each persona, list **specific elements and interactions that fail** — not generic descriptions.

---

## Phase 4: Present Findings

### Design Health Score

Score each of Nielsen's 10 heuristics 0–4:

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | ? | [specific finding or "—"] |
| 2 | Match System / Real World | ? | |
| 3 | User Control and Freedom | ? | |
| 4 | Consistency and Standards | ? | |
| 5 | Error Prevention | ? | |
| 6 | Recognition Rather Than Recall | ? | |
| 7 | Flexibility and Efficiency | ? | |
| 8 | Aesthetic and Minimalist Design | ? | |
| 9 | Error Recovery | ? | |
| 10 | Help and Documentation | ? | |
| **Total** | | **??/40** | **[Rating band]** |

Be honest. A 4 means genuinely excellent. Most real interfaces score 20-32.

### Anti-Patterns Verdict
Pass/fail with screenshot evidence. List specific AI slop tells found.

### Overall Impression
Gut reaction — what works, what doesn't, the single biggest opportunity.

### What's Working
2-3 things done well, with specific reasons why.

### Priority Issues
3-5 most impactful problems, ordered by importance. For each:
- **[P0-P3] What**: Name it clearly
- **Why it matters**: How this hurts users
- **Fix**: Concrete recommendation
- **Screenshot**: Reference the specific screenshot showing the problem

### Persona Red Flags
Specific failures per persona from Phase 3, with screenshots.

### Recommended Actions

List recommended skills in priority order:
1. **`skill-name`** — Brief description with specific context from findings

> Re-run `critique` after fixes to see your score improve.

**NEVER**: Give vague feedback. Say "consider exploring" instead of giving a concrete fix. Soften criticism. Skip the browser testing phases — code review alone misses visual and interaction problems.
