//! The closing card: eight rotor wedges pop in as one blade and fan out into the mark,
//! then crossfade into the ASCII DROID wordmark.

use crate::fonts::{Font, mono_runs};
use crate::motion::{back_out, cubic_out, interpolate, interpolate_eased};
use crate::svg::{Anchor, ROTOR_PATH, ROTOR_VIEWBOX, Rect, Rgba, TextStyle, blended, faded, glow};
use crate::theme::Palette;
use fframes::{Svgr, svgr};

const DROID_ASCII: [&str; 7] = [
    "  ██████████    ██████████    ██████████    ███    ██████████",
    "  ███    ███    ███    ███    ███    ███    ███    ███    ███",
    "  ███    ███    ███    ███    ███    ███    ███    ███    ███",
    "  ███    ███    ██████████    ███    ███    ███    ███    ███",
    "  ███    ███    ███ ███       ███    ███    ███    ███    ███",
    "  ███    ███    ███  ███      ███    ███    ███    ███    ███",
    "  ██████████    ███   ███     ██████████    ███    ██████████",
];
const TAGLINE: &str = "AUTONOMOUS ENGINEERING";
/// The rotor's hub in its own viewBox; every wedge spins around it.
const HUB: (f32, f32) = (303.105, 319.528);
const FAN_END: f32 = 45.;
const CROSSFADE_END: f32 = 75.;

pub fn draw<'a>(palette: &Palette, (w, h): (f32, f32), frame: usize) -> Svgr<'a> {
    let f = frame as f32;
    let wedges = interpolate(f, &[FAN_END, CROSSFADE_END], &[1., 0.]);
    let wordmark = 1. - wedges;
    svgr!(
        <g>
            <rect width={w} height={h} fill={palette.bg} />
            {faded(wedges, fan(f, w / 2., h / 2.))}
            {faded(wordmark, wordmark_layer(palette, w, h))}
        </g>
    )
}

/// Eight copies of the mark clipped to one 45-degree slice: scaled in together, then each
/// rotated `i * 45` degrees so the slices tile the whole rotor.
fn fan<'a>(f: f32, cx: f32, cy: f32) -> Svgr<'a> {
    let scale = interpolate_eased(f, &[0., 12.], &[0., 1.], back_out(1.5));
    let glitching = f > 12. && f < 24. && (f as usize).is_multiple_of(3);
    let opacity = if glitching { 0.5 } else { interpolate(f, &[0., 12.], &[0., 1.]) };
    if scale <= 0. {
        return Svgr::empty();
    }
    let (hx, hy) = HUB;
    let slice = format!("{hx},{hy} {vw},{hy} {vw},{top}", vw = ROTOR_VIEWBOX.0, top = hy - (ROTOR_VIEWBOX.0 - hx));
    let blades: Svgr = (0..8)
        .map(|i| {
            let angle = interpolate_eased(f, &[12., FAN_END], &[0., i as f32 * 45.], cubic_out);
            let place = format!("translate({cx} {cy}) scale({scale}) rotate({angle}) translate({} {})", -hx, -hy);
            blended(
                "screen",
                opacity,
                svgr!(
                    <g transform={place} clip-path="url(#outro-slice)">
                        <path d={ROTOR_PATH} fill="white" />
                    </g>
                ),
            )
        })
        .collect();
    svgr!(
        <g>
            <clipPath id="outro-slice">
                <polygon points={slice} />
            </clipPath>
            {blades}
        </g>
    )
}

fn wordmark_layer<'a>(palette: &Palette, w: f32, h: f32) -> Svgr<'a> {
    let mono = TextStyle::new(Font::mono(400), 24.);
    let line_height = 24. * 1.2;
    let tagline = TextStyle::new(Font::sans(300), 32.).spaced(0.2);
    let tagline_height = tagline.normal_line_height();
    let block_height = DROID_ASCII.len() as f32 * line_height;
    let top = (h - block_height - 40. - tagline_height) / 2.;
    let advance = mono.width("█");
    let columns = DROID_ASCII.iter().map(|line| line.chars().count()).max().unwrap_or(0);
    let left = (w - columns as f32 * advance) / 2.;
    let block_area = Rect::new(left, top, columns as f32 * advance, block_height);

    let ascii = |fill: Rgba| -> Svgr<'a> {
        DROID_ASCII
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                let baseline = mono.baseline(top + row as f32 * line_height, line_height);
                mono_runs(line).into_iter().map(move |(column, run)| {
                    mono.draw(run, left + column as f32 * advance, baseline, fill, Anchor::Start)
                })
            })
            .collect()
    };

    let accent = Rgba::hex(palette.accent);
    let tagline_top = top + block_height + 40.;
    let tagline_baseline = tagline.baseline(tagline_top, tagline_height);
    let tagline_text = |fill: Rgba| tagline.draw(TAGLINE.to_owned(), w / 2., tagline_baseline, fill, Anchor::Middle);
    let tagline_width = tagline.width(TAGLINE);
    let tagline_area = Rect::new((w - tagline_width) / 2., tagline_top, tagline_width, tagline_height);
    svgr!(
        <g>
            {glow("outro-ascii-glow".to_owned(), 10., block_area, ascii(Rgba::WHITE.alpha(0.4)))}
            {ascii(Rgba::WHITE)}
            {glow("outro-tagline-glow".to_owned(), 7.5, tagline_area, tagline_text(accent.a8(0x66)))}
            {tagline_text(accent)}
        </g>
    )
}
