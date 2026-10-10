//! Compositing in Rust on 8-bit premultiplied pixels: layers blended in at whole pixels,
//! resampling, blurs and the final colour grade. tiny-skia runs every image, faded group and
//! `mix-blend-mode` through its floating point pipeline, a full pass per layer per frame.

use crate::svg::{Rect, Rgba};
use fast_image_resize::images::{TypedImage, TypedImageRef};
use fast_image_resize::pixels::U8x4;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use fframes::usvgr::PreloadedImageData;
use multiversion::multiversion;
use std::array;
use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Premultiplied RGBA pixels being composited: a frame, or a layer of one.
#[derive(Clone)]
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blend {
    Over,
    Screen,
}

/// `v / 255`, rounded to nearest, for `v` up to `255 * 255`.
fn div255(v: u16) -> u16 {
    let v = v + 128;
    (v + (v >> 8)) >> 8
}

/// Loads a channel like tiny-skia.
fn unit(channel: u8) -> f32 {
    f32::from(channel) * (1. / 255.)
}

/// Stores a channel like tiny-skia, clamped to 0..1 (by the saturating cast) and rounded;
/// exact halves round up rather than to even, which no blend here lands on in practice.
fn store(value: f32) -> u8 {
    (value * 255. + 0.5) as u8
}

/// Luminosity as the non-separable blend modes define it.
fn lum([r, g, b]: [f32; 3]) -> f32 {
    r * 0.30 + g * 0.59 + b * 0.11
}

impl Pixels {
    pub fn transparent(width: u32, height: u32) -> Self {
        Self { width, height, data: vec![0; width as usize * height as usize * 4] }
    }

    /// A copy of `image` to composite onto.
    pub fn of(image: &PreloadedImageData) -> Self {
        Self { width: image.width, height: image.height, data: image.data.to_vec() }
    }

    /// Blends `layer`, placed at whole pixel `at` and faded by `opacity`, onto these pixels:
    /// tiny-skia's blend modes on every channel alike, in 8-bit fixed point (within a level
    /// of its floating point pipeline). Runs of pixels go through branch-free loops, which the
    /// compiler vectorises; source-over skips transparent runs and copies opaque ones.
    pub fn blend(&mut self, layer: &PreloadedImageData, at: (u32, u32), opacity: f32, mode: Blend) {
        let o = (opacity.clamp(0., 1.) * 255. + 0.5) as u16;
        if o > 0 {
            blend(self, layer, at, o, mode);
        }
    }

    /// How far `blur(sigma)` spreads a pixel along each axis.
    pub fn blur_reach(sigma: f32) -> usize {
        box_sizes(sigma).iter().map(|size| (size - 1) / 2).sum()
    }

    /// A Gaussian blur of `sigma` pixels, approximated like svgr's `feGaussianBlur`: five box
    /// blurs per axis, with the pixels outside counting as transparent black.
    pub fn blur(&mut self, sigma: f32) {
        let width = self.width as usize;
        let pixels = self.data.as_chunks_mut::<4>().0;
        let mut back = vec![[0; 4]; pixels.len()];
        for size in box_sizes(sigma) {
            let radius = (size - 1) / 2;
            blur_columns(radius, width, pixels, &mut back);
            for (src, dst) in back.chunks_exact(width).zip(pixels.chunks_exact_mut(width)) {
                blur_row(radius, src, dst);
            }
        }
    }

    /// These pixels as an `<image>` source.
    pub fn into_image(self) -> Arc<PreloadedImageData> {
        // svgr keys resampled copies of an image by its id and address, and a freed image's
        // address is soon reused: a fresh id keeps a new image from hitting a stale copy.
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = format!("pixels-{}", NEXT.fetch_add(1, Ordering::Relaxed));
        Arc::new(PreloadedImageData { data: Cow::Owned(self.data), width: self.width, height: self.height, id })
    }
}

