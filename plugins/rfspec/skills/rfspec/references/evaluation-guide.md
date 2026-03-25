# Evaluation Guide

How to compare and evaluate competing model responses.

## Comparing Options

When results come back from `/rfspec`:

1. **Read all options** before responding. Understand each approach fully.
2. **Identify meaningful differences** -- not cosmetic ones. Focus on:
   - Architectural choices (patterns, libraries, data flow)
   - Scope differences (what each included or excluded)
   - Risk areas (where one flagged something the others missed)
   - Concrete vs. vague (which named actual files, functions, steps)
3. **Write a brief comparison** -- 2-4 sentences per option covering strengths and gaps. Compare, don't summarize.
4. **Present the choice** using AskUser:
   - Use Option A as-is
   - Use Option B as-is
   - Use Option C as-is
   - Synthesize a refined version combining the best of all three
   - None of these work

## Synthesizing

If the user picks synthesis:

1. Start from the strongest option as the base.
2. Pull in specific elements from the others -- name what you're taking and why.
3. Resolve contradictions (don't concatenate).
4. The final result should read as a single coherent document, not a patchwork.

## Handling Rejection

If the user rejects all options:

1. Ask what's missing or wrong.
2. Incorporate their feedback into a refined prompt.
3. Offer to re-run `/rfspec` with the updated prompt.
