//! Layers that frame every segment: the warm background, drifting particles, the rotor
//! watermark, film grain and the final colour grade. The full-frame ones are rasterised
//! once and composited per frame (see `composite`).

use crate::composite::{Blend, Grade, Pixels};
use crate::raster;
use crate::svg::{Rect, Rgba, faded, rotor, stops};
use crate::theme::{BgStyle, Palette, PresetConfig, Treatment};
use fframes::media::ImageData;
use fframes::usvgr::PreloadedImageData;
use fframes::{Svgr, svgr};
use std::collections::VecDeque;
use std::f32::consts::FRAC_1_SQRT_2;
use std::sync::{Arc, Mutex};

const HALFTONE: &[u8] = include_bytes!("../media/bg-halftone-rotor.png");

/// The warm glow level is rounded to this many steps per unit: a step moves no channel by
/// more than about a quarter of an 8-bit level, and runs of frames share one painting (and
/// svgr's resampled copy of it).
const GLOW_STEPS: f32 = 512.;

/// A CSS `radial-gradient(ellipse at center, ...)` (farthest-corner) filling `r`.
fn radial<'a>(r: Rect, css: &[(f32, Rgba)]) -> Svgr<'a> {
    svgr!(
        <g>
            <radialGradient id="radial" cx="0.5" cy="0.5" r={FRAC_1_SQRT_2}>{stops(css)}</radialGradient>
            <rect x={r.x} y={r.y} width={r.w} height={r.h} fill="url(#radial)" />
        </g>
    )
}

/// A warm corner glow at full strength, rasterised over the whole pixels it touches.
struct Glow {
    at: (u32, u32),
    image: Arc<PreloadedImageData>,
    /// Its opacity relative to the shared glow level.
    strength: f32,
}

/// The warm background: a vignette gradient, four corner glows that intensify from a mild
/// wash to a rich bloom over a segment, and the halftone rotor texture screened over them.
struct Warm {
    base: Pixels,
    glows: Vec<Glow>,
    halftone: Arc<PreloadedImageData>,
}

enum Painting {
    Warm(Warm),
    /// A background that never changes.
    Still(Arc<PreloadedImageData>),
}

pub struct Scenery {
    painting: Painting,
    /// Recent warm paintings by glow step, shared by the render workers.
    painted: Mutex<VecDeque<(u32, Arc<PreloadedImageData>)>>,
    /// Fractal noise grain, a colour-blended temperature tint and a soft corner vignette.
    grade: Grade,
}

