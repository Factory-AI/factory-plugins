//! The clip layer: window chrome around each clip, the panel entrances, directed zooms and
//! spotlights. Everything here runs on the clip clock (frames since the first clip frame).

use crate::clips::Clip;
use crate::fonts::Font;
use crate::motion::{Bezier, interpolate_eased};
use crate::props::{Effect, Layout, ObjectFit, Region};
use crate::svg::{Anchor, Card, Rect, Rgba, Shadow, Similarity, TextStyle, faded, rounded_rect_d};
use crate::theme::{Bar, BarSide, Palette, PresetConfig};
use fframes::usvgr::PreloadedImageData;
use fframes::{FFramesContext, FFramesSyncedVideoFrame, Frame, Svgr, SyncVideoFrameInput, svgr};
use std::sync::Arc;

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
    /// The windows with their clips at `frame` on the clip clock, inside every zoom.
    pub fn layout<'a>(&self, frame: usize, video: &Frame, ctx: &FFramesContext<'_, '_>) -> Svgr<'a> {
        let (w, h) = self.size;
        let m = self.config.margin;
        let f = frame as f32;
        let entrance = |delay: f32| interpolate_eased(f, &[delay, delay + 15.], &[0., 1.], Bezier::EXPO_OUT.f());
        let windows: Svgr = match self.layout {
            Layout::Single => self.clips.first().map_or_else(Svgr::empty, |clip| {
                let inner = Rect::new(m, m, w - 2. * m, h - 2. * m);
                let progress = entrance(0.);
                let place = Similarity::scale_about(0.92 + 0.08 * progress, inner.cx(), inner.cy());
                let title = (!self.window_title.is_empty()).then_some(self.window_title);
                faded(progress, self.window("window0", inner, title, self.frame_of(clip, frame, video, ctx), place))
            }),
            Layout::SideBySide => {
                let shown = &self.clips[..self.clips.len().min(2)];
                let panel_w = ((w - 2. * m - SIDE_BY_SIDE_GAP) / 2.).floor();
                let panel_h = h - 2. * m;
                // Panels are content-box: the 1px border sits outside the panel size.
                let row = shown.len() as f32 * (panel_w + 2.) + (shown.len().max(1) - 1) as f32 * SIDE_BY_SIDE_GAP;
                let left = (w - row) / 2.;
                shown
                    .iter()
                    .enumerate()
                    .map(|(i, clip)| {
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
                        let image = self.frame_of(clip, frame, video, ctx);
                        faded(progress, self.window(&format!("window{i}"), inner, Some(&label), image, place))
                    })
                    .collect()
            }
        };
        self.effects.iter().fold(windows, |content, effect| match effect {
            Effect::Zoom { t, dur, to } => {
                let scale = zoom_scale(f, *t, *dur, to);
                let origin = (to.x.0 + to.w.0 / 2.) * w;
                let place = Similarity::scale_about(scale, origin, (to.y.0 + to.h.0 / 2.) * h);
                svgr!(<g transform={place.attr()}>{content}</g>)
            }
            _ => content,
        })
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

    /// The clip frame shown `frame` frames after playback started: `speed` source frames per
    /// output frame, holding the last frame once the clip ends.
    fn frame_of(
        &self,
        clip: &Clip,
        frame: usize,
        video: &Frame,
        ctx: &FFramesContext<'_, '_>,
    ) -> Option<Arc<PreloadedImageData>> {
        let mut at = video.clone();
        at.index = ((frame as f64 * self.speed + 1e-9).floor() as usize).min(clip.last_frame);
        let input = SyncVideoFrameInput { start_from: 0., looping: false, editor_fallback_image: None };
        at.get_synced_video_frame(ctx, &clip.name, &input).map(|decoded| decoded.into_image().href())
    }

    /// Window chrome around `inner` (the content size; the 1px border lies outside it).
    fn window<'a>(
        &self,
        id: &str,
        inner: Rect,
        title: Option<&str>,
        image: Option<Arc<PreloadedImageData>>,
        place: Similarity,
    ) -> Svgr<'a> {
        let PresetConfig { bar, bar_side, radius, padding, shadow, .. } = *self.config;
        let palette = self.palette;
        let bar_height = if bar == Bar::None { 0. } else { BAR_HEIGHT };
        let shadows: &[Shadow] = match (shadow, palette.warm) {
            (false, _) => &[],
            (true, true) => &WARM_SHADOWS,
            (true, false) => &COOL_SHADOWS,
        };

        let window_bar = if bar == Bar::None {
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
        };

        let content = Rect::new(
            inner.x + padding,
            inner.y + bar_height + padding,
            inner.w - 2. * padding,
            inner.h - bar_height - 2. * padding,
        );
        let clip_id = format!("{id}-content");
        let clip_url = format!("url(#{clip_id})");
        let aspect = match self.object_fit {
            ObjectFit::Contain => "xMidYMid meet",
            ObjectFit::Cover => "xMidYMid slice",
            ObjectFit::Fill => "none",
        };
        let video = image.map_or_else(Svgr::empty, |href| {
            svgr!(<image href={href} x={content.x} y={content.y} width={content.w} height={content.h} preserveAspectRatio={aspect} />)
        });

        let card = Card {
            id,
            rect: inner.outset(1.),
            radius,
            fill: Rgba::hex(palette.surface),
            border: Rgba::WHITE.alpha(0.06),
            accent: None,
            shadows,
            frost: None,
            to_screen: place,
        };
        svgr!(
            <g transform={place.attr()}>
                {card.draw(svgr!(
                    <g>
                        {window_bar}
                        <clipPath id={clip_id}>
                            <rect x={content.x} y={content.y} width={content.w} height={content.h} />
                        </clipPath>
                        <g clip-path={clip_url}>{video}</g>
                    </g>
                ))}
            </g>
        )
    }
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
