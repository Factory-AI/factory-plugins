//! The Showcase composition: background and particles under a title card, the clip content
//! with its overlays and the Droid outro (joined by two transitions), then the watermark,
//! and the film grain and colour grade. Each frame is composited in layers (see `raster`).

use crate::clips::{self, Clip};
use crate::composite::Pixels;
use crate::content::Stage;
use crate::outro;
use crate::overlays::{CodeCard, Overlays};
use crate::props::{Effect, ShowcaseProps};
use crate::raster::{self, Backdrop};
use crate::scenery::{self, Scenery};
use crate::theme::{Palette, PresetConfig};
use crate::timing::{self, TITLE_FRAMES, TRANSITION_FRAMES, Timeline};
use crate::title::{TitleCard, TitleLayout};
use crate::transition::{Direction, Transition};
use fframes::{AudioMap, Duration, FFramesContext, Frame, Svgr, Video, svgr};
use std::collections::HashSet;

pub struct Showcase {
    props: ShowcaseProps,
    clips: Vec<Clip>,
    timeline: Timeline,
    palette: &'static Palette,
    config: &'static PresetConfig,
    size: (f32, f32),
    scenery: Scenery,
    title: TitleLayout,
    code_cards: Vec<CodeCard>,
}

impl Showcase {
    /// `clips` are probed sources; the longest one sets the content length.
    pub fn new(props: ShowcaseProps, clips: Vec<Clip>) -> Result<Self, String> {
        let mut warned = HashSet::new();
        for name in props.effects.iter().filter_map(Effect::unrendered_name) {
            if warned.insert(name) {
                eprintln!("Showcase: effect fx='{name}' is schema-valid but not rendered by this composition");
            }
        }
        let palette = props.preset.palette();
        let config = props.preset.config();
        let size = (props.width as f32, props.height as f32);
        let scenery = Scenery::new(palette, config, props.fidelity.treatment(), size)?;
        let code_cards = props.code_annotations.iter().map(|a| CodeCard::new(a, palette, config, size)).collect();
        Ok(Self {
            timeline: Timeline { content: timing::content_frames(clips::longest(&clips), props.speed) },
            title: TitleLayout::new(&props.title, &props.subtitle, size.0),
            props,
            clips,
            palette,
            config,
            size,
            scenery,
            code_cards,
        })
    }

    pub fn props(&self) -> &ShowcaseProps {
        &self.props
    }

    pub fn clips(&self) -> &[Clip] {
        &self.clips
    }

    pub fn timeline(&self) -> Timeline {
        self.timeline
    }

    /// The finished frame at `frame.index`, opaque. Between its transitions the content
    /// paints its own background over the whole frame; around them the background and
    /// particles show.
    fn frame(&self, frame: &Frame, ctx: &FFramesContext<'_, '_>) -> Pixels {
        let f = frame.index;
        let timeline = self.timeline;
        let mut canvas = if (timeline.clips_start()..timeline.outro_start()).contains(&f) {
            self.content(f - timeline.content_start(), frame, ctx)
        } else {
            let mut canvas = Pixels::of(&self.scenery.background(f, timeline.total()));
            canvas.draw(svgr!(<g>{scenery::particles(self.palette.accent, self.size, f)}{self.series(frame, ctx)}</g>));
            canvas
        };
        if self.props.preset.is_factory() {
            canvas.draw(scenery::watermark(self.size, f));
        }
        self.scenery.grade(&mut canvas);
        canvas
    }

    /// Title, content and outro with their transitions at global `frame.index`.
    fn series<'a>(&'a self, frame: &Frame, ctx: &FFramesContext<'_, '_>) -> Svgr<'a> {
        let f = frame.index;
        let (content_start, outro_start) = (self.timeline.content_start(), self.timeline.outro_start());
        let transition = Transition { style: self.props.transition_style, warm: self.palette.warm, size: self.size };
        let progress = |since: usize| since as f32 / TRANSITION_FRAMES as f32;
        let mut scenes = Vec::with_capacity(2);
        if f < TITLE_FRAMES {
            let props = &self.props;
            let card = TitleCard {
                layout: &self.title,
                title: &props.title,
                subtitle: &props.subtitle,
                speed_note: &props.speed_note,
                palette: self.palette,
                size: self.size,
            };
            let title = card.draw(f);
            scenes.push(match f.checked_sub(content_start) {
                Some(since) => transition.present(Direction::Exiting, progress(since), title),
                None => title,
            });
        }
        if (content_start..outro_start + TRANSITION_FRAMES).contains(&f) {
            let local = f - content_start;
            let content = raster::image(self.content(local, frame, ctx).into_image());
            scenes.push(if local < TRANSITION_FRAMES {
                transition.present(Direction::Entering, progress(local), content)
            } else if f >= outro_start {
                transition.present(Direction::Exiting, progress(f - outro_start), content)
            } else {
                content
            });
        }
        if let Some(local) = f.checked_sub(outro_start) {
            let outro = outro::draw(self.palette, self.size, local);
            scenes.push(if local < TRANSITION_FRAMES {
                transition.present(Direction::Entering, progress(local), outro)
            } else {
                outro
            });
        }
        scenes.into_iter().collect()
    }

    /// The content segment at `local` frames into it. The clips and every overlay start one
    /// transition in, so overlay times are seconds from the first clip frame.
    fn content(&self, local: usize, frame: &Frame, ctx: &FFramesContext<'_, '_>) -> Pixels {
        let sequence = self.timeline.content_sequence();
        let mut scene = Pixels::of(&self.scenery.background(local, sequence));
        let Some(clock) = local.checked_sub(TRANSITION_FRAMES) else {
            return scene;
        };
        let props = &self.props;
        let stage = Stage {
            palette: self.palette,
            config: self.config,
            size: self.size,
            layout: props.layout,
            clips: &self.clips,
            labels: &props.labels,
            window_title: &props.window_title,
            speed: props.speed,
            object_fit: props.object_fit,
            effects: &props.effects,
        };
        stage.compose(&mut scene, clock, frame, ctx);
        let backdrop = Backdrop::new(scene);
        let overlays = Overlays { palette: self.palette, config: self.config, size: self.size, backdrop: &backdrop };
        let sweeps =
            if props.sections.len() > 1 { overlays.section_sweeps(&props.sections, clock) } else { Svgr::empty() };
        let drawn = svgr!(
            <g>
                {overlays.callouts(&props.effects, clock)}
                {sweeps}
                {overlays.section_headers(&props.sections, clock, sequence - TRANSITION_FRAMES)}
                {overlays.keystrokes(&props.keys, clock)}
                {overlays.code_cards(&self.code_cards, clock)}
            </g>
        );
        let mut content = backdrop.into_scene();
        content.draw(drawn);
        content
    }
}

impl Video for Showcase {
    const FPS: usize = timing::FPS;
    // Nominal: `render` rasterises every frame at the props size, whatever these say.
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;

    fn duration(&self) -> Duration<'_> {
        Duration::Frames(self.timeline.total())
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }

    /// The whole frame, composited and graded: one image, which `render` takes the pixels of.
    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let (w, h) = self.size;
        svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" viewBox={format!("0 0 {w} {h}")} width={w} height={h}>
                {raster::image(self.frame(&frame, ctx).into_image())}
            </svg>
        )
    }
}
