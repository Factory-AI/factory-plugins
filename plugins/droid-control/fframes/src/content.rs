//! The clip layer: window chrome around each clip, the panel entrances, directed zooms and
//! spotlights. Everything here runs on the clip clock (frames since the first clip frame).

use crate::clips::Clip;
use crate::composite::{self, Blend, Pixels};
use crate::fonts::Font;
use crate::motion::{Bezier, interpolate_eased};
use crate::props::{Effect, Layout, ObjectFit, Region};
use crate::raster;
use crate::svg::{Anchor, Card, Rect, Rgba, Shadow, Similarity, TextStyle, faded, rounded_rect_d};
use crate::theme::{Bar, BarSide, Palette, PresetConfig};
use fframes::usvgr::PreloadedImageData;
use fframes::{FFramesContext, FFramesSyncedVideoFrame, Frame, Svgr, SyncVideoFrameInput, svgr};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// An image and the whole screen pixels it is drawn 1:1 at.
type Layer = (Arc<PreloadedImageData>, Rect);

/// What a window showed last, so that a window or clip holding still is drawn once.
#[derive(Default)]
struct Shown {
    /// The chrome (see `Stage::chrome`) and the transform it was drawn by.
    chrome: Option<(Similarity, Layer)>,
    /// The clip frame: as decoded, and as drawn (the `crop` of it resampled to the screen).
    clip: Option<(Arc<PreloadedImageData>, Rect, Layer)>,
}

thread_local! {
    /// By window id.
    static SHOWN: RefCell<HashMap<String, Shown>> = RefCell::default();
}

const BAR_HEIGHT: f32 = 36.;
const SIDE_BY_SIDE_GAP: f32 = 16.;

const WARM_SHADOWS: [Shadow; 3] = [
    Shadow::new(8., 60., Rgba(30, 12, 4, 0.6)),
    Shadow::new(2., 20., Rgba(60, 24, 8, 0.3)),
    Shadow::new(0., 120., Rgba(238, 96, 24, 0.07)),
];
const COOL_SHADOWS: [Shadow; 2] = [Shadow::new(8., 40., Rgba(0, 0, 0, 0.45)), Shadow::new(2., 12., Rgba(0, 0, 0, 0.3))];

/// What the clip layer draws from: the composition's look and its clip sources.
pub struct Stage<'s> {
    pub palette: &'s Palette,
    pub config: &'s PresetConfig,
    pub size: (f32, f32),
    pub layout: Layout,
    pub clips: &'s [Clip],
    pub labels: &'s [String],
    pub window_title: &'s str,
    pub speed: f64,
    pub object_fit: ObjectFit,
    pub effects: &'s [Effect],
}

