# Third-Party Notices

This plugin depends on several third-party tools and libraries. Executables are installed separately by the user.

## Desktop automation documentation

- **[Cua Driver](https://github.com/trycua/cua)** — desktop-use guidance is adapted from Cua under the [included MIT license](CUA-LICENSE.md). Driver code and binaries are not bundled.

## Video rendering

- **[fframes](https://github.com/dmtrKovalenko/fframes)** (MIT) -- Rust SVG-based video framework that renders the `fframes/` showcase composition. Its SVG stack, [svgr/usvgr](https://github.com/dmtrKovalenko/fframes), is MPL-2.0; its clip decoder statically links [FFmpeg](https://ffmpeg.org/) (LGPL-2.1+) through `ffmpeg-sys-fframes` (WTFPL).
- **[Geist and Geist Mono](https://github.com/vercel/geist-font)** -- SIL Open Font License 1.1, embedded in the renderer from `fframes/media/` (license text in `fframes/media/GEIST-OFL.txt`).
- **[syntect](https://github.com/trishume/syntect)** (MIT) and **[two-face](https://github.com/CosmicHorrorDev/two-face)** (MIT OR Apache-2.0) -- syntax highlighting for code annotations.
- **[clap](https://github.com/clap-rs/clap)**, **[serde](https://serde.rs/)**, **[signal-hook](https://github.com/vorner/signal-hook)**, **[tempfile](https://github.com/Stebalien/tempfile)** -- MIT OR Apache-2.0.

## Terminal automation

- **[tuistory](https://github.com/nicholasgasior/tuistory)** -- virtual PTY automation CLI
- **[asciinema](https://asciinema.org/)** -- terminal session recorder (GPL-3.0)
- **[agg](https://github.com/asciinema/agg)** -- asciinema GIF generator (Apache-2.0)

## Browser automation

- **[agent-browser](https://docs.factory.ai/)** -- CDP browser automation CLI

## System tools

- **[ffmpeg](https://ffmpeg.org/)** -- multimedia framework (LGPL-2.1+ / GPL-2.0+, depending on build configuration)
- **[cage](https://github.com/cage-kiosk/cage)** -- Wayland kiosk compositor (MIT)
- **[wtype](https://github.com/atx/wtype)** -- Wayland keystroke injection (MIT)

## Design influences

- **[@hyperframes/shader-transitions](https://github.com/heygen-com/hyperframes/tree/main/packages/shader-transitions)** (Apache-2.0) -- the `transitionStyle` prop's naming and taxonomy (`whip-pan`, `light-leak`, `flash`, `glitch-lite`) was shaped by Hyperframes' shader-transitions catalog. Implementations in `fframes/src/transition.rs` are original SVG filter and overlay effects, not GLSL ports.
