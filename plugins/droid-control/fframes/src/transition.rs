//! The two scene transitions (title -> content, content -> outro). Each style restyles both
//! the exiting and the entering scene from the same 0..1 progress; the entering scene is
//! drawn on top.
//!
//! Style names follow @hyperframes/shader-transitions (Apache-2.0); the looks are drawn
//! here with SVG filters and blend modes.

use crate::motion::{Bezier, interpolate, seeded_random};
use crate::props::TransitionStyle;
use crate::svg::{Rect, Rgba, Similarity, blended, blur, faded, stops};
use fframes::{Svgr, svgr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Entering,
    Exiting,
}

pub struct Transition {
    pub style: TransitionStyle,
    pub warm: bool,
    pub size: (f32, f32),
}

impl Transition {
    /// `scene` restyled at `progress` (0..1) of the transition.
    pub fn present<'a>(&self, direction: Direction, progress: f32, scene: Svgr<'a>) -> Svgr<'a> {
        let p = progress;
        let entering = direction == Direction::Entering;
        let id = if entering { "enter" } else { "exit" };
        match self.style {
            TransitionStyle::MotionBlur => {
                if entering {
                    let (w, h) = self.size;
                    let place = Similarity::scale_about(1.03 - 0.03 * p, w / 2., h / 2.);
                    faded(p, svgr!(<g transform={place.attr()}>{self.blurred(id, 6. * (1. - p), scene)}</g>))
                } else {
                    faded(1. - p, self.blurred(id, 6. * p, scene))
                }
            }
            TransitionStyle::Flash => {
                if entering {
                    let flash = interpolate(p, &[0., 0.35, 0.55, 0.85, 1.], &[0., 0.25, 1., 0.2, 0.]);
                    let color = if self.warm { Rgba(255, 220, 180, 1.) } else { Rgba(235, 240, 255, 1.) };
                    let (w, h) = self.size;
                    svgr!(
                        <g>
                            {faded(((p - 0.45) / 0.35).clamp(0., 1.), scene)}
                            {blended("screen", flash, svgr!(<rect width={w} height={h} fill={color.rgb()} />))}
                        </g>
                    )
                } else {
                    faded((1. - (p - 0.2) / 0.35).clamp(0., 1.), scene)
                }
            }
            TransitionStyle::WhipPan => {
                let eased = Bezier(0.6, 0., 0.1, 1.).ease(p);
                let (shift, sigma, opacity) = if entering {
                    (
                        0.3 * (1. - eased),
                        interpolate(p, &[0., 0.6, 1.], &[14., 4., 0.]),
                        interpolate(p, &[0., 0.2, 1.], &[0., 1., 1.]),
                    )
                } else {
                    (
                        -0.3 * eased,
                        interpolate(p, &[0., 0.4, 1.], &[0., 10., 14.]),
                        interpolate(p, &[0., 0.8, 1.], &[1., 1., 0.]),
                    )
                };
                let place = Similarity::translate(shift * self.size.0, 0.);
                faded(opacity, svgr!(<g transform={place.attr()}>{self.blurred(id, sigma, scene)}</g>))
            }
            TransitionStyle::LightLeak => {
                if entering {
                    let scene_opacity = interpolate(p, &[0., 0.5, 1.], &[0., 0.4, 1.]);
                    let sweep = interpolate(p, &[0., 0.3, 0.6, 1.], &[0., 1., 0.6, 0.]);
                    svgr!(<g>{faded(scene_opacity, scene)}{blended("screen", sweep, self.light_leak(p))}</g>)
                } else {
                    faded(interpolate(p, &[0., 0.5, 1.], &[1., 0.6, 0.]), scene)
                }
            }
            TransitionStyle::GlitchLite => self.glitch(entering, p, scene),
        }
    }

    /// CSS `filter: blur(sigma)` over the whole frame.
    fn blurred<'a>(&self, id: &str, sigma: f32, scene: Svgr<'a>) -> Svgr<'a> {
        if sigma <= 0.01 {
            return scene;
        }
        let (w, h) = self.size;
        let filter = format!("transition-{id}-blur");
        let url = format!("url(#{filter})");
        svgr!(<g>{blur(filter, sigma, Rect::new(0., 0., w, h))}<g filter={url}>{scene}</g></g>)
    }

    /// A warm (or cool) highlight band at 105 degrees, sweeping left to right.
    fn light_leak<'a>(&self, p: f32) -> Svgr<'a> {
        let (w, h) = self.size;
        let leak = if self.warm { Rgba(238, 96, 24, 0.) } else { Rgba(137, 180, 250, 0.) };
        let at = -0.4 + 1.8 * p;
        // CSS gradient line for 105deg: through the centre, long enough to reach the corners.
        let angle = 105f32.to_radians();
        let (dx, dy) = (angle.sin(), -angle.cos());
        let half = (w * dx.abs() + h * dy.abs()) / 2.;
        let (cx, cy) = (w / 2., h / 2.);
        svgr!(
            <g>
                <linearGradient id="light-leak" gradientUnits="userSpaceOnUse" x1={cx - dx * half} y1={cy - dy * half} x2={cx + dx * half} y2={cy + dy * half}>
                    {stops(&[
                        (0., leak),
                        (at - 0.3, leak),
                        (at, leak.alpha(0.45)),
                        (at + 0.12, Rgba(255, 240, 200, 0.35)),
                        (at + 0.4, leak),
                        (1., leak),
                    ])}
                </linearGradient>
                <rect width={w} height={h} fill="url(#light-leak)" />
            </g>
        )
    }

    /// RGB-split ghosts and a displaced horizontal band around the midpoint.
    fn glitch<'a>(&self, entering: bool, p: f32, scene: Svgr<'a>) -> Svgr<'a> {
        let (w, h) = self.size;
        let scene_opacity = if entering {
            interpolate(p, &[0., 0.4, 1.], &[0., 1., 1.])
        } else {
            interpolate(p, &[0., 0.6, 1.], &[1., 1., 0.])
        };
        let intensity = interpolate(p, &[0., 0.35, 0.55, 0.75, 1.], &[0., 1., 1., 0.3, 0.]);
        let seed = if entering { 1000 } else { 2000 };
        let random =
            |name: &str, steps: f32| seeded_random(&format!("glitch-{name}-{}", (p * steps).round() as i32 + seed));
        let jitter = (random("x", 60.) - 0.5) * 16. * intensity;
        let split = 6. * intensity;
        let band_top = 20. + 40. * random("band-top", 30.);
        let band_height = 6. + 14. * random("band-h", 30.);
        let id = if entering { "enter" } else { "exit" };
        let shifted =
            |dx: f32, content: Svgr<'a>| svgr!(<g transform={Similarity::translate(dx, 0.).attr()}>{content}</g>);

        let ghost = |name: &str, hue: f32, dx: f32| {
            if split <= 0. {
                return Svgr::empty();
            }
            let filter = format!("glitch-{id}-{name}");
            let url = format!("url(#{filter})");
            let tint = tint_matrix(hue);
            blended(
                "screen",
                0.45,
                shifted(
                    dx,
                    svgr!(
                        <g>
                            <filter id={filter} filterUnits="userSpaceOnUse" x="0" y="0" width={w} height={h} color-interpolation-filters="sRGB">
                                <feColorMatrix type="matrix" values={tint[0].clone()} />
                                <feColorMatrix type="matrix" values={tint[1].clone()} />
                                <feColorMatrix type="matrix" values={tint[2].clone()} />
                                <feColorMatrix type="matrix" values={tint[3].clone()} />
                            </filter>
                            <g filter={url}>{scene.clone()}</g>
                        </g>
                    ),
                ),
            )
        };
        let band = if intensity > 0.6 {
            let clip = format!("glitch-{id}-band");
            let url = format!("url(#{clip})");
            faded(
                0.85,
                shifted(
                    jitter * 2.2,
                    svgr!(
                        <g>
                            <clipPath id={clip}>
                                <rect x="0" y={band_top / 100. * h} width={w} height={band_height / 100. * h} />
                            </clipPath>
                            <g clip-path={url}>{scene.clone()}</g>
                        </g>
                    ),
                ),
            )
        } else {
            Svgr::empty()
        };
        faded(
            scene_opacity,
            svgr!(
                <g>
                    {ghost("red", -50., jitter - split)}
                    {ghost("blue", 180., jitter + split)}
                    {shifted(jitter, scene.clone())}
                    {band}
                </g>
            ),
        )
    }
}

