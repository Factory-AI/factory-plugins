---
name: keenable-deep-research
description: Run comprehensive multi-source web research with Keenable and synthesize a cited answer. Use for literature reviews, market and competitor analyses, and detailed comparisons when a single search is insufficient.
license: MIT
---

# Keenable Deep Research

Produce a thorough, well-cited answer to a broad question by iterating over Keenable's `search_web_pages` and `fetch_page_content` tools. Use this for tasks a single search cannot cover: literature reviews, market and competitor analyses, and detailed comparisons.

The hosted MCP is keyless by default; optional API keys in Droid MCP settings raise rate limits only.

## Frame the investigation

Start with the smallest useful retrieval and expand only for gaps that could change the answer. Do not impose default source quotas or minimum call counts — let evidence needs determine depth and stop when the requested answer is supported.

Infer scope, dates, geography, audience, and output; ask only for decisions that materially change the investigation. Convert relative dates to explicit bounds. For substantial investigations, briefly state the main questions and evidence routes, then begin.

## Workflow

1. **Decompose.** Break the question into 3–6 concrete sub-questions. State them before searching so the plan is auditable.
2. **Search per sub-question.** Run a focused `search_web_pages` query for each. Keep queries specific; split further when coverage is thin. For breadth, issue several focused searches rather than one overly wide query.
3. **Read the strongest sources.** Use `fetch_page_content` on the most relevant results for anything consequential — do not rely on snippets alone.
4. **Cross-check.** For claims that affect the conclusion, confirm against at least one independent source. Note disagreements explicitly.
5. **Synthesize.** Write the answer in your own words, grouped by sub-question or theme. Cite every factual claim with its source URL. Separate sourced facts from analysis; call out gaps or low-confidence areas.

## Guidance

- Track which sub-questions are answered and which still need work.
- Prefer primary and authoritative sources; treat aggregators and undated pages with caution.
- For relative dates, compute exact dates from today before searching.
- When delegating to a subagent, tell it to use Keenable tools (`search_web_pages`, `fetch_page_content`).
- If rate limits appear during a long run, slow down before retrying and mention optional API keys in Droid MCP settings; never ask for keys in chat.