impl Stage<'_> {
    /// Composites the windows with their clips at `frame` on the clip clock, inside every
    /// zoom, and the spotlights over them onto `canvas`.
    pub fn compose(&self, canvas: &mut Pixels, frame: usize, video: &Frame, ctx: &FFramesContext<'_, '_>) {
        let (w, h) = self.size;
        let m = self.config.margin;
        let f = frame as f32;
        let entrance = |delay: f32| interpolate_eased(f, &[delay, delay + 15.], &[0., 1.], Bezier::EXPO_OUT.f());
        let zoom = self.effects.iter().fold(Similarity::IDENTITY, |zoom, effect| match effect {
            Effect::Zoom { t, dur, to } => {
                let (cx, cy) = ((to.x.0 + to.w.0 / 2.) * w, (to.y.0 + to.h.0 / 2.) * h);
                zoom.then(Similarity::scale_about(zoom_scale(f, *t, *dur, to), cx, cy))
            }
            _ => zoom,
        });
        match self.layout {
            Layout::Single => {
                if let Some(clip) = self.clips.first() {
                    let inner = Rect::new(m, m, w - 2. * m, h - 2. * m);
                    let progress = entrance(0.);
                    let place = Similarity::scale_about(0.92 + 0.08 * progress, inner.cx(), inner.cy());
                    let title = (!self.window_title.is_empty()).then_some(self.window_title);
                    let window = Window { id: "window0", inner, title, to_screen: place.then(zoom), opacity: progress };
                    self.window(canvas, &window, clip, frame, video, ctx);
                }
            }
            Layout::SideBySide => {
                let shown = &self.clips[..self.clips.len().min(2)];
                let panel_w = ((w - 2. * m - SIDE_BY_SIDE_GAP) / 2.).floor();
                let panel_h = h - 2. * m;
                // Panels are content-box: the 1px border sits outside the panel size.
                let row = shown.len() as f32 * (panel_w + 2.) + (shown.len().max(1) - 1) as f32 * SIDE_BY_SIDE_GAP;
                let left = (w - row) / 2.;
                for (i, clip) in shown.iter().enumerate() {
                    let inner = Rect::new(
                        left + i as f32 * (panel_w + 2. + SIDE_BY_SIDE_GAP) + 1.,
                        (h - panel_h) / 2.,
                        panel_w,
                        panel_h,
                    );
                    let progress = entrance(i as f32 * 6.);
                    let place = Similarity::translate(0., 12. * (1. - progress)).then(Similarity::scale_about(
                        0.92 + 0.08 * progress,
                        inner.cx(),
                        inner.cy(),
                    ));
                    let label = self.labels.get(i).cloned().unwrap_or_else(|| format!("Clip {}", i + 1));
                    let id = format!("window{i}");
                    let window =
                        Window { id: &id, inner, title: Some(&label), to_screen: place.then(zoom), opacity: progress };
                    self.window(canvas, &window, clip, frame, video, ctx);
                }
            }
        }
        canvas.draw(self.spotlights(frame));
    }

    /// Dimmed frame with a rounded cutout around each active spotlight region.
    pub fn spotlights<'a>(&self, frame: usize) -> Svgr<'a> {
        let (w, h) = self.size;
        let f = frame as f32;
        self.effects
            .iter()
            .filter_map(|effect| match effect {
                Effect::Spotlight { t, dur, on, dim } => {
                    let (start, end) = (t * 30., (t + dur) * 30.);
                    if f < start || f > end {
                        return None;
                    }
                    let local = f - start;
                    let total = end - start;
                    let fade_in = interpolate_eased(local, &[0., 9.], &[0., 1.], Bezier::EXPO_OUT.f());
                    let fade_out = crate::motion::interpolate(local, &[total - 9., total], &[1., 0.]);
                    let hole = Rect::new(on.x.0 * w, on.y.0 * h, on.w.0 * w, on.h.0 * h);
                    let d = format!("M0 0H{w}V{h}H0Z{}", rounded_rect_d(hole, 8.));
                    Some(faded(
                        fade_in.min(fade_out),
                        svgr!(<path d={d} fill-rule="evenodd" fill={Rgba::BLACK.alpha(*dim).to_string()} />),
                    ))
                }
                _ => None,
            })
            .collect()
    }

    /// The clip frame shown `frame` frames after playback started (`speed` source frames per
    /// output frame, holding the last frame once the clip ends), fitted into `window` as drawn
    /// on screen: its visible part resampled to the whole screen pixels it
    /// covers inside the content box, and those pixels. A partly covered edge pixel keeps the
    /// window fill. A frame identical to the last one the window showed at the same place
    /// keeps its image, so a screen recording holding still is resampled once.
    fn clip_frame(
        &self,
        window: &Window<'_>,
        clip: &Clip,
        frame: usize,
        video: &Frame,
        ctx: &FFramesContext<'_, '_>,
    ) -> Option<Layer> {
        let &Window { id, inner, to_screen, .. } = window;
        let mut at = video.clone();
        at.index = ((frame as f64 * self.speed + 1e-9).floor() as usize).min(clip.last_frame);
        let input = SyncVideoFrameInput { start_from: 0., looping: false, editor_fallback_image: None };
        let decoded = at.get_synced_video_frame(ctx, &clip.name, &input)?.into_image().href();
        let size = (decoded.width as f32, decoded.height as f32);
        let content = self.content_box(inner);
        let (w, h) = self.size;
        let bounds = to_screen.apply(content).intersect(Rect::new(0., 0., w, h));
        let (at, crop) = raster::coverage(size, to_screen.apply(fitted(size, content, self.object_fit)), bounds)?;
        SHOWN.with_borrow_mut(|shown| {
            let last = &mut shown.entry(id.to_owned()).or_default().clip;
            if let Some((last, last_crop, layer)) = last
                && (layer.1, *last_crop) == (at, crop)
                && (last.width, last.height, &last.data) == (decoded.width, decoded.height, &decoded.data)
            {
                return Some(layer.clone());
            }
            let size = (decoded.width, decoded.height);
            let image = composite::resample(&decoded.data, size, crop, (at.w as u32, at.h as u32)).into_image();
            Some(last.insert((decoded, crop, (image, at))).2.clone())
        })
    }

    /// A window's chrome, everything but its clip: the card's shadows and its body with `bar`,
    /// composited into one layer over the screen pixels they cover. A window that has not
    /// moved since it was last drawn keeps its chrome.
    fn chrome<'a>(&self, card: &Card<'_>, bar: impl FnOnce() -> Svgr<'a>) -> Layer {
        SHOWN.with_borrow_mut(|shown| {
            let last = &mut shown.entry(card.id.to_owned()).or_default().chrome;
            if let Some((to_screen, layer)) = last
                && *to_screen == card.to_screen
            {
                return layer.clone();
            }
            let (w, h) = self.size;
            let shadows = card.shadows();
            // The shadows' pixels take in the whole card.
            let at = shadows.as_ref().map_or_else(
                || card.to_screen.apply(card.rect).whole_pixels().intersect(Rect::new(0., 0., w, h)),
                |(_, at)| *at,
            );
            let mut layer = Pixels::transparent(at.w as u32, at.h as u32);
            if let Some((shadows, from)) = shadows {
                layer.blend(&shadows, ((from.x - at.x) as u32, (from.y - at.y) as u32), 1., Blend::Over);
            }
            let place = card.to_screen.then(Similarity::translate(-at.x, -at.y));
            layer.draw(svgr!(<g transform={place.attr()}>{card.body(bar())}</g>));
            last.insert((card.to_screen, (layer.into_image(), at))).1.clone()
        })
    }

    /// Where a window's clip goes: `inner` below the title bar, inset by the padding.
    fn content_box(&self, inner: Rect) -> Rect {
        let PresetConfig { bar, padding, .. } = *self.config;
        let bar_height = if bar == Bar::None { 0. } else { BAR_HEIGHT };
        Rect::new(
            inner.x + padding,
            inner.y + bar_height + padding,
            inner.w - 2. * padding,
            inner.h - bar_height - 2. * padding,
        )
    }

    /// A window and its clip, composited onto `canvas` from their screen pixels. The clip lies
    /// inside the content box, clear of the bar, the border and the rounded corners (every
    /// preset pads it by more than the corners cut in), so it can go on last.
    fn window(
        &self,
        canvas: &mut Pixels,
        window: &Window<'_>,
        clip: &Clip,
        frame: usize,
        video: &Frame,
        ctx: &FFramesContext<'_, '_>,
    ) {
        let &Window { id, inner, title, to_screen, opacity } = window;
        if opacity <= 0. {
            return;
        }
        let PresetConfig { bar, bar_side, radius, shadow, .. } = *self.config;
        let palette = self.palette;
        let shadows: &[Shadow] = match (shadow, palette.warm) {
            (false, _) => &[],
            (true, true) => &WARM_SHADOWS,
            (true, false) => &COOL_SHADOWS,
        };

        let window_bar = || {
            if bar == Bar::None {
                Svgr::empty()
            } else {
                let cy = inner.y + BAR_HEIGHT / 2.;
                let (first, step) = match bar_side {
                    BarSide::Left => (inner.x + 8. + 14., 20.),
                    BarSide::Right => (inner.right() - 8. - 80. + 62., -20.),
                };
                let dots: Svgr = ["#ff5f57", "#febc2e", "#28c840"]
                    .into_iter()
                    .enumerate()
                    .map(|(i, color)| {
                        let cx = first + i as f32 * step;
                        match bar {
                            Bar::Rings => {
                                svgr!(<circle cx={cx} cy={cy} r="6" fill="none" stroke={color} stroke-width="1.5" />)
                            }
                            _ => svgr!(<circle cx={cx} cy={cy} r="6" fill={color} />),
                        }
                    })
                    .collect();
                let label = title.map_or_else(Svgr::empty, |title| {
                    let style = TextStyle::new(Font::sans(400), 13.);
                    style.draw(
                        title.to_owned(),
                        inner.cx(),
                        style.baseline(inner.y, BAR_HEIGHT),
                        Rgba::hex(palette.muted),
                        Anchor::Middle,
                    )
                });
                svgr!(<g>{dots}{label}</g>)
            }
        };

        let card = Card {
            id,
            rect: inner.outset(1.),
            radius,
            fill: Rgba::hex(palette.surface),
            border: Rgba::WHITE.alpha(0.06),
            accent: None,
            shadows,
            frost: None,
            to_screen,
            screen: self.size,
        };
        let chrome = self.chrome(&card, window_bar);
        let video = self.clip_frame(window, clip, frame, video, ctx);
        let paint = |layer: &mut Pixels| {
            for (image, at) in [Some(&chrome), video.as_ref()].into_iter().flatten() {
                layer.blend(image, (at.x as u32, at.y as u32), 1., Blend::Over);
            }
        };
        if opacity >= 1. {
            paint(canvas);
        } else {
            // Faded as a whole, like a CSS opacity group.
            let mut layer = Pixels::transparent(canvas.width, canvas.height);
            paint(&mut layer);
            canvas.blend(&layer.into_image(), (0, 0), opacity, Blend::Over);
        }
    }
}