/// `sepia(1) saturate(6) hue-rotate(hue) contrast(1.1)` as four colour matrices, applied in
/// turn so every step clamps like the CSS filter chain.
fn tint_matrix(hue: f32) -> [String; 4] {
    let (sin, cos) = hue.to_radians().sin_cos();
    let s = 6.;
    let rows = |m: [[f32; 3]; 3], offset: f32| {
        m.iter()
            .map(|row| format!("{} {} {} 0 {offset}", row[0], row[1], row[2]))
            .chain(["0 0 0 1 0".to_owned()])
            .collect::<Vec<_>>()
            .join(" ")
    };
    [
        rows([[0.393, 0.769, 0.189], [0.349, 0.686, 0.168], [0.272, 0.534, 0.131]], 0.),
        rows(
            [
                [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
            ],
            0.,
        ),
        rows(
            [
                [
                    0.213 + cos * 0.787 - sin * 0.213,
                    0.715 - cos * 0.715 - sin * 0.715,
                    0.072 - cos * 0.072 + sin * 0.928,
                ],
                [
                    0.213 - cos * 0.213 + sin * 0.143,
                    0.715 + cos * 0.285 + sin * 0.140,
                    0.072 - cos * 0.072 - sin * 0.283,
                ],
                [
                    0.213 - cos * 0.213 - sin * 0.787,
                    0.715 - cos * 0.715 + sin * 0.715,
                    0.072 + cos * 0.928 + sin * 0.072,
                ],
            ],
            0.,
        ),
        rows([[1.1, 0., 0.], [0., 1.1, 0.], [0., 0., 1.1]], -0.05),
    ]
}