/// The body of [`Pixels::blend`], compiled for each SIMD level the CPU may have.
#[multiversion(targets = "simd")]
fn blend(pixels: &mut Pixels, layer: &PreloadedImageData, (x, y): (u32, u32), o: u16, mode: Blend) {
    let columns = layer.width.min(pixels.width.saturating_sub(x)) as usize;
    let rows = layer.height.min(pixels.height.saturating_sub(y));
    const RUN: usize = 64;
    // Per channel of a run: what source-over keeps of the destination, `1 - sa`.
    let mut keep = [0u8; RUN];
    for row in 0..rows {
        let src = (row * layer.width) as usize * 4;
        let dst = (((y + row) * pixels.width + x) as usize) * 4;
        let src = &layer.data[src..src + columns * 4];
        let dst = &mut pixels.data[dst..dst + columns * 4];
        match mode {
            Blend::Over => {
                for (dst, src) in dst.chunks_mut(RUN).zip(src.chunks(RUN)) {
                    // Premultiplied: a transparent pixel is all zeros.
                    let words = src.as_chunks::<4>().0.iter().map(|s| u32::from_le_bytes(*s));
                    let (any, all) = words.fold((0, !0), |(any, all), s| (any | s, all & s));
                    if any == 0 {
                        continue;
                    }
                    if all >> 24 == 255 && o == 255 {
                        dst.copy_from_slice(src);
                        continue;
                    }
                    for (keep, s) in keep.as_chunks_mut::<4>().0.iter_mut().zip(src.as_chunks::<4>().0) {
                        *keep = [255 - div255(u16::from(s[3]) * o) as u8; 4];
                    }
                    for ((d, s), keep) in dst.iter_mut().zip(src).zip(&keep) {
                        *d = (div255(u16::from(*s) * o) + div255(u16::from(*d) * u16::from(*keep))) as u8;
                    }
                }
            }
            Blend::Screen => {
                for (d, s) in dst.iter_mut().zip(src) {
                    let (s, d0) = (div255(u16::from(*s) * o), u16::from(*d));
                    *d = (s + d0 - div255(s * d0)) as u8;
                }
            }
        }
    }
}

/// `crop` (in pixels, not necessarily whole) of the `size` RGBA pixels `data`, resampled to
/// `width` by `height` pixels with the Mitchell filter of tiny-skia's bicubic sampling.
/// Resampled once here, an image is then drawn 1:1 instead of sampled with sixteen taps per
/// output pixel.
pub fn resample(data: &[u8], size: (u32, u32), crop: Rect, (width, height): (u32, u32)) -> Pixels {
    thread_local! {
        static RESIZER: RefCell<Resizer> = RefCell::new(Resizer::new());
    }
    let source = TypedImageRef::<U8x4>::from_buffer(size.0, size.1, data).expect("images hold their size in pixels");
    let mut resampled = vec![0; width as usize * height as usize * 4];
    let mut target = TypedImage::<U8x4>::from_buffer(width, height, &mut resampled).expect("sized for the target");
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(FilterType::Mitchell))
        .crop(f64::from(crop.x), f64::from(crop.y), f64::from(crop.w), f64::from(crop.h))
        // Premultiplied already.
        .use_alpha(false);
    RESIZER
        .with_borrow_mut(|resizer| resizer.resize_typed(&source, &mut target, &options))
        .expect("source and target are both RGBA8");
    Pixels { width, height, data: resampled }
}

/// Widths of the five box blurs that approximate a Gaussian of `sigma`.
fn box_sizes(sigma: f32) -> [usize; 5] {
    const STEPS: f32 = 5.;
    let ideal = (12. * sigma * sigma / STEPS).sqrt() + 1.;
    let lower = ideal.floor() as usize;
    let lower = if lower.is_multiple_of(2) { lower - 1 } else { lower };
    let l = lower as f32;
    let narrow =
        ((12. * sigma * sigma - STEPS * l * l - 4. * STEPS * l - 3. * STEPS) / (-4. * l - 4.)).round() as usize;
    array::from_fn(|i| if i < narrow { lower } else { lower + 2 })
}

/// The mean of a box blur window, rounded like svgr: to nearest, ties to even, by adding and
/// taking away 1.5 * 2^23, which leaves no fraction bits (`round_ties_even` is a libm call
/// on baseline x86-64).
fn mean(sum: [i32; 4], scale: f32) -> [u8; 4] {
    const SNAP: f32 = 12_582_912.;
    sum.map(|s| (s as f32 * scale + SNAP - SNAP) as u8)
}

fn accumulate(sum: &mut [i32; 4], px: [u8; 4], sign: i32) {
    for c in 0..4 {
        sum[c] += sign * i32::from(px[c]);
    }
}

