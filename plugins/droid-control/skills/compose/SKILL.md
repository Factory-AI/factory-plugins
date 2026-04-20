---
name: compose
description: Background knowledge for droid-control workflows -- not invoked directly. Video assembly via Remotion — title cards, layout, transitions, effects, and showcase polish.
user-invocable: false
---

# Compose

This atom owns the full video assembly pipeline. You receive raw outputs from the **capture** stage and produce a single polished artifact. Follow the pipeline below step by step.

## Inputs

The command or capture stage should have provided a handoff with two sections:

### Mechanical (structured)

- **clips**: paths to raw recordings (.cast, .mp4, .webm, .png)
- **driver**: tuistory | true-input | agent-browser
- **layout**: `single` | `side-by-side`
- **labels**: text for each clip (e.g., "BEFORE (dev)", "AFTER (PR)")
- **speed**: multiplier (default 3x)
- **fidelity**: `auto` | `compact` | `standard` | `inspect` (optional; auto => side-by-side=inspect, single=standard)
- **title**: text for the title card
- **subtitle**: one-sentence summary
- **sections**: text banners for chapters `[{t, title}]` (optional)
- **keys**: keystroke events `[{t, label, dur?}]` (if overlay requested)
- **showcase**: preset name -- `macos`, `minimal`, `hero`, `presentation`, `factory`, `factory-hero`
- **effects tier**: `utilitarian` | `full` | `none` (see "Choosing effects at compose time" below)
- **output**: desired output path

### Creative (natural language)

Free-text guidance on what to emphasize: which moments to hold, what the title card should convey, whether phase cards are warranted, how to trim dead time. Use this -- along with the effects tier -- for editorial decisions, including choosing specific effects to apply.

## Pipeline

```
1. Build props   →  construct the Showcase JSON props
2. Render        →  render-showcase.sh (converts .cast, stages clips, renders, cleans up)
3. Finalize      →  verify and output
```

Remotion handles all compositing in a single pass — title cards, transitions, window chrome, backgrounds, keystroke overlays, spotlights, particles, noise, and color grading are all automatic. You construct the props JSON; the engine does the rest.

## Remotion project & helper script

```bash
REMOTION_DIR=${DROID_PLUGIN_ROOT}/remotion
RENDER=${DROID_PLUGIN_ROOT}/scripts/render-showcase.sh
```

## Showcase mode vs Demo mode

Both use the same Remotion pipeline but target different visual registers.

| | Showcase | Demo |
|---|---|---|
| **Goal** | Cinematic, high-polish marketing material | Clear, utilitarian demonstration — single or comparison, whichever the story calls for |
| **Preset** | `factory`, `factory-hero`, or `hero` | `macos`, `minimal`, or `presentation` |
| **Effects tier** | **Full** -- spotlight, zoom, callout, keystroke overlay. Go all out. | **Utilitarian** -- zoom for readability, keystroke overlay for user actions |
| **Audience** | External — landing pages, social, marketing | Internal — PR reviews, docs, QA |

**Decision rule**: If the video will be seen outside the eng team, use Showcase mode. If it's for a PR description, internal demo, or documentation embed, use Demo mode. The visual polish layers (warm glow, particles, color grade, motion blur) are always present but their intensity is palette-driven — Factory presets produce rich cinematic warmth while Catppuccin presets stay subtle and cool.

### Choosing effects at compose time

The command stage committed an **effects tier** (utilitarian, full, or none). Now that you have actual recordings, choose specific effects:

- **Utilitarian**: Add zoom effects for any small or hard-to-read text. Add keystroke overlay if user actions were captured. Skip spotlight and callout unless something is genuinely hard to find on screen.
- **Full**: Use the full palette. Spotlight the key proof points. Zoom into details. Add callout annotations where the UI isn't self-explanatory. Layer keystroke overlay throughout. Aim for cinematic -- the viewer should feel guided, not left to scan.
- **None**: Pass `"effects": []` in props. Keystroke overlay is still allowed if committed separately.

## Step 1: Choose fidelity and pacing

