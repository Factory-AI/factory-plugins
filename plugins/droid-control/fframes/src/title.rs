//! The opening title card: tagline, title and subtitle rise in while the rotor sweeps an
//! accent line under the title (a gradient sweep for cool palettes), all inside a slow
//! breathing zoom.

use crate::fonts::Font;
use crate::motion::{Bezier, interpolate, interpolate_eased};
use crate::svg::{Anchor, Rect, Rgba, Similarity, TextStyle, faded, glow, rotor, stops};
use crate::theme::Palette;
use crate::timing::TITLE_FRAMES;
use fframes::{Svgr, svgr};

const TITLE_MAX_SIZE: f32 = 64.;
const SUBTITLE_MAX_SIZE: f32 = 24.;
const SWEEP_WIDTH: f32 = 280.;
const ROTOR_SIZE: f32 = 28.;

/// Title and subtitle sizes, fitted once to 80% and 70% of the width.
#[derive(Debug, Clone, Copy)]
pub struct TitleLayout {
    title: TextStyle,
    subtitle: TextStyle,
}

fn fit(text: &str, style: TextStyle, within: f32, max: f32) -> TextStyle {
    let at_100 = TextStyle { size: 100., ..style }.width(text);
    let size = if at_100 > 0. { (within * 100. / at_100).min(max) } else { max };
    TextStyle { size, ..style }
}

impl TitleLayout {
    pub fn new(title: &str, subtitle: &str, width: f32) -> Self {
        let subtitle = if subtitle.is_empty() { " " } else { subtitle };
        Self {
            title: fit(title, TextStyle::new(Font::sans(700), 0.).spaced(-0.04), width * 0.8, TITLE_MAX_SIZE),
            subtitle: fit(subtitle, TextStyle::new(Font::sans(400), 0.), width * 0.7, SUBTITLE_MAX_SIZE),
        }
    }
}

pub struct TitleCard<'s> {
    pub layout: &'s TitleLayout,
    pub title: &'s str,
    pub subtitle: &'s str,
    pub speed_note: &'s str,
    pub palette: &'s Palette,
    pub size: (f32, f32),
}