/// A vertical box blur of `2 radius + 1` rows, with the window sums kept for a whole row so
/// memory is walked in order.
#[multiversion(targets = "simd")]
fn blur_columns(radius: usize, width: usize, src: &[[u8; 4]], dst: &mut [[u8; 4]]) {
    let height = src.len() / width;
    let scale = 1. / (2 * radius + 1) as f32;
    let row = |y: usize| &src[y * width..(y + 1) * width];
    let add = |sums: &mut [[i32; 4]], y: usize, sign: i32| {
        sums.iter_mut().zip(row(y)).for_each(|(sum, px)| accumulate(sum, *px, sign));
    };
    let mut sums = vec![[0; 4]; width];
    for y in 0..radius.min(height) {
        add(&mut sums, y, 1);
    }
    for (y, out) in dst.chunks_exact_mut(width).enumerate() {
        if y + radius < height {
            add(&mut sums, y + radius, 1);
        }
        out.iter_mut().zip(&sums).for_each(|(px, sum)| *px = mean(*sum, scale));
        if y >= radius {
            add(&mut sums, y - radius, -1);
        }
    }
}

/// A horizontal box blur of `2 radius + 1` pixels along one row.
#[multiversion(targets = "simd")]
fn blur_row(radius: usize, src: &[[u8; 4]], dst: &mut [[u8; 4]]) {
    let scale = 1. / (2 * radius + 1) as f32;
    let mut sum = [0; 4];
    for px in &src[..radius.min(src.len())] {
        accumulate(&mut sum, *px, 1);
    }
    for (x, out) in dst.iter_mut().enumerate() {
        if let Some(px) = src.get(x + radius) {
            accumulate(&mut sum, *px, 1);
        }
        *out = mean(sum, scale);
        if x >= radius {
            accumulate(&mut sum, src[x - radius], -1);
        }
    }
}

/// Three static layers over opaque frames: overlay-blended grain, a full cover of one colour
/// with `mix-blend-mode: color`, and a black vignette, in fixed point a pixel at a time.
pub struct Grade {
    /// Per channel of every pixel: the faded grain's overlay strength `k = 2 s - sa` (in
    /// 2^-15) on the colour channels, what the vignette keeps (in 255ths) on alpha.
    terms: Vec<i16>,
    /// Per colour channel: the tint's hue term, its colour minus its luminosity (in 1/256
    /// levels).
    hue: [i32; 3],
    /// The tint's alpha per luminosity unit (in 2^-12 levels per unit of `30 r + 59 g + 11 b`).
    lift: i32,
    /// How fast the hue term is clipped above the luminosity where it would overflow (in
    /// 2^-24 per luminosity unit), see `grade`.
    clip: i32,
    /// What the tint leaves of the destination, `1 - alpha` (in 2^-16).
    keep: i32,
}

impl Grade {
    /// `grain` faded by `opacity`, then `tint`, then `vignette` (black at varying alpha); the
    /// layers are frame-sized.
    pub fn new(grain: &Pixels, opacity: f32, tint: Rgba, vignette: &Pixels) -> Self {
        let terms = grain
            .data
            .as_chunks::<4>()
            .0
            .iter()
            .zip(vignette.data.as_chunks::<4>().0)
            .flat_map(|(g, v)| {
                let k = |c: usize| ((2. * unit(g[c]) - unit(g[3])) * opacity * 32768.).round() as i16;
                [k(0), k(1), k(2), 255 - i16::from(v[3])]
            })
            .collect();
        // The colour is filled into an 8-bit layer before it is blended.
        let alpha = unit(store(tint.3));
        let s = [tint.0, tint.1, tint.2].map(|channel| unit(store(unit(channel) * tint.3)));
        let hue = s.map(|channel| channel - lum(s));
        let widest = hue.iter().copied().fold(0., f32::max);
        Self {
            terms,
            hue: hue.map(|h| (h * 255. * 256.).round() as i32),
            lift: (alpha * 255. * 4096. / 25500.).round() as i32,
            clip: (alpha / (widest * 25500.) * 16_777_216.).round().min(65536.) as i32,
            keep: ((1. - alpha) * 65536.).round() as i32,
        }
    }

    pub fn apply(&self, frame: &mut Pixels) {
        grade(self, frame);
    }
}