`render-showcase.sh` auto-selects `inspect` for side-by-side and `standard` for single-clip layouts when `fidelity` is omitted or set to `auto`.

| Fidelity | Default output size | Remotion encode | Polish overlays | Best for |
|---|---|---|---|---|
| `compact` | 1920x1080 | H.264 CRF 21, JPEG frames | full grain + grade | Small embeds |
| `standard` | 1920x1080 | H.264 CRF 18, JPEG frames | reduced grain + grade | Single-panel demos |
| `inspect` | 2560x1440 | H.264 CRF 14, PNG frames | minimal grain + grade | Side-by-side comparisons / tiny text |

### .cast conversion behavior

`render-showcase.sh` converts `.cast` inputs through `agg -> gif -> ffmpeg -> mp4` before Remotion render, using the asciicast's own cols/rows and fixed font metrics so element positions remain stable across fidelity profiles.

**CRITICAL: `agg` replaces ALL 16 ANSI colors with its theme palette.** The render script uses a custom Droid CLI theme. If you manually run `agg`, never omit `--theme` and never use built-in themes like `monokai` or `dracula`.

The `--theme` flag accepts a comma-separated hex string (no `#` prefix): `bg,fg,color0..color7` (10 values) or `bg,fg,color0..color7,color8..color15` (18 values for bright variants).

Note: `tuistory` recordings of the Droid CLI typically emit NO color escape codes -- the CLI uses Ink's direct rendering which doesn't produce standard ANSI SGR sequences in the cast output. The theme's `bg` (first value: `181818`) and `fg` (second value: `e0d0c0`, warm white) are the only colors that will affect the output. The warm-white fg avoids the cold blue-grey look of default themes.

For other terminals that DO emit ANSI color codes, build the full theme string from the terminal's actual color settings.

**Pacing**: Target the final video duration, not a speed factor. A blind multiplier either makes text illegible or leaves dead air.

| Demo type | Target duration | Why |
|---|---|---|
| Single feature, 3-5 steps | 30-45s | Viewer watches the whole thing in one breath |
| Before/after comparison, side-by-side | 45-75s | Each panel needs time to land; frozen-vs-active contrasts need a beat |
| Multi-phase or complex flow | 60-120s | Phase cards give the viewer reset points; rushing defeats the purpose |

Set the `speed` prop to hit the target: if the raw recording is 3 minutes and the target is 60s, use `"speed": 3`. If it's already 40s raw, use `"speed": 1` or trim dead time instead. **Trim first, speed second** -- cut LLM thinking pauses, build waits, and network delays from the `.cast` with `asciinema cut` or by splitting segments, then apply a gentle speed-up only if still over target.

**Keystroke timing adjustment**: If a keystroke list was emitted during capture, its timestamps are in raw recording time. When you apply a speed multiplier, you **must** divide every timestamp by the speed factor before passing it to the Remotion props. A keystroke at raw `t=6.0s` in a 3x video should appear at `t=2.0s`.

### Non-.cast clips

`.mp4`, `.webm`, and `.png` clips are passed through to Remotion unchanged except for staging into `public/`. Re-encode non-`.cast` clips manually only if their pixel format or dimensions are invalid.

### Clip aspect ratio (mandatory check for browser captures)

At the default 1920×1080 output with factory preset margins, panels come out roughly:

| Layout | Panel aspect |
|---|---|
| `single` | ~1760×920 (≈16:9 landscape) |
| `side-by-side` | ~872×920 per panel (≈8:9, near-square / slight portrait) |

`.cast` conversions target panel aspect automatically. **Pre-recorded `.mp4` / `.webm` clips do not** — if the clip aspect doesn't match the panel aspect, the clip will letterbox (with the default `objectFit: "contain"`) or crop (with `"cover"`).

**Common pitfall**: browser captures are typically 16:9 landscape (e.g. 1280×720). Dropped into a `side-by-side` layout they render as a thin band with giant black bars above and below.

Two fixes, in priority order:

1. **Re-capture at a panel-friendly viewport** — go back to the capture stage and set viewport to ~960×1000 for `side-by-side`, ~1280×720 for `single`.
2. **Pass `"objectFit": "cover"` in props** — crops the clip edges to fill the panel. Acceptable when the relevant UI is centered and edges are expendable. Not acceptable if cropped content matters (e.g. sidebar UI cut off).

`.cast` clips rarely need this since their rendered aspect is derived from cols/rows; it's almost always a browser-capture concern.

### Duration checkpoint (mandatory, before proceeding)

Check whether the planned speed factor produces a final duration within the pacing table's target range:

```
final_duration = clip_duration / speed_factor
```

| If final_duration is... | Action |
|---|---|
| Within the target range | Proceed |
| Below the minimum | **Reduce the speed factor** until the target is met. If even at 1x the clip is below the minimum, the recording is too short — return to **capture** and add more interaction steps. |
| Above the maximum | Trim dead time first (`asciinema cut`), then increase the `speed` prop |

This checkpoint is not optional. A video that lands outside the target range fails verification.

## Step 2: Build props

### Choose layout

**Default: `single`.** One clip of the target/final state. New features, bug-fix proofs, walkthroughs, and README heroes all belong here.

Use `side-by-side` only when the story is fundamentally a comparison: regression (broken vs fixed), behavior-preserving refactor, or an explicit user request. Never fabricate a "before" clip to justify the side-by-side shape.

Save the `showcaseSchema` JSON to a temp file:

```bash
DEMO_TMP="$(mktemp -d /tmp/droid-demo-XXXXXX)"
PROPS="${DEMO_TMP}/showcase-props.json"

cat > "$PROPS" << 'EOF'
{
  "clips": ["demo.cast"],
  "layout": "single",
  "fidelity": "auto",
  "labels": [],
  "speed": 3,
  "title": "PR #11621 — Prevent session freezes",
  "subtitle": "Bash Mode blocks interactive commands and supports ESC cancellation",
  "preset": "factory",
  "keys": [
    {"t": 2.0, "label": "vim"},
    {"t": 5.5, "label": "sleep 100"},
    {"t": 8.0, "label": "Esc"}
  ],
  "sections": [],
  "effects": [],
  "speedNote": "3x speed",
  "windowTitle": "droid demo"
}
EOF
```

For a comparison flow, swap `"clips"` to two paths, `"layout"` to `"side-by-side"`, and populate `"labels"` (e.g., `["BEFORE (main)", "AFTER (PR #11621)"]`).

Use a run-scoped props path like `$PROPS`; do not reuse a global `/tmp/showcase-props.json` across rerenders or concurrent demos.

**CRITICAL: `clipDuration` handling.** The render script auto-detects clip duration via ffprobe when `clipDuration` is omitted from the props. If you set it manually, it **must** match the actual clip duration or you get blank frames (too long) or truncation (too short). When in doubt, omit it and let the script auto-detect.

### Props reference

| Prop | Type | Required | Description |
|---|---|---|---|
| `clips` | `string[]` | yes | Filenames (basenames only — the render script handles staging) |
| `layout` | `"single" \| "side-by-side"` | yes | Composition layout |
| `labels` | `string[]` | yes | Labels for each clip (visible in side-by-side; pass `[]` for single) |
| `fidelity` | `"compact" \| "standard" \| "inspect"` | no | Output quality/compression profile. Omit for auto-selection by layout. |
| `speed` | `number` | no | Playback speed for `.cast -> agg` conversion. |
| `title` | `string` | yes | Title card heading |
| `subtitle` | `string` | yes | Title card subheading |
| `preset` | preset name | yes | Visual preset — see table below |
| `keys` | `Keystroke[]` | yes | Keystroke overlay events (pass `[]` for none) |
| `sections` | `Section[]` | no | Section banners to mark chapters (pass `[]` for none) |
| `effects` | `Effect[]` | yes | Effect timeline (pass `[]` for none) |
| `clipDuration` | `number` | no | Clip duration in seconds. **Auto-detected by render script if omitted.** |
| `speedNote` | `string` | no | Shown on title card (e.g., `"3x speed"`) |
| `windowTitle` | `string` | no | Text in the window title bar |
| `width` | `number` | no | Output width (default: 2560 for inspect, else 1920) |
| `height` | `number` | no | Output height (default: 1440 for inspect, else 1080) |
| `objectFit` | `"contain" \| "cover" \| "fill"` | no | How each clip fits its panel. Default `"contain"` (letterbox to preserve aspect). Use `"cover"` when clip aspect doesn't match panel aspect and you'd rather crop than see black bars. See "Clip aspect ratio" below. |
| `codeAnnotations` | `CodeAnnotation[]` | no | Timed syntax-highlighted code overlays shown during the main content sequence. See "Code annotations" below. |
| `transitionStyle` | `"motion-blur" \| "flash" \| "whip-pan" \| "light-leak" \| "glitch-lite"` | no | Presentation used for title→content and content→outro transitions. Default `"motion-blur"` preserves existing aesthetic. See "Transition styles" below. |