/// A window as laid out at a frame: its content rect (the 1px border lies outside it), bar
/// title, where it is drawn on screen and how faded in it is.
struct Window<'s> {
    id: &'s str,
    inner: Rect,
    title: Option<&'s str>,
    to_screen: Similarity,
    opacity: f32,
}

/// Where an image of `size` lands in `content` under `fit`, like `preserveAspectRatio`.
fn fitted((iw, ih): (f32, f32), content: Rect, fit: ObjectFit) -> Rect {
    let (sx, sy) = (content.w / iw, content.h / ih);
    let s = match fit {
        ObjectFit::Contain => sx.min(sy),
        ObjectFit::Cover => sx.max(sy),
        ObjectFit::Fill => return content,
    };
    Rect::new(content.cx() - iw * s / 2., content.cy() - ih * s / 2., iw * s, ih * s)
}

/// Directed zoom into `to`: ease in over the first 30%, hold, ease out over the last 30%.
fn zoom_scale(f: f32, t: f32, dur: f32, to: &Region) -> f32 {
    let (start, end) = (t * 30., (t + dur) * 30.);
    let (in_end, hold_end) = (start + (end - start) * 0.3, start + (end - start) * 0.7);
    let ease = Bezier(0.25, 0.1, 0.25, 1.);
    let progress = if f < start || f > end {
        0.
    } else if f <= in_end {
        interpolate_eased(f, &[start, in_end], &[0., 1.], ease.f())
    } else if f <= hold_end {
        1.
    } else {
        interpolate_eased(f, &[hold_end, end], &[1., 0.], ease.f())
    };
    let target = (1. / to.w.0.max(0.01)).min(1. / to.h.0.max(0.01));
    1. + (target - 1.) * progress
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::props::Pct;

    #[test]
    fn zoom_fills_the_frame_with_the_region_during_the_hold() {
        let to = Region { x: Pct(0.25), y: Pct(0.25), w: Pct(0.5), h: Pct(0.25) };
        assert_eq!(zoom_scale(0., 1., 2., &to), 1.);
        assert!(zoom_scale(40., 1., 2., &to) > 1.);
        assert_eq!(zoom_scale(48., 1., 2., &to), 2.);
        assert_eq!(zoom_scale(60., 1., 2., &to), 2.);
        assert_eq!(zoom_scale(91., 1., 2., &to), 1.);
    }
}
