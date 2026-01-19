# Image generation

Generate and edit images from the command line using nanobanana, a CLI wrapper around Gemini's image models.

## Setup

Install via npm:

```bash
npm install -g @factory/nanobanana
```

Or download a standalone binary from https://github.com/Factory-AI/nanobanana-cli/releases

Set your API key:

```bash
export GEMINI_API_KEY="your-api-key"
```

Get a key at https://aistudio.google.com/apikey

## Commands

### generate

Create images from text:

```bash
nanobanana generate "sunset over mountains"
```

Generate multiple variations:

```bash
nanobanana generate "company logo" --count=4
```

Apply styles:

```bash
nanobanana generate "cat portrait" --styles=watercolor,sketch,pixel-art
```

Open images after generation:

```bash
nanobanana generate "forest path" --preview
```

### edit

Modify an existing image:

```bash
nanobanana edit photo.png "add sunglasses to the person"
nanobanana edit landscape.jpg "change the sky to sunset"
nanobanana edit product.png "remove the background"
```

### restore

Fix old or damaged photos:

```bash
nanobanana restore old_photo.jpg
nanobanana restore damaged.png "remove scratches, enhance colors"
```

### icon

Generate app icons:

```bash
nanobanana icon "settings gear" --style=minimal
nanobanana icon "chat bubble" --type=favicon --sizes=16,32,64
nanobanana icon "rocket" --style=flat --background=transparent
```

Options: `--type` (app-icon, favicon, ui-element), `--style` (flat, minimal, modern, skeuomorphic), `--sizes`, `--background`, `--corners`

### pattern

Create seamless patterns and textures:

```bash
nanobanana pattern "hexagons" --style=geometric --colors=duotone
nanobanana pattern "marble texture" --type=texture
nanobanana pattern "subtle gradient" --type=wallpaper --density=sparse
```

Options: `--type` (seamless, texture, wallpaper), `--style` (geometric, organic, abstract, floral, tech), `--density`, `--colors`

### diagram

Generate technical diagrams:

```bash
nanobanana diagram "user authentication flow" --type=flowchart
nanobanana diagram "microservices architecture" --type=architecture
nanobanana diagram "database schema" --type=database
```

Options: `--type` (flowchart, architecture, network, database, wireframe, mindmap, sequence), `--style`, `--layout`, `--complexity`

### story

Create sequential images for tutorials or narratives:

```bash
nanobanana story "seed growing into tree" --steps=5 --type=process
nanobanana story "making coffee" --type=tutorial --steps=6
nanobanana story "day to night transition" --type=timeline --steps=4
```

Options: `--steps` (2-8), `--type` (story, process, tutorial, timeline), `--style`, `--transition`

### tips

Get prompting help:

```bash
nanobanana tips              # general tips
nanobanana tips generate     # tips for generate command
nanobanana tips diagram      # tips for diagram command
```

## Output

All images save to `./nanobanana-output/` in the current directory. Filenames are based on the prompt.

## Prompting

The difference between a useless image and a useful one is usually the prompt.

**Be specific.** "Create a logo" gives you something generic. "Minimalist logo for a fintech company, geometric shapes, blue and white, suitable for favicon" gets you something you can actually use.

**Include context.** What's the image for? Who's the audience? Are there brand colors to match? Technical requirements like transparency or specific dimensions?

**Reference styles.** "In the style of Material Design" or "Similar to iOS app icons" or "Inspired by mid-century modern illustration" helps the model understand what you're after.

**Generate variations.** Run with `--count=4` and pick the best one, then iterate. First attempts rarely nail it.

## When to use what

**generate** - Most flexible. Use when other commands don't fit or when you want full control over the prompt.

**edit** - When you have an image and want to change specific parts without regenerating the whole thing. Good for adding/removing objects, changing colors, swapping backgrounds.

**restore** - Old family photos, damaged images, low-quality scans. Knows to preserve the original feel while fixing problems.

**icon** - App icons, favicons, UI elements. Adds the right styling and sizing automatically.

**pattern** - Backgrounds, textures, materials. Makes them tileable by default.

**diagram** - Technical illustrations. Handles the layout and labeling conventions for different diagram types.

**story** - Multi-step visuals. Maintains consistency across frames.

## Limitations

Generated images have an invisible SynthID watermark for provenance tracking.

Safety filters block some content.

Complex scenes with many specific elements may need multiple attempts.

The model interprets prompts. If the result isn't what you wanted, try rephrasing rather than adding more words.