impl Scenery {
    pub fn new(
        palette: &'static Palette,
        config: &'static PresetConfig,
        treatment: Treatment,
        size: (f32, f32),
    ) -> Result<Self, String> {
        let (w, h) = size;
        let full = Rect::new(0., 0., w, h);
        let painting = if palette.warm {
            let halftone = ImageData::new_from_bytes("bg-halftone-rotor.png", HALFTONE)
                .map_err(|err| format!("could not decode the embedded halftone texture: {err:?}"))?;
            let side = w.max(h);
            let glow = |(x, y, bw, bh): (f32, f32, f32, f32), color: Rgba, strength: f32, edge: f32| {
                let area = Rect::new(x * w, y * h, bw * w, bh * h);
                let (x0, y0) = (area.x.floor().max(0.), area.y.floor().max(0.));
                let (x1, y1) = (area.right().ceil().min(w), area.bottom().ceil().min(h));
                let touched = Rect::new(x0, y0, x1 - x0, y1 - y0);
                let image = raster::rasterize(radial(area, &[(0., color), (edge, color.alpha(0.))]), touched, 1.);
                Glow { at: (x0 as u32, y0 as u32), image: image.into_image(), strength }
            };
            Painting::Warm(Warm {
                base: raster::rasterize(
                    radial(
                        full,
                        &[
                            (0., Rgba::hex(palette.bg)),
                            (0.55, Rgba::hex("#1a0e08")),
                            (0.85, Rgba::hex("#2a1510")),
                            (1., Rgba::hex("#351a12")),
                        ],
                    ),
                    full,
                    1.,
                ),
                glows: vec![
                    glow((-0.1, 0.5, 0.6, 0.6), Rgba(200, 80, 20, 1.), 1., 0.7),
                    glow((0.55, -0.15, 0.55, 0.55), Rgba(180, 60, 15, 1.), 0.8, 0.7),
                    glow((0.6, 0.55, 0.45, 0.5), Rgba(160, 40, 10, 1.), 0.6, 0.65),
                    glow((-0.05, -0.08, 0.4, 0.4), Rgba(238, 96, 24, 1.), 0.4, 0.6),
                ],
                halftone: raster::rasterize(
                    svgr!(<image href={halftone.href()} x={(w - side) / 2.} y={(h - side) / 2.} width={side} height={side} />),
                    full,
                    1.,
                )
                .into_image(),
            })
        } else {
            let still = match config.bg_style {
                BgStyle::Gradient => radial(
                    full,
                    &[
                        (0., Rgba::hex(palette.bg)),
                        (0.5, Rgba::hex("#0f0f1a")),
                        (0.8, Rgba::hex("#121220")),
                        (1., Rgba::hex("#1a1028")),
                    ],
                ),
                BgStyle::Solid => svgr!(<rect width={w} height={h} fill={palette.bg} />),
            };
            Painting::Still(raster::rasterize(still, full, 1.).into_image())
        };
        let intensity = treatment.grade_intensity;
        let grade = Grade::new(
            &raster::rasterize(
                svgr!(
                    <g>
                        <filter id="noise-filter" filterUnits="userSpaceOnUse" x="0" y="0" width={w} height={h}>
                            <feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch" />
                        </filter>
                        <rect width={w} height={h} filter="url(#noise-filter)" />
                    </g>
                ),
                full,
                1.,
            ),
            treatment.noise_opacity,
            if palette.warm { Rgba(200, 120, 40, intensity) } else { Rgba(80, 100, 200, intensity * 0.75) },
            &raster::rasterize(radial(full, &[(0.4, Rgba::BLACK.alpha(0.)), (1., Rgba::BLACK.alpha(0.15))]), full, 1.),
        );
        Ok(Self { painting, painted: Mutex::default(), grade })
    }

    /// The opaque full-frame background at `frame` of a `total`-frame segment.
    pub fn background(&self, frame: usize, total: usize) -> Arc<PreloadedImageData> {
        let (base, glows, halftone) = match &self.painting {
            Painting::Warm(Warm { base, glows, halftone }) => (base, glows, halftone),
            Painting::Still(still) => return Arc::clone(still),
        };
        let step = ((0.05 + 0.15 * (frame as f32 / total as f32).clamp(0., 1.)) * GLOW_STEPS).round() as u32;
        let cached = self
            .painted
            .lock()
            .expect("paintings lock")
            .iter()
            .find(|(s, _)| *s == step)
            .map(|(_, image)| Arc::clone(image));
        cached.unwrap_or_else(|| {
            let level = step as f32 / GLOW_STEPS;
            let mut painted = base.clone();
            for Glow { at, image, strength } in glows {
                painted.blend(image, *at, level * strength, Blend::Over);
            }
            painted.blend(halftone, (0, 0), 0.15, Blend::Screen);
            let image = painted.into_image();
            let mut recent = self.painted.lock().expect("paintings lock");
            // Enough for every worker's current frame and the next.
            if recent.len() == 8 {
                recent.pop_front();
            }
            recent.push_back((step, Arc::clone(&image)));
            image
        })
    }

    /// Film grain and the colour grade over a finished, opaque frame: the topmost layers.
    pub fn grade(&self, frame: &mut Pixels) {
        self.grade.apply(frame);
    }
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
