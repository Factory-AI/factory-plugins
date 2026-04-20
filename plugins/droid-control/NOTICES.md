# Third-Party Notices

This plugin depends on several third-party tools and libraries. They are not bundled -- each is installed separately by the user. Their respective licenses apply at the point of installation and use.

## Video rendering

- **[Remotion](https://www.remotion.dev/)** -- React-based video renderer used by the compose/showcase pipeline. Remotion is free for individuals, small teams (<=3 employees), and non-profits. Larger companies require a [company license](https://www.remotion.pro/). See the [full license terms](https://github.com/remotion-dev/remotion/blob/main/LICENSE.md).
- **[React](https://react.dev/)** -- MIT License
- **[Zod](https://zod.dev/)** -- MIT License
- **[prism-react-renderer](https://github.com/FormidableLabs/prism-react-renderer)** -- MIT License. Powers the syntax highlighting in the `CodeAnnotationOverlay` component.

## Terminal automation

- **[tuistory](https://github.com/nicholasgasior/tuistory)** -- virtual PTY automation CLI
- **[asciinema](https://asciinema.org/)** -- terminal session recorder (GPL-3.0)
- **[agg](https://github.com/asciinema/agg)** -- asciinema GIF generator (Apache-2.0)

## Browser automation

- **[agent-browser](https://docs.factory.ai/)** -- Playwright-backed browser automation CLI

## System tools

- **[ffmpeg](https://ffmpeg.org/)** -- multimedia framework (LGPL-2.1+ / GPL-2.0+, depending on build configuration)
- **[cage](https://github.com/cage-kiosk/cage)** -- Wayland kiosk compositor (MIT)
- **[wtype](https://github.com/atx/wtype)** -- Wayland keystroke injection (MIT)

## Design influences

- **[@hyperframes/shader-transitions](https://github.com/heygen-com/hyperframes/tree/main/packages/shader-transitions)** (Apache-2.0) -- the `transitionStyle` prop's naming and taxonomy (`whip-pan`, `light-leak`, `flash`, `glitch-lite`) was shaped by Hyperframes' shader-transitions catalog. Implementations in `ShowcaseTransition.tsx` are original Remotion-native CSS/SVG overlays, not GLSL ports.