/// The body of [`Grade::apply`], compiled for each SIMD level the CPU may have.
#[multiversion(targets = "simd")]
fn grade(grade: &Grade, frame: &mut Pixels) {
    let &Grade { hue, lift, clip, keep, .. } = grade;
    for (px, terms) in frame.data.as_chunks_mut::<4>().0.iter_mut().zip(grade.terms.as_chunks::<4>().0) {
        // Overlay onto an opaque `d`: `d (1 + k)` up to `d = 0.5`, `d (1 - k) + k` above.
        let grained: [i32; 3] = array::from_fn(|c| {
            let d = i32::from(px[c]);
            d + ((i32::from(terms[c]) * d.min(255 - d) + (1 << 14)) >> 15)
        });
        // The colour blend sets the tint's colour to the destination's luminosity `l`: the
        // hue term over `l * alpha`, scaled down toward it where the brightest channel would
        // pass `alpha` (tiny-skia's clip_color), and none of it below zero.
        let l = 30 * grained[0] + 59 * grained[1] + 11 * grained[2];
        let clipped = (((25500 - l) * clip) >> 12).min(4096);
        let lifted = (l * lift) >> 4;
        for c in 0..3 {
            let tinted = (lifted + ((hue[c] * clipped) >> 12)).max(0);
            let level = (((grained[c] * keep) >> 8) + tinted + 128) >> 8;
            px[c] = div255((level.min(255) * i32::from(terms[3])) as u16) as u8;
        }
        px[3] = 255;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, px: [u8; 4]) -> Pixels {
        Pixels { width, height, data: px.repeat((width * height) as usize) }
    }

    #[test]
    fn layers_blend_only_where_they_are_placed() {
        let mut frame = solid(3, 2, [100, 100, 100, 255]);
        frame.blend(&solid(1, 1, [128, 0, 0, 128]).into_image(), (2, 1), 1., Blend::Over);
        assert_eq!(frame.data[..20], [100, 100, 100, 255].repeat(5));
        assert_eq!(frame.data[20..], [178, 50, 50, 255]);
    }

    #[test]
    fn blur_matches_svgr_gaussian_blur() {
        let (width, height) = (23, 17);
        let mut seed = 0x9e37_79b9u32;
        let data = (0..width * height)
            .flat_map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let a = (seed >> 24) as u8;
                [seed as u8, (seed >> 8) as u8, (seed >> 16) as u8].map(|c| c.min(a)).into_iter().chain([a])
            })
            .collect();
        let mut blurred = Pixels { width, height, data };
        let area = Rect::new(0., 0., width as f32, height as f32);
        let image = blurred.clone().into_image();
        let expected = crate::raster::rasterize(
            fframes::svgr!(
                <g>
                    {crate::svg::blur("b".to_owned(), 2.5, area)}
                    <g filter="url(#b)">
                        <image href={image} width={area.w} height={area.h} image-rendering="optimizeSpeed" />
                    </g>
                </g>
            ),
            area,
            1.,
        );
        blurred.blur(2.5);
        assert!(blurred.data == expected.data);
    }

    #[test]
    fn screen_follows_its_formula() {
        let mut screened = solid(1, 1, [51, 204, 0, 255]);
        screened.blend(&solid(1, 1, [102, 102, 102, 255]).into_image(), (0, 0), 1., Blend::Screen);
        assert_eq!(screened.data, [133, 224, 102, 255]);
    }

    fn graded(d: [u8; 4], grain: [u8; 4], tint: Rgba, vignette: u8) -> Vec<u8> {
        let mut frame = solid(1, 1, d);
        Grade::new(&solid(1, 1, grain), 1., tint, &solid(1, 1, [0, 0, 0, vignette])).apply(&mut frame);
        frame.data
    }

    #[test]
    fn grain_is_overlay_blended() {
        let none = Rgba::BLACK.alpha(0.);
        assert_eq!(graded([51, 204, 255, 255], [102, 102, 102, 255], none, 0), [41, 194, 255, 255]);
    }

    #[test]
    fn the_tint_keeps_luminosity_and_takes_the_hue() {
        let grey = graded([128, 128, 128, 255], [0, 0, 0, 0], Rgba(255, 0, 0, 1.), 0);
        assert!(grey[0] > grey[1] && grey[1] == grey[2], "{grey:?}");
        let lum = 0.30 * f32::from(grey[0]) + 0.59 * f32::from(grey[1]) + 0.11 * f32::from(grey[2]);
        assert!((lum - 128.).abs() <= 1., "{grey:?}");
    }

    #[test]
    fn the_vignette_darkens_by_its_alpha() {
        assert_eq!(graded([200, 100, 0, 255], [0, 0, 0, 0], Rgba::BLACK.alpha(0.), 51), [160, 80, 0, 255]);
    }
}