### Preset quick reference

| Preset | Look | Palette | Best for |
|---|---|---|---|
| `factory` | Warm black bg, traffic lights, 12px radius, 80px margin | Factory (warm) | Official Factory content |
| `factory-hero` | Same as factory + gradient bg | Factory (warm) | Factory landing pages, social |
| `hero` | Gradient bg, generous margins, prominent shadow | Catppuccin (cool) | Non-Factory marketing |
| `macos` | Dark bg, traffic lights, clean frame | Catppuccin (cool) | General-purpose demos |
| `presentation` | Black bg, generous margins | Catppuccin (cool) | Slide decks, talks |
| `minimal` | No window bar, tiny radius, tight margins | Catppuccin (cool) | Docs embeds, inline clips |

### Keystroke schema

```
{ t: number, label: string, dur?: number }
```

- `t`: Time in seconds relative to clip start (not title card). Adjust for speed factor.
- `label`: Display text (e.g., `"Ctrl+C"`, `"vim main.rs"`)
- `dur`: Display duration in seconds (default: 1.2s). Auto-cut when next keystroke starts.

### Section schema

```json
{ "t": 2.0, "title": "Testing basic echo" }
```

- `t`: Time in seconds relative to clip start (not title card). Adjust for speed factor.
- `title`: Display text for the section header. Remains visible until the next section starts.

### Effect types

| Effect | Props | Description |
|---|---|---|
| `fade-in` | `t`, `dur` | Fade from black |
| `fade-out` | `t`, `dur` | Fade to black |
| `zoom` | `t`, `dur`, `to: {x,y,w,h}` | Directed zoom to a target region (30% in, 40% hold, 30% out) |
| `spotlight` | `t`, `dur`, `on: {x,y,w,h}`, `dim?` | Dim everything except a region (`dim`: 0–1, default 0.6) |
| `callout` | `t`, `dur`, `text`, `at: {x,y}` | Text overlay anchored to a point |

Regions use percentage strings (e.g., `"25%"`) relative to the video dimensions.

### When to use effects

| Effect | Use when… | Don't use when… |
|---|---|---|
| `spotlight` | Drawing attention to a specific region (error, status change) | The whole frame is relevant |
| `zoom` | Small text or detail that's hard to read at full scale | Content is already legible |
| `callout` | Annotating something the viewer might not recognize | The UI is self-explanatory |
| Keystroke overlay | Showing user actions (typing, key presses) | No user interaction in the clip |

**Less is more.** One well-timed spotlight has more impact than five overlapping effects.

### Code annotations

Timed syntax-highlighted code card laid over the captured video. Use for PR demos where the decisive source change needs to sit next to the runtime proof.

| Field | Type | Required | Description |
|---|---|---|---|
| `t` | `number` | yes | Start time in seconds, relative to clip start. Adjust for `speed` factor (same rule as `keys[].t`). |
| `dur` | `number` | yes | How long the card stays visible, in seconds. |
| `code` | `string` | yes | Source text; `\n` for multiline; no trailing newline. |
| `language` | `string` | no | Prism language id (`tsx`, `ts`, `py`, `rust`, `bash`, ...). Default `tsx`. |
| `title` | `string` | no | Small caption above the code (usually a file path). |
| `highlight` | `[{start,end}]` | no | 1-based inclusive line ranges with accent background + left border. |
| `focus` | `[{start,end}]` | no | 1-based inclusive line ranges kept at full opacity; others are dimmed/blurred. |
| `position` | `"top-right" \| "center" \| "bottom-left"` | no | Default `"top-right"`. Move to `"bottom-left"` if the captured top-right is load-bearing. |

