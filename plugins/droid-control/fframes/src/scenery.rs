//! Layers that frame every segment: the warm background, drifting particles, the rotor
//! watermark, film grain and the final colour grade.

use crate::svg::{Rect, Rgba, blended, faded, rotor, stops};
use crate::theme::{BgStyle, Palette, PresetConfig};
use fframes::media::ImageData;
use fframes::{Svgr, svgr};
use std::f32::consts::FRAC_1_SQRT_2;

/// A CSS `radial-gradient(ellipse at center, ...)` (farthest-corner) filling `r`.
fn radial<'a>(id: String, r: Rect, css: &[(f32, Rgba)]) -> Svgr<'a> {
    let fill = format!("url(#{id})");
    svgr!(
        <g>
            <radialGradient id={id} cx="0.5" cy="0.5" r={FRAC_1_SQRT_2}>{stops(css)}</radialGradient>
            <rect x={r.x} y={r.y} width={r.w} height={r.h} fill={fill} />
        </g>
    )
}

/// Full-bleed background. Warm palettes get a vignette, four corner glows that intensify
/// from a mild wash to a rich bloom over `total` frames, and the halftone rotor texture.
pub fn background<'a>(
    id: &str,
    palette: &Palette,
    config: &PresetConfig,
    halftone: &ImageData<'_>,
    (w, h): (f32, f32),
    frame: usize,
    total: usize,
) -> Svgr<'a> {
    let full = Rect::new(0., 0., w, h);
    let bg = Rgba::hex(palette.bg);
    if !palette.warm {
        return match config.bg_style {
            BgStyle::Gradient => radial(
                format!("{id}-cool"),
                full,
                &[(0., bg), (0.5, Rgba::hex("#0f0f1a")), (0.8, Rgba::hex("#121220")), (1., Rgba::hex("#1a1028"))],
            ),
            BgStyle::Solid => svgr!(<rect width={w} height={h} fill={palette.bg} />),
        };
    }

    let warmth = (frame as f32 / total as f32).clamp(0., 1.);
    let glow = 0.05 + 0.15 * warmth;
    let blob = |name: &str, (x, y, bw, bh): (f32, f32, f32, f32), color: Rgba, edge: f32| {
        radial(format!("{id}-{name}"), Rect::new(x * w, y * h, bw * w, bh * h), &[(0., color), (edge, color.alpha(0.))])
    };
    let halftone_side = w.max(h);
    svgr!(
        <g>
            {radial(
                format!("{id}-vignette"),
                full,
                &[(0., bg), (0.55, Rgba::hex("#1a0e08")), (0.85, Rgba::hex("#2a1510")), (1., Rgba::hex("#351a12"))],
            )}
            {blob("bl", (-0.1, 0.5, 0.6, 0.6), Rgba(200, 80, 20, glow), 0.7)}
            {blob("tr", (0.55, -0.15, 0.55, 0.55), Rgba(180, 60, 15, glow * 0.8), 0.7)}
            {blob("br", (0.6, 0.55, 0.45, 0.5), Rgba(160, 40, 10, glow * 0.6), 0.65)}
            {blob("tl", (-0.05, -0.08, 0.4, 0.4), Rgba(238, 96, 24, glow * 0.4), 0.6)}
            {blended("screen", 0.15, svgr!(
                <image href={halftone.href()} x={(w - halftone_side) / 2.} y={(h - halftone_side) / 2.}
                    width={halftone_side} height={halftone_side} />
            ))}
        </g>
    )
}

/// 30 accent dots on slow Lissajous paths.
pub fn particles<'a>(accent: &'a str, (w, h): (f32, f32), frame: usize) -> Svgr<'a> {
    let f = frame as f32;
    let dots: Svgr = (0..30)
        .map(|i| {
            let base_x = ((i * 73 + 17) % 100) as f32 / 100. * w;
            let base_y = ((i * 47 + 31) % 100) as f32 / 100. * h;
            let size = (2 + (i % 3) * 2) as f32;
            let dx = (f * 0.008 + (i * 13 + 7) as f32 * 0.1).sin() * 60.;
            let dy = (f * 0.006 + (i * 19 + 11) as f32 * 0.1).cos() * 40.;
            svgr!(<circle cx={base_x + dx + size / 2.} cy={base_y + dy + size / 2.} r={size / 2.} fill={accent} />)
        })
        .collect();
    faded(0.07, dots)
}

/// The rotor in the bottom-right corner, fading in to 20% over the first second.
pub fn watermark<'a>((w, h): (f32, f32), frame: usize) -> Svgr<'a> {
    let size = 48.;
    faded(0.2 * (frame as f32 / 30.).min(1.), rotor(w - 32. - size, h - 24. - size, size, "white"))
}

/// Static fractal noise, overlay-blended.
pub fn noise<'a>((w, h): (f32, f32), opacity: f32) -> Svgr<'a> {
    blended(
        "overlay",
        opacity,
        svgr!(
            <g>
                <filter id="noise-filter" filterUnits="userSpaceOnUse" x="0" y="0" width={w} height={h}>
                    <feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch" />
                </filter>
                <rect width={w} height={h} filter="url(#noise-filter)" />
            </g>
        ),
    )
}

/// The topmost grade: a colour-blended temperature tint and a soft corner vignette.
pub fn grade<'a>(palette: &Palette, (w, h): (f32, f32), intensity: f32) -> Svgr<'a> {
    let tint = if palette.warm { Rgba(200, 120, 40, intensity) } else { Rgba(80, 100, 200, intensity * 0.75) };
    svgr!(
        <g>
            {blended("color", 1., svgr!(<rect width={w} height={h} fill={tint.to_string()} />))}
            {radial(
                "grade-vignette".to_owned(),
                Rect::new(0., 0., w, h),
                &[(0.4, Rgba::BLACK.alpha(0.)), (1., Rgba::BLACK.alpha(0.15))],
            )}
        </g>
    )
}
