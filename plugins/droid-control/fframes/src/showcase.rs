//! The Showcase composition: background and particles under a title card, the clip content
//! with its overlays and the Droid outro (joined by two transitions), then the watermark,
//! film grain and colour grade on top.

use crate::clips::{self, Clip};
use crate::content::Stage;
use crate::outro;
use crate::overlays::{CodeCard, Overlays};
use crate::props::{Effect, ShowcaseProps};
use crate::scenery;
use crate::theme::{Palette, PresetConfig};
use crate::timing::{self, TITLE_FRAMES, TRANSITION_FRAMES, Timeline};
use crate::title::{TitleCard, TitleLayout};
use crate::transition::{Direction, Transition};
use fframes::media::ImageData;
use fframes::{AudioMap, Duration, FFramesContext, Frame, Svgr, Video, svgr};
use std::collections::HashSet;

const HALFTONE: &[u8] = include_bytes!("../media/bg-halftone-rotor.png");

pub struct Showcase {
    props: ShowcaseProps,
    clips: Vec<Clip>,
    timeline: Timeline,
    palette: &'static Palette,
    config: &'static PresetConfig,
    size: (f32, f32),
    halftone: ImageData<'static>,
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
        let halftone = ImageData::new_from_bytes("bg-halftone-rotor.png", HALFTONE)
            .map_err(|err| format!("could not decode the embedded halftone texture: {err:?}"))?;
        let code_cards = props.code_annotations.iter().map(|a| CodeCard::new(a, palette, config, size)).collect();
        Ok(Self {
            timeline: Timeline { content: timing::content_frames(clips::longest(&clips), props.speed) },
            title: TitleLayout::new(&props.title, &props.subtitle, size.0),
            props,
            clips,
            palette,
            config,
            size,
            halftone,
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

    fn background<'a>(&self, id: &str, frame: usize, total: usize) -> Svgr<'a> {
        scenery::background(id, self.palette, self.config, &self.halftone, self.size, frame, total)
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
            let content = self.content(local, frame, ctx);
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
    fn content<'a>(&'a self, local: usize, frame: &Frame, ctx: &FFramesContext<'_, '_>) -> Svgr<'a> {
        let sequence = self.timeline.content_sequence();
        let background = self.background("content-bg", local, sequence);
        let Some(clock) = local.checked_sub(TRANSITION_FRAMES) else {
            return background;
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
        let backdrop: Svgr =
            [background, stage.layout(clock, frame, ctx), stage.spotlights(clock)].into_iter().collect();
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
        [backdrop, drawn].into_iter().collect()
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

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let f = frame.index;
        let (w, h) = self.size;
        let treatment = self.props.fidelity.treatment();
        let watermark = if self.props.preset.is_factory() { scenery::watermark(self.size, f) } else { Svgr::empty() };
        svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" viewBox={format!("0 0 {w} {h}")} width={w} height={h}>
                {self.background("bg", f, self.timeline.total())}
                {scenery::particles(self.palette.accent, self.size, f)}
                {self.series(&frame, ctx)}
                {watermark}
                {scenery::noise(self.size, treatment.noise_opacity)}
                {scenery::grade(self.palette, self.size, treatment.grade_intensity)}
            </svg>
        )
    }
}
