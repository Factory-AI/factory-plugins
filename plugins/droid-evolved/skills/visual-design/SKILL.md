---
name: visual-design
version: 3.0.0
description: |
  Visual design and content creation. Use when:
  - User asks for images: logos, icons, app assets, diagrams, flowcharts,
    architecture diagrams, patterns, textures, photo edits, restorations
  - User needs a presentation or slide deck
  - User wants help with frontend design, UI/UX, or making something look better
  - User asks you to build a web app, website, landing page, or any HTML/CSS interface
  Covers nanobanana CLI for image generation, Slidev for presentations,
  and practical frontend design patterns.
---

# Visual design

This skill covers visual and design work. Three areas, each with its own doc.

## Image generation

Create and edit images from the command line using nanobanana CLI.

```bash
npm install -g @factory/nanobanana
export GEMINI_API_KEY="your-key"

nanobanana generate "company logo" --count=4 --styles=modern,minimal
nanobanana edit photo.png "remove background"
nanobanana icon "settings gear" --style=flat
nanobanana diagram "auth flow" --type=flowchart
```

Handles: logos, icons, diagrams, patterns, photo restoration, UI assets, visual sequences.

See: [image-generation.md](./image-generation.md)

## Presentations

Create slides using Slidev, a markdown-based presentation tool.

```bash
npm init slidev@latest
slidev                    # dev server
slidev export --format pptx   # export to PowerPoint
slidev build              # build as hostable SPA
```

Write slides in markdown, get code highlighting, animations, diagrams, and Vue components.

See: [presentations.md](./presentations.md) and [reference-slide-example.md](./reference-slide-example.md)

## Frontend design

Tactics for building good-looking interfaces. Covers the creative side (tone, aesthetics, inspiration) and the technical side (spacing scales, type hierarchy, color systems, responsive patterns).

See: [frontend-design.md](./frontend-design.md)
