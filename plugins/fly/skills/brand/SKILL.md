---
name: brand
description: Use this skill whenever work should look or sound like Fly.io — choosing or placing a Fly.io logo, picking brand colors, status colors or gradients, checking text contrast, styling buttons, terminals or code blocks, setting type in the Fly.io typefaces (including hero titles), finding Fly.io illustrations or putting text on them, co-branding with a partner, writing in the Fly.io voice or spelling product names (Fly.io, Fly Machines, Sprites, flyctl), or checking what the brand assets may be used for. Trigger it for slides, docs, UI, social cards, emails or marketing copy that use the Fly.io brand.
license: MIT
metadata:
  author: Fly.io
  version: "1.1.0"
  homepage: https://branding.fly.dev
---

# Fly.io Brand

The Fly.io brand guidelines live at **https://branding.fly.dev**. Treat that
site as the source of truth: get logos, hex values, fonts, imagery and rules
from it rather than from memory. Values drift, and guessed brand colors are
almost always wrong.

The site serves the same data two ways. **Try WebMCP first**; fall back to
JSON if it isn't available.

## Step 1: explore the WebMCP tools with agent-browser

Every page on the site registers [WebMCP](https://github.com/webmachinelearning/webmcp)
tools. Open it in [agent-browser](https://www.npmjs.com/package/agent-browser)
and list them before doing anything else:

```bash
npx -y agent-browser open https://branding.fly.dev/
npx -y agent-browser webmcp list
```

Read a tool's input schema before calling it, then invoke it with JSON params:

```bash
npx -y agent-browser webmcp list recommend_logo --json
npx -y agent-browser webmcp invoke recommend_logo --params '{"background":"dark","space":"wide"}'
npx -y agent-browser webmcp invoke check_contrast --params '{"foreground":"#A78BFA","background":"#171434"}'
```

Each result is `{"content":[{"type":"text","text":"<JSON>"}]}`; parse the
`text` field. Close the browser when you're done:

```bash
npx -y agent-browser close
```

### The tools

| Tool | Use it for | Example params |
| --- | --- | --- |
| `get_brand_overview` | Start here: key colors, typefaces, preferred logo, section URLs | — |
| `recommend_logo` | The right logo file for a placement, with clear-space & size rules | `{"background":"light\|dark\|photo","space":"wide\|tall\|tiny\|avatar"}` |
| `list_logos` | Every logo file (SVG, PNG, ZIP), filterable | `{"layout":"brandmark","background":"dark"}` |
| `get_partner_lockup_rules` | Placing the Fly.io logo next to a partner's | — |
| `get_colors` | Hex values by group; `status` gives the success, info, warning & error scales; `gradients` gives named gradients & mesh backgrounds as CSS, plus button styles | `{"group":"logo\|core\|navy\|violet\|purple\|gray\|status\|gradients"}` |
| `check_contrast` | WCAG ratio & AA/AAA pass/fail before putting text on a color | `{"foreground":"#…","background":"#…"}` |
| `get_typography` | Typefaces, weights, font URLs, type scale, the hero-title style & its scribble underline, pairings & rules | — |
| `get_code_styles` | Terminal window anatomy, the syntax color palette & code-block rules | — |
| `list_imagery` | Illustrations (newest first) with URLs & alt text, plus text-on-image rules | `{"kind":"hero\|editorial\|spot"}` |
| `get_voice_guidelines` | Voice principles with real examples, tone by context, product naming | — |
| `get_usage_terms` | What the brand may & may not be used for | — |
| `navigate_to_page` | Move the open tab to a section (the only tool that changes anything) | `{"page":"/logo"}` |

## Step 2 (fallback): fetch the JSON

Use this when agent-browser can't run (no `npx`, no Chrome, a locked-down
sandbox) or when `webmcp list` reports no tools:

```bash
curl -s https://branding.fly.dev/brand.json
```

It holds the same data as the tools. Pull just the part you need:

```bash
curl -s https://branding.fly.dev/brand.json | jq '.colors.core'
curl -s https://branding.fly.dev/brand.json | jq '.logos[] | select(.background == "dark")'
curl -s https://branding.fly.dev/brand.json | jq '.voice.naming'
```

| Key | Contents |
| --- | --- |
| `logos`, `logoKinds`, `logoVariants`, `logoRules` | Logo files & when to use each, clear space, minimum size, misuse |
| `balloonMark` | The balloon on its own for avatars & app icons |
| `partnerRules` | Co-branding lockups |
| `colors` | `logo`, `core`, `scales`, `status` scales & contrast-checked `pairings` |
| `gradients` | `named` gradients & `mesh` backgrounds (CSS), `buttons` styles |
| `typography` | `typefaces`, `scale`, `heroTitle`, `pairings`, `rules` |
| `code` | Terminal anatomy, `syntax` colors, `rules` |
| `imagery` | `illustrations`, usage `rules`, `textOnImage` |
| `voice` | `principles`, tone by `contexts`, product `naming` |
| `usage` | `allowed`, `notAllowed`, `notes` |

Asset paths in the JSON are relative; prefix them with
`https://branding.fly.dev`. For the whole guide as Markdown (handy for a
project's `AGENTS.md`), fetch `https://branding.fly.dev/llms.txt`.

## Rules that always apply

Check the details with the tools or JSON, but never break these:

- **Name:** it's **Fly.io** in prose. Products are Fly Machines, Sprites &
  Managed Postgres. The CLI is `flyctl`; the command people type is `fly`.
- **Logo:** use the color landscape logo by default & the inverted version on
  dark backgrounds. Never recolor, stretch, rotate or add effects to it.
- **Contrast:** run `check_contrast` before putting text on any brand color.
- **Usage:** no merchandise, nothing that implies Fly.io endorses something
  without a written agreement, & link to https://fly.io on web pages.
