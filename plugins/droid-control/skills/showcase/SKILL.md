---
name: showcase
description: Background knowledge for droid-control workflows -- not invoked directly. Visual polish for videos via Remotion-powered window chrome, animations, and branded backgrounds.
user-invocable: false
---

# Showcase Polish

This atom describes the visual polish system. It is invoked by the **compose** atom — you should not need to invoke it directly. Load it when you need to understand what the presets look like and how the cinematic layers work.

## What you control

You control the visual output by choosing a **preset** and passing **props**. Everything else is automatic — the Remotion components handle all cinematic layers internally based on the preset and palette.

## Presets

Each preset configures window chrome, spacing, background style, and palette selection.

| Preset | Look | Best for |
|---|---|---|
| `factory` | Warm black bg with amber radial glow, traffic-light dots, 12px radius, generous margins. Rich cinematic warmth. | Official Factory content |
| `factory-hero` | Same as `factory` + gradient background. Maximum cinematic punch. | Factory landing pages, social media |
| `hero` | Cool gradient bg, large margins, prominent shadow. | Non-Factory marketing, third-party |
| `macos` | Clean dark bg, traffic lights, subtle shadow. Professional but understated. | General-purpose demos, README heroes |
| `presentation` | Black bg, generous margins. Designed to look good on slides. | Talks, slide decks |
| `minimal` | No window bar, tiny radius, tight margins. Barely-there frame. | Docs embeds, inline clips |

### What each preset automatically includes

**Factory / factory-hero presets** (warm palette):
- Warm radial background vignette with amber glow blobs that intensify over video duration
- Warm-tinted box shadow with faint accent glow halo
- Warm color grade overlay (amber tint)
- Floating particles in Factory Orange

**All other presets** (cool Catppuccin palette):
- Cool-toned solid or gradient background
- Neutral box shadow
- Subtle cool color grade overlay
- Floating particles in accent blue

**All presets** include: floating particles, noise texture overlay, color grade, motion blur title→content transition (configurable via `transitionStyle`), animated window entrance, staggered panel entrance (side-by-side), and optional `codeAnnotations` syntax-highlighted overlays during the main content sequence.

## Visual palettes

Palette is auto-selected based on preset. Factory/factory-hero use the warm palette; everything else uses cool.

### Factory (warm)

| Token | Hex | Role |
|---|---|---|
| bg | `#0a0804` | Warm near-black |
| surface | `#18120e` | Terminal content bg |
| accent | `#EE6018` | Factory Orange |
| text | `#f0e8e0` | Warm white |
| muted | `#948781` | De-emphasized text |

### Catppuccin (cool)

| Token | Hex | Role |
|---|---|---|
| bg | `#0d1117` | Cool dark |
| surface | `#181818` | Content bg |
| accent | `#89b4fa` | Blue accent |
| text | `#cdd6f4` | Cool white |
| muted | `#6c7086` | De-emphasized text |

## Transition styles

`transitionStyle` selects the crossfade presentation. Schema lives in `compose/SKILL.md`; preset-tier matching:

| Preset | Recommended (default first) | Avoid |
|---|---|---|
| `factory`, `factory-hero` | `motion-blur`, `light-leak`, `whip-pan`, `flash` | `glitch-lite` (clashes with warm tone) |
| `hero`, `presentation` | `motion-blur`, `whip-pan`, `flash` | `light-leak` (warm sweep clashes with cool palette) |
| `macos`, `minimal` | `motion-blur` | `glitch-lite`, `light-leak` (too much personality for utilitarian frames) |

`codeAnnotations` is preset-agnostic — palette and font stack are auto-derived. See `compose/SKILL.md` for schema and authoring rules.

## Operational notes

**Render time**: ~1-3 minutes for a 30-60s video at 1920x1080. Set worker timeouts to 5 minutes.

**Common failure modes**:
- `clipDuration` mismatch: video has blank frames at end or truncates early. The `render-showcase.sh` script auto-detects duration via ffprobe — prefer using it over manual `npx remotion render`.
- Missing clips in `public/`: render fails with "Could not read file." The render script handles staging automatically.
- Missing npm dependencies: run `cd ${REMOTION_DIR} && npm install` if rendering fails on first use.

**Debugging layout**: Use `npx remotion still Showcase --props='...' --frame=30 --scale=0.5 /tmp/check.png` to render a single frame and inspect it visually before committing to a full render.

**Cleanup**: The `render-showcase.sh` script removes staged clips from `public/` after rendering. If you run `npx remotion render` directly, clean up `public/` manually.

## Rendering

Use the render script from **compose** — see compose/SKILL.md Step 3 for full usage:

```bash
RENDER=${DROID_PLUGIN_ROOT}/scripts/render-showcase.sh

$RENDER --props /tmp/props.json --output /tmp/showcase.mp4 /tmp/clip.mp4
```

## Advanced: GlitchTitle

A stylized glitch title card component exists at `src/components/GlitchTitle.tsx` for edgy/hacker-aesthetic intros (pixel-decay effect with scattered blocks assembling into text). **This is NOT wired into the default Showcase composition.** Using it requires writing a custom Remotion composition. Only pursue this if specifically requested — the standard TitleCard handles all normal use cases.

## Prerequisites

- **Node.js** (>= 18)
- **Chrome / Chromium** (Remotion uses headless Chrome)
- **ffmpeg** and **ffprobe** (Remotion uses these under the hood)

```bash
cd ${DROID_PLUGIN_ROOT}/remotion && npm install
```
