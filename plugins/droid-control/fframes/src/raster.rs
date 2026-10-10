//! Rasterising markup in Rust, outside fframes' single tree per frame: frames composited in
//! layers, static layers baked once and reused as images, images resampled to the screen
//! pixels they cover, and the scene blurred at reduced resolution behind frosted panels.
//! Each stands in for work a CPU rasteriser would otherwise redo on every frame: large
//! Gaussian blurs, sixteen-tap bicubic sampling, and images run through its floating point
//! pipeline even where they land pixel for pixel.

use crate::composite::{Pixels, resample};
use crate::fonts;
use crate::svg::{Rect, Similarity};
use fframes::usvgr::{self, Group, ImageKind, Node, PreloadedImageData, fontdb};
use fframes::{Svgr, svgr};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::sync::{Arc, LazyLock, Mutex};
use svgr::tiny_skia::{IntSize, Pixmap, Transform};

static FONTS: LazyLock<fontdb::Database> = LazyLock::new(|| {
    let mut fonts_db = fontdb::Database::new();
    for font in fonts::font_data() {
        fonts_db.load_font_source(fontdb::Source::Binary(font.data));
    }
    fonts_db.load_system_fonts();
    fonts_db
});

struct Rasterizer {
    converter: usvgr::Cache,
    cache: svgr::SvgrCache,
    pool: svgr::PixmapPool,
}

thread_local! {
    static RASTERIZER: RefCell<Rasterizer> = RefCell::new(Rasterizer {
        converter: usvgr::Cache::new_with_text_cache(10),
        cache: svgr::SvgrCache::new(20),
        pool: svgr::PixmapPool::new(),
    });
}

/// `content` over `area` (user units) at `scale` pixels per unit.
pub fn rasterize(content: Svgr<'_>, area: Rect, scale: f32) -> Pixels {
    let (width, height) = ((area.w * scale).round().max(1.) as u32, (area.h * scale).round().max(1.) as u32);
    let mut pixels = Pixels::transparent(width, height);
    render(content, area, &mut pixels);
    pixels
}

/// `content` over `area` (user units) drawn onto `pixels`, which cover `area`.
fn render(content: Svgr<'_>, area: Rect, pixels: &mut Pixels) {
    let view_box = format!("{} {} {} {}", area.x, area.y, area.w, area.h);
    let document = svgr!(
        <svg xmlns="http://www.w3.org/2000/svg" viewBox={view_box} width={area.w} height={area.h}>{content}</svg>
    );
    let size = IntSize::from_wh(pixels.width, pixels.height).expect("pixels are never empty");
    let mut pixmap = Pixmap::from_vec(mem::take(&mut pixels.data), size).expect("pixels hold their size");
    RASTERIZER.with_borrow_mut(|Rasterizer { converter, cache, pool }| {
        let tree = document
            .into_svg_tree(&usvgr::Options::default(), converter, &FONTS)
            .expect("rasterised markup is valid SVG");
        let context = svgr::Context::new_from_pixmap_unsafe(&pixmap);
        let fit = Transform::from_scale(pixmap.width() as f32 / area.w, pixmap.height() as f32 / area.h);
        svgr::render(&tree, fit, &mut pixmap.as_mut(), cache, pool, &context);
    });
    pixels.data = pixmap.take();
}

impl Pixels {
    /// Rasterises `markup` onto these pixels, one per user unit.
    pub fn draw(&mut self, markup: Svgr<'_>) {
        render(markup, Rect::new(0., 0., self.width as f32, self.height as f32), self);
    }
}

/// `image` drawn pixel for pixel from the origin.
pub fn image<'a>(image: Arc<PreloadedImageData>) -> Svgr<'a> {
    let (width, height) = (image.width, image.height);
    svgr!(<image href={image} width={width} height={height} preserveAspectRatio="none" image-rendering="optimizeSpeed" />)
}