impl TitleCard<'_> {
    pub fn draw<'a>(&self, frame: usize) -> Svgr<'a> {
        let TitleCard { layout, title, subtitle, speed_note, palette, size: (w, h) } = *self;
        let f = frame as f32;
        let expo = || Bezier::EXPO_OUT.f();
        let tagline = interpolate_eased(f, &[0., 12.], &[0., 1.], expo());
        let entrance = interpolate_eased(f, &[0., 18.], &[0., 1.], expo());
        let rise = interpolate_eased(f, &[4.5, 22.5], &[0., 1.], expo());

        let note_style = TextStyle::new(Font::mono(400), 14.).spaced(0.08);
        let note = speed_note.to_uppercase();
        let note_height = if note.is_empty() { 0. } else { note_style.normal_line_height() + 16. };
        let title_height = layout.title.normal_line_height();
        let accent_height = if palette.warm { 16. + ROTOR_SIZE + 8. + 16. } else { 20. + 2. + 20. };
        let subtitle_height = if subtitle.is_empty() { 0. } else { layout.subtitle.normal_line_height() };
        let top = (h - note_height - title_height - accent_height - subtitle_height) / 2.;
        let title_top = top + note_height;
        let accent_top = title_top + title_height;
        let subtitle_top = accent_top + accent_height;
        let cx = w / 2.;

        let note = if note.is_empty() {
            Svgr::empty()
        } else {
            let text = note_style.draw(
                note,
                cx,
                note_style.baseline(top, note_style.normal_line_height()),
                Rgba::hex(palette.muted),
                Anchor::Middle,
            );
            faded(tagline, svgr!(<g transform={Similarity::translate(0., 15. * (1. - tagline)).attr()}>{text}</g>))
        };

        let accent = Rgba::hex(palette.accent);
        let title_baseline = layout.title.baseline(title_top, title_height);
        let title_text = |fill: Rgba| layout.title.draw(title.to_owned(), cx, title_baseline, fill, Anchor::Middle);
        let title_area =
            Rect::new(cx - layout.title.width(title) / 2., title_top, layout.title.width(title), title_height);
        let place_title = Similarity::scale_about(1.08 - 0.08 * entrance, cx, title_top + title_height / 2.)
            .then(Similarity::translate(0., 30. * (1. - entrance)));
        let title = faded(
            entrance,
            svgr!(
                <g transform={place_title.attr()}>
                    {glow("title-glow-far".to_owned(), 20., title_area, title_text(accent.a8(0x1a)))}
                    {glow("title-glow-near".to_owned(), 10., title_area, title_text(accent.a8(0x4d)))}
                    {title_text(Rgba::hex(palette.text))}
                </g>
            ),
        );

        let accent_line = if palette.warm {
            self.rotor_sweep(f, cx - SWEEP_WIDTH / 2., accent_top + 16. + (ROTOR_SIZE + 8.) / 2.)
        } else {
            Self::gradient_sweep(f, palette, cx, accent_top + 20.)
        };

        let subtitle = if subtitle.is_empty() {
            Svgr::empty()
        } else {
            let baseline = layout.subtitle.baseline(subtitle_top, subtitle_height);
            let text =
                layout.subtitle.draw(subtitle.to_owned(), cx, baseline, Rgba::hex(palette.muted), Anchor::Middle);
            faded(rise, svgr!(<g transform={Similarity::translate(0., 20. * (1. - rise)).attr()}>{text}</g>))
        };

        let breathing = 1. + 0.04 * Bezier(0.45, 0., 0.55, 1.).ease(f / TITLE_FRAMES as f32);
        svgr!(
            <g transform={Similarity::scale_about(breathing, w / 2., h / 2.).attr()}>
                {note}
                {title}
                {accent_line}
                {subtitle}
            </g>
        )
    }

    /// The spinning rotor rolls right while a gradient trail grows behind it.
    fn rotor_sweep<'a>(&self, f: f32, left: f32, cy: f32) -> Svgr<'a> {
        let (start, end) = (4.5, 21.);
        let progress = interpolate_eased(f, &[start, end], &[0., 1.], Bezier::EXPO_OUT.f());
        let x = progress * SWEEP_WIDTH;
        let rotor_opacity = interpolate(f, &[start, start + 2.4, end, end + 7.5], &[0., 1., 1., 0.]);
        let trail_opacity = interpolate(f, &[start, start + 3.], &[0., 1.]);
        let accent = Rgba::hex(self.palette.accent);

        let trail = if x > 0. {
            faded(
                trail_opacity,
                svgr!(
                    <g>
                        <linearGradient id="title-trail" gradientUnits="userSpaceOnUse" x1={left} y1="0" x2={left + x} y2="0">
                            {stops(&[(0., accent.alpha(0.)), (0.3, accent.a8(0x66)), (1., accent)])}
                        </linearGradient>
                        <rect x={left} y={cy - 1.} width={x} height="2" rx="1" fill="url(#title-trail)" />
                    </g>
                ),
            )
        } else {
            Svgr::empty()
        };

        let (rx, ry) = (left + x - ROTOR_SIZE / 2., cy - ROTOR_SIZE / 2.);
        let region = Rect::new(rx, ry, ROTOR_SIZE, ROTOR_SIZE).outset(12.);
        let spin = format!("rotate({} {} {})", 720. * progress, left + x, cy);
        let rotor = faded(
            rotor_opacity,
            svgr!(
                <g transform={spin}>
                    <filter id="title-rotor-glow" filterUnits="userSpaceOnUse" x={region.x} y={region.y} width={region.w} height={region.h} color-interpolation-filters="sRGB">
                        <feDropShadow dx="0" dy="0" stdDeviation="3" flood-color="white" flood-opacity="0.3" />
                    </filter>
                    <g filter="url(#title-rotor-glow)">{rotor(rx, ry, ROTOR_SIZE, "white")}</g>
                </g>
            ),
        );
        svgr!(<g>{trail}{rotor}</g>)
    }

    /// A short accent line widens while a highlight sweeps through it.
    fn gradient_sweep<'a>(f: f32, palette: &Palette, cx: f32, top: f32) -> Svgr<'a> {
        let width = interpolate_eased(f, &[6., 21.], &[0., 120.], Bezier::EXPO_OUT.f());
        if width <= 0. {
            return Svgr::empty();
        }
        let position = interpolate_eased(f, &[6., 30.], &[-200., 100.], Bezier::EXPO_OUT.f());
        let accent = Rgba::hex(palette.accent);
        let left = cx - width / 2.;
        // `background-size: 200%` positioned at `position`%, repeating.
        let tile = left - width * position / 100.;
        svgr!(
            <g>
                <linearGradient id="title-sweep" gradientUnits="userSpaceOnUse" x1={tile} y1="0" x2={tile + 2. * width} y2="0" spreadMethod="repeat">
                    {stops(&[(0., accent.alpha(0.)), (0.25, accent), (0.5, Rgba::WHITE), (0.75, accent), (1., accent.alpha(0.))])}
                </linearGradient>
                <rect x={left} y={top} width={width} height="2" rx="1" fill="url(#title-sweep)" />
            </g>
        )
    }
}
