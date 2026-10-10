//! Keyframe interpolation and easing: inputs are clamped to the keyframe range, CSS
//! cubic-bezier easings shape each segment.

/// Piecewise-linear interpolation over increasing `input` stops, clamped at both ends.
pub fn interpolate(x: f32, input: &[f32], output: &[f32]) -> f32 {
    interpolate_eased(x, input, output, linear)
}

/// Like [`interpolate`], with `easing` applied to the progress of every segment.
pub fn interpolate_eased(x: f32, input: &[f32], output: &[f32], easing: impl Fn(f32) -> f32) -> f32 {
    debug_assert!(input.len() == output.len() && input.len() >= 2);
    let last = input.len() - 1;
    let segment = (1..last).take_while(|&i| x >= input[i]).last().unwrap_or(0);
    let (a, b) = (input[segment], input[segment + 1]);
    let t = if b > a { ((x - a) / (b - a)).clamp(0., 1.) } else { 1. };
    output[segment] + (output[segment + 1] - output[segment]) * easing(t)
}

pub fn linear(t: f32) -> f32 {
    t
}

/// CSS `cubic-bezier(x1, y1, x2, y2)`.
#[derive(Debug, Clone, Copy)]
pub struct Bezier(pub f32, pub f32, pub f32, pub f32);

impl Bezier {
    pub const EXPO_OUT: Bezier = Bezier(0.16, 1., 0.3, 1.);

    pub fn ease(self, t: f32) -> f32 {
        let Bezier(x1, y1, x2, y2) = self;
        if t <= 0. || t >= 1. {
            return t.clamp(0., 1.);
        }
        let curve = |p1: f64, p2: f64, s: f64| {
            let u = 1. - s;
            3. * u * u * s * p1 + 3. * u * s * s * p2 + s * s * s
        };
        let slope = |p1: f64, p2: f64, s: f64| {
            let u = 1. - s;
            3. * u * u * p1 + 6. * u * s * (p2 - p1) + 3. * s * s * (1. - p2)
        };
        let (x1, x2, target) = (f64::from(x1), f64::from(x2), f64::from(t));
        let mut s = target;
        for _ in 0..8 {
            let d = slope(x1, x2, s);
            if d.abs() < 1e-7 {
                break;
            }
            s -= (curve(x1, x2, s) - target) / d;
        }
        if !(0. ..=1.).contains(&s) || (curve(x1, x2, s) - target).abs() > 1e-6 {
            let (mut lo, mut hi) = (0., 1.);
            s = target;
            for _ in 0..40 {
                if curve(x1, x2, s) < target {
                    lo = s;
                } else {
                    hi = s;
                }
                s = (lo + hi) / 2.;
            }
        }
        curve(f64::from(y1), f64::from(y2), s) as f32
    }

    pub fn f(self) -> impl Fn(f32) -> f32 {
        move |t| self.ease(t)
    }
}

/// `Easing.out(Easing.back(s))`: overshoots the end, then settles.
pub fn back_out(s: f32) -> impl Fn(f32) -> f32 {
    move |t| {
        let u = 1. - t;
        1. - u * u * ((s + 1.) * u - s)
    }
}

/// `Easing.out(Easing.cubic)`.
pub fn cubic_out(t: f32) -> f32 {
    1. - (1. - t).powi(3)
}

/// Remotion's deterministic `random(seed)`: mulberry32 over a Java-style string hash.
pub fn seeded_random(seed: &str) -> f32 {
    let hash = seed
        .encode_utf16()
        .fold(0i32, |hash, unit| hash.wrapping_shl(5).wrapping_sub(hash).wrapping_add(i32::from(unit)));
    let mut t = (hash as u32).wrapping_add(0x6d2b_79f5);
    t = (t ^ (t >> 15)).wrapping_mul(t | 1);
    t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
    f64::from(t ^ (t >> 14)) as f32 / 4_294_967_296.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_clamps_and_walks_segments() {
        let stops = [0., 0.35, 0.55, 0.85, 1.];
        let values = [0., 0.25, 1., 0.2, 0.];
        assert_eq!(interpolate(-1., &stops, &values), 0.);
        assert!((interpolate(0.45, &stops, &values) - 0.625).abs() < 1e-6);
        assert_eq!(interpolate(2., &stops, &values), 0.);
    }

    #[test]
    fn bezier_matches_css_endpoints_and_midpoint() {
        let ease = Bezier(0.25, 0.1, 0.25, 1.);
        assert_eq!(ease.ease(0.), 0.);
        assert_eq!(ease.ease(1.), 1.);
        // CSS `ease` at 50% progress.
        assert!((ease.ease(0.5) - 0.8024).abs() < 1e-3);
    }

    #[test]
    fn back_out_overshoots_then_settles() {
        let ease = back_out(1.5);
        assert!(ease(0.6) > 1.);
        assert!((ease(1.) - 1.).abs() < 1e-6);
    }

    #[test]
    fn seeded_random_is_deterministic_and_in_unit_range() {
        let value = seeded_random("glitch-x-1000");
        assert_eq!(value, seeded_random("glitch-x-1000"));
        assert!((0. ..1.).contains(&value));
        assert_ne!(value, seeded_random("glitch-x-1001"));
    }
}