/// The pixels of a tree that is just one `image`: a frame composited before it reached the tree.
pub fn sole_image(tree: usvgr::Tree) -> Option<Pixels> {
    fn find(group: &Group) -> Option<&Arc<PreloadedImageData>> {
        match group.children() {
            [Node::Group(group)] => find(group),
            [Node::Image(image)] => match image.kind() {
                ImageKind::DATA(data) => Some(data),
                ImageKind::SVG { .. } => None,
            },
            _ => None,
        }
    }
    let image = Arc::clone(find(tree.root())?);
    drop(tree);
    Some(match Arc::try_unwrap(image) {
        Ok(image) => Pixels { width: image.width, height: image.height, data: image.data.into_owned() },
        Err(shared) => Pixels::of(&shared),
    })
}

/// `image`, whole screen pixels at `at`, drawn 1:1 from inside a box placed by `to_screen`.
pub fn on_pixels<'a>(image: Arc<PreloadedImageData>, at: Rect, to_screen: Similarity) -> Svgr<'a> {
    svgr!(
        <g transform={to_screen.inverse().attr()}>
            <image href={image} x={at.x} y={at.y} width={at.w} height={at.h} preserveAspectRatio="none" image-rendering="optimizeSpeed" />
        </g>
    )
}

static BAKED: LazyLock<Mutex<HashMap<String, Arc<PreloadedImageData>>>> = LazyLock::new(Default::default);

/// A static layer over `area`, drawn once per `key` over the whole pixels `area` touches
/// (in its units, passed to `draw`): the image, and that rect. `key` identifies everything
/// `draw` depends on except where `area` sits.
pub fn baked(key: String, area: Rect, draw: impl FnOnce(Rect) -> Pixels) -> (Arc<PreloadedImageData>, Rect) {
    let pixels = area.whole_pixels();
    let key = format!("{key} at +{}+{}", area.x - pixels.x, area.y - pixels.y);
    // Held while drawing, so render workers needing the same layer at once draw it once.
    let mut baked = BAKED.lock().expect("bake cache lock");
    let image = baked.entry(key).or_insert_with(|| draw(pixels).into_image());
    (Arc::clone(image), pixels)
}

/// `content` through an SVG Gaussian blur of `sigma` over `region` (see `Pixels::blur`):
/// rasterised over the whole pixels `region` touches and blurred there, and that rect.
pub fn blurred(content: Svgr<'_>, region: Rect, sigma: f32) -> (Pixels, Rect) {
    let pixels = region.whole_pixels();
    let mut layer = rasterize(content, pixels, 1.);
    layer.blur(sigma);
    (layer, pixels)
}

/// Where an image stretched over `drawn` (screen units) shows inside `bounds`: the whole
/// screen pixels it covers there, and the part of the image (in its `size` pixels) they show.
/// A partly covered pixel at the edges is left out.
pub fn coverage((iw, ih): (f32, f32), drawn: Rect, bounds: Rect) -> Option<(Rect, Rect)> {
    const SLACK: f32 = 1e-3;
    let visible = drawn.intersect(bounds);
    let (x, y) = ((visible.x - SLACK).ceil(), (visible.y - SLACK).ceil());
    let (right, bottom) = ((visible.right() + SLACK).floor(), (visible.bottom() + SLACK).floor());
    if x >= right || y >= bottom {
        return None;
    }
    let (sx, sy) = (iw / drawn.w, ih / drawn.h);
    let (cx, cy) = (((x - drawn.x) * sx).max(0.), ((y - drawn.y) * sy).max(0.));
    let crop = Rect::new(cx, cy, ((right - drawn.x) * sx).min(iw) - cx, ((bottom - drawn.y) * sy).min(ih) - cy);
    Some((Rect::new(x, y, right - x, bottom - y), crop))
}

type Resampled = (Arc<PreloadedImageData>, Rect, Rect, Arc<PreloadedImageData>);