Keep it short — aim for ≤ 15 lines per card, hold for 3–6 seconds.

### Transition styles

`transitionStyle` selects the title→content and content→outro crossfade presentation. Both transitions in one render share the same style. `flash` and `light-leak` derive their tint from the preset palette. Default `motion-blur` is always safe; preset-tier guidance lives in `showcase/SKILL.md`.

| Style | Feel | Use when… |
|---|---|---|
| `motion-blur` | Subtle dolly, blur + opacity crossfade | Default for PR demos, Factory content, most showcase work |
| `flash` | Quick palette-tinted flash at midpoint | Bug-fix proofs where the "after" state should feel sudden |
| `whip-pan` | Horizontal pan + motion blur | Energetic showcase / marketing when pacing is fast |
| `light-leak` | Warm gradient sweep | Factory-branded landing/social clips |
| `glitch-lite` | RGB channel offset + horizontal band | Security/vulnerability proof, terminal aesthetic; never default, never twice |

## Step 3: Render

Use the render script — it handles clip staging, duration detection, rendering, and cleanup:

```bash
RENDER=${DROID_PLUGIN_ROOT}/scripts/render-showcase.sh

# Basic render
$RENDER --props "$PROPS" --output /tmp/demo.mp4 \
  /tmp/before.cast /tmp/after.cast

# Or with inline props (useful for simple cases)
$RENDER --props-inline '{"clips":["clip.mp4"],"layout":"single","labels":[],"title":"Demo","subtitle":"Test","preset":"macos","keys":[],"effects":[]}' \
  --output /tmp/demo.mp4 /tmp/clip.mp4
```

The script:
1. Converts `.cast` inputs to `.mp4` using the selected fidelity profile
2. Copies clip files into `${REMOTION_DIR}/public/`
3. Auto-detects `clipDuration` via ffprobe if missing from props
4. Runs `npx remotion render Showcase` with profile-specific encode flags
5. Cleans up staged and generated clips

**Quick frame check** (sanity-check layout before full render):

```bash
cd ${REMOTION_DIR}
npx remotion still Showcase --props="$(cat "$PROPS")" --frame=30 --scale=0.5 /tmp/check.png
```

**Render time**: Expect ~1-3 minutes for a 30-60s video at 1920x1080. Set worker timeouts accordingly (5 minutes is safe).

## Step 4: Finalize

Check the result:

```bash
ffprobe -v quiet -print_format json -show_format -show_streams /tmp/demo.mp4
```

Confirm:
- Resolution is 1920x1080 (or matches the expected output)
- Duration is reasonable (not 0s, not hours)
- File size is manageable (under 5 MB for GitHub embeds, 25 MB hard limit)
- Pixel format is yuv420p (universal playback)

## Outputs

Hand to the **verify** stage:

```
## Compose outputs
- video: /tmp/demo-pr-11621.mp4
- resolution: 1920x1080
- duration: 42s
- size: 3.2 MB
- preset: factory
- keystrokes: 3 events overlaid
- effects: 1 spotlight
- engine: remotion
```

## Screenshot-only artifacts (proofs, QA)

Not every deliverable is a video. For proof and QA workflows, compose may just organize screenshots and snapshots:

### Annotated screenshot set

```bash
ffmpeg -y -i before.png -i after.png \
  -filter_complex "
    [0:v]scale=960:-1[left];
    [1:v]scale=960:-1[right];
    [left][right]hstack=inputs=2[out]" \
  -map "[out]" comparison.png
```

### Markdown report with embedded evidence

For text-based deliverables, organize the evidence into a structured report rather than a video. The verify stage handles this.
