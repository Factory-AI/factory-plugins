# Showcase Polish

This atom describes the visual polish system. It is invoked by the **compose** atom — you should not need to invoke it directly. Load it when you need to understand what the presets look like and how the cinematic layers work.

## What you control

You control the visual output by choosing a **preset** and passing **props**. Everything else is automatic — the fframes composition renders every cinematic layer internally based on the preset and palette.

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

`transitionStyle` selects the crossfade presentation. Schema lives in `../compose/ATOM.md`; preset-tier matching:

| Preset | Recommended (default first) | Avoid |
|---|---|---|
| `factory`, `factory-hero` | `motion-blur`, `light-leak`, `whip-pan`, `flash` | `glitch-lite` (clashes with warm tone) |
| `hero`, `presentation` | `motion-blur`, `whip-pan`, `flash` | `light-leak` (warm sweep clashes with cool palette) |
| `macos`, `minimal` | `motion-blur` | `glitch-lite`, `light-leak` (too much personality for utilitarian frames) |

`codeAnnotations` is preset-agnostic — palette and font stack are auto-derived. See `../compose/ATOM.md` for schema and authoring rules.

## Operational notes

**Render time**: about 6-7x the video length on 4 cores at 1920x1080 (a 13.5s video renders in ~90s); it scales with cores and output pixels. Set worker timeouts to 5x that estimate, plus a few minutes for the first-use build.

**Common failure modes**:
- Content truncated or a panel frozen early: the content length is the longest clip, probed by the renderer; a shorter clip holds its final frame. Trim or re-record the sources.
- First render fails while building: the renderer is compiled from `${DROID_PLUGIN_ROOT}/fframes` on first use and needs Rust, clang/libclang, and libx264 (see Prerequisites). Later renders reuse the binary.

**Debugging layout**: `render-showcase.sh --still <frame>` renders one frame through the same validation and staging as a full render (see `../compose/ATOM.md` Step 3).

**Cleanup**: the renderer removes only the work directory it created, on success, failure, or cancellation via Ctrl-C / SIGTERM (to the script's PID or its process group).

## Rendering

Use the render script from **compose** — see `../compose/ATOM.md` Step 3 for full usage:

```bash
RENDER=${DROID_PLUGIN_ROOT}/scripts/render-showcase.sh

$RENDER --props "${RUN_DIR}/props.json" --output "${RUN_DIR}/showcase.mp4" "${RUN_DIR}/clip.mp4"
```

## Prerequisites

- **Rust** (stable, via [rustup](https://rustup.rs)), **clang/libclang**, and **libx264** to build the renderer once
- **ffmpeg** and **ffprobe** (clip probing and the H.264 encode)
- **agg** for `.cast` clips

```bash
sudo apt-get install -y clang libclang-dev libx264-dev   # macOS: xcode-select --install; brew install x264
cargo build --release --manifest-path ${DROID_PLUGIN_ROOT}/fframes/Cargo.toml
```