thread_local! {
    /// Recent `on_screen` results: source, screen pixels, crop and the resampled image. The
    /// source is held so that its address, which identifies it, is not reused meanwhile.
    static RESAMPLED: RefCell<VecDeque<Resampled>> = RefCell::default();
}

/// `image` stretched over `area` (local units) placed on screen by `to_screen`, resampled to
/// the whole screen pixels it covers inside `bounds` (see `coverage`), and those pixels: one
/// separable resample instead of tiny-skia's sixteen taps per output pixel, and none at all
/// for an image that holds still, whose last results are kept.
pub fn on_screen(
    image: &Arc<PreloadedImageData>,
    area: Rect,
    to_screen: Similarity,
    bounds: Rect,
) -> Option<(Arc<PreloadedImageData>, Rect)> {
    let size = (image.width as f32, image.height as f32);
    let (at, crop) = coverage(size, to_screen.apply(area), bounds)?;
    let resampled = RESAMPLED.with_borrow_mut(|recent| {
        let hit = recent.iter().find(|(source, a, c, _)| Arc::ptr_eq(source, image) && (*a, *c) == (at, crop));
        if let Some((.., resampled)) = hit {
            return Arc::clone(resampled);
        }
        let resampled = if crop == Rect::new(0., 0., size.0, size.1) && (at.w, at.h) == size {
            Arc::clone(image)
        } else {
            resample(&image.data, (image.width, image.height), crop, (at.w as u32, at.h as u32)).into_image()
        };
        if recent.len() == 16 {
            recent.pop_front();
        }
        recent.push_back((Arc::clone(image), at, crop, Arc::clone(&resampled)));
        resampled
    });
    Some((resampled, at))
}

/// The scene behind the overlays, blurred where a frosted panel shows it. Each blur runs on
/// a downscaled copy (still at least 3 px of blur wide) of just the part of the scene it
/// needs, and each patch is scaled back up to screen pixels, like browsers do for large
/// `backdrop-filter` radii.
pub struct Backdrop {
    scene: Pixels,
}

impl Backdrop {
    pub fn new(scene: Pixels) -> Self {
        Self { scene }
    }

    /// The scene under `area` (screen units) blurred by `sigma`, and the screen rect it covers.
    pub fn frosted(&self, sigma: f32, area: Rect) -> Option<(Arc<PreloadedImageData>, Rect)> {
        let scene = &self.scene;
        let (w, h) = (scene.width as f32, scene.height as f32);
        // The whole screen pixels `area` touches, so the patch is drawn 1:1.
        let at = area.whole_pixels().intersect(Rect::new(0., 0., w, h));
        if at.w <= 0. || at.h <= 0. {
            return None;
        }
        let factor = (sigma / 3.).floor().clamp(1., 4.);
        let (small_w, small_h) = ((w / factor).round(), (h / factor).round());
        let (sx, sy) = (small_w / w, small_h / h);
        let sigma = sigma * sx;
        // Only the part the patch is resampled from is downscaled and blurred, with room for
        // the blur's reach and the resampling filter around it: there it comes out as if all
        // of the scene were.
        let reach = (Pixels::blur_reach(sigma) + 3) as f32;
        let (rx, ry) = ((at.x * sx - reach).floor().max(0.), (at.y * sy - reach).floor().max(0.));
        let (region_w, region_h) =
            ((at.right() * sx + reach).ceil().min(small_w) - rx, (at.bottom() * sy + reach).ceil().min(small_h) - ry);
        let source = Rect::new(rx / sx, ry / sy, region_w / sx, region_h / sy);
        let mut region = resample(&scene.data, (scene.width, scene.height), source, (region_w as u32, region_h as u32));
        region.blur(sigma);
        let crop = Rect::new(at.x * sx - rx, at.y * sy - ry, at.w * sx, at.h * sy);
        let patch = resample(&region.data, (region.width, region.height), crop, (at.w as u32, at.h as u32));
        Some((patch.into_image(), at))
    }

    pub fn into_scene(self) -> Pixels {
        self.scene
    }
}
