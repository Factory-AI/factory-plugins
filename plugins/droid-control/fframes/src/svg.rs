//! SVG building blocks shared by the layers: geometry, colours, blur filters and the
//! rounded "card" every overlay is drawn as.

use crate::fonts::Font;
use fframes::{Svgr, svgr};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn cx(&self) -> f32 {
        self.x + self.w / 2.
    }

    pub fn cy(&self) -> f32 {
        self.y + self.h / 2.
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn outset(&self, by: f32) -> Rect {
        Rect::new(self.x - by, self.y - by, self.w + 2. * by, self.h + 2. * by)
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }
}

/// A CSS colour with straight alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub f32);

impl Rgba {
    pub const WHITE: Rgba = Rgba(255, 255, 255, 1.);
    pub const BLACK: Rgba = Rgba(0, 0, 0, 1.);

    /// `#rrggbb`.
    pub fn hex(hex: &str) -> Rgba {
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("palette colours are #rrggbb");
        Rgba(channel(1), channel(3), channel(5), 1.)
    }

    pub fn alpha(self, alpha: f32) -> Rgba {
        Rgba(self.0, self.1, self.2, alpha)
    }

    /// The alpha of an 8-digit CSS hex suffix, e.g. `#181818E6` is `hex("#181818").a8(0xE6)`.
    pub fn a8(self, alpha: u8) -> Rgba {
        self.alpha(f32::from(alpha) / 255.)
    }

    pub fn rgb(&self) -> String {
        format!("rgb({},{},{})", self.0, self.1, self.2)
    }
}

impl fmt::Display for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgba({},{},{},{})", self.0, self.1, self.2, self.3)
    }
}

/// A uniform scale followed by a translation, `p -> s * p + (dx, dy)`: every overlay
/// transform (CSS `translate` and `scale` around a `transform-origin`) has this shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Similarity {
    pub s: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Similarity {
    pub const IDENTITY: Similarity = Similarity { s: 1., dx: 0., dy: 0. };

    pub fn translate(dx: f32, dy: f32) -> Self {
        Self { s: 1., dx, dy }
    }

    /// CSS `scale(s)` around (`cx`, `cy`).
    pub fn scale_about(s: f32, cx: f32, cy: f32) -> Self {
        Self { s, dx: cx - s * cx, dy: cy - s * cy }
    }

    /// `self` followed by `outer`.
    pub fn then(self, outer: Similarity) -> Self {
        Self { s: self.s * outer.s, dx: self.dx * outer.s + outer.dx, dy: self.dy * outer.s + outer.dy }
    }

    pub fn inverse(self) -> Self {
        Self { s: 1. / self.s, dx: -self.dx / self.s, dy: -self.dy / self.s }
    }

    pub fn apply(self, r: Rect) -> Rect {
        Rect::new(r.x * self.s + self.dx, r.y * self.s + self.dy, r.w * self.s, r.h * self.s)
    }

    pub fn attr(self) -> String {
        format!("matrix({} 0 0 {} {} {})", self.s, self.s, self.dx, self.dy)
    }
}

/// `<g>` with `opacity`, skipping the group entirely once it is invisible.
pub fn faded<'a>(opacity: f32, content: Svgr<'a>) -> Svgr<'a> {
    if opacity <= 0. {
        return Svgr::empty();
    }
    svgr!(<g opacity={opacity.min(1.)}>{content}</g>)
}

/// `<g>` composited with a CSS `mix-blend-mode` (only accepted through `style`).
pub fn blended<'a>(mode: &str, opacity: f32, content: Svgr<'a>) -> Svgr<'a> {
    if opacity <= 0. {
        return Svgr::empty();
    }
    svgr!(<g style={format!("mix-blend-mode:{mode}")} opacity={opacity.min(1.)}>{content}</g>)
}

/// The Factory rotor mark, `ROTOR_VIEWBOX` units wide and tall.
pub const ROTOR_PATH: &str = "M396.406 180.565C395.606 180.367 394.858 179.997 394.213 179.483C393.568 178.968 393.041 178.321 392.671 177.585C392.299 176.848 392.092 176.04 392.061 175.216C392.031 174.391 392.179 173.571 392.496 172.809C403.427 146.207 408.25 124.922 400.466 116.012C379.851 92.371 297.179 139.382 270.819 155.303C270.113 155.728 269.324 155.995 268.505 156.086C267.687 156.178 266.858 156.091 266.076 155.833C265.294 155.575 264.576 155.152 263.973 154.591C263.369 154.031 262.894 153.346 262.579 152.586C251.499 126.04 239.85 107.576 228.044 106.775C196.75 104.634 171.526 196.337 164.142 226.226C163.945 227.026 163.577 227.775 163.064 228.419C162.55 229.065 161.903 229.591 161.167 229.962C160.432 230.334 159.625 230.541 158.801 230.571C157.977 230.602 157.157 230.454 156.396 230.137C129.794 219.206 108.499 214.383 99.5981 222.167C75.9574 242.782 122.96 325.454 138.881 351.814C139.307 352.519 139.575 353.308 139.667 354.128C139.759 354.946 139.674 355.776 139.416 356.558C139.158 357.341 138.733 358.059 138.172 358.662C137.61 359.266 136.925 359.74 136.163 360.053C109.626 371.133 91.1623 382.782 90.3522 394.588C88.22 425.882 179.915 451.107 209.813 458.491C210.612 458.69 211.358 459.059 212.001 459.573C212.644 460.087 213.169 460.735 213.538 461.47C213.909 462.205 214.117 463.011 214.146 463.834C214.177 464.656 214.03 465.477 213.714 466.236C202.783 492.838 197.96 514.132 205.744 523.034C226.359 546.675 309.041 499.672 335.4 483.751C336.106 483.325 336.896 483.057 337.715 482.965C338.534 482.872 339.363 482.958 340.145 483.217C340.928 483.475 341.645 483.899 342.249 484.461C342.852 485.022 343.327 485.708 343.641 486.469C354.721 513.006 366.36 531.471 378.175 532.281C409.47 534.413 434.693 442.718 442.068 412.82C442.267 412.02 442.637 411.272 443.152 410.629C443.666 409.985 444.314 409.46 445.051 409.089C445.787 408.719 446.595 408.512 447.419 408.483C448.243 408.453 449.063 408.601 449.824 408.919C476.425 419.85 497.711 424.663 506.621 416.888C530.262 396.273 483.25 313.591 467.329 287.232C466.906 286.526 466.64 285.736 466.55 284.919C466.459 284.1 466.546 283.272 466.804 282.49C467.062 281.708 467.485 280.991 468.045 280.388C468.604 279.785 469.288 279.308 470.047 278.992C496.593 267.912 515.057 256.263 515.858 244.457C517.999 213.162 426.295 187.94 396.406 180.565ZM360.503 150.564C366.518 161.346 335.521 233.191 312.467 283.443C312.082 284.283 311.449 284.985 310.652 285.454C309.856 285.923 308.934 286.138 308.012 286.068C307.09 285.998 306.211 285.647 305.495 285.063C304.778 284.478 304.258 283.689 304.003 282.8C294.692 250.127 284.05 211.739 272.663 179.15C272.216 177.871 272.238 176.474 272.726 175.21C273.214 173.946 274.136 172.897 275.325 172.25C303.761 156.719 352.421 136.095 360.503 150.564ZM224.226 159.456C236.098 162.827 264.981 235.547 284.208 287.382C284.529 288.247 284.577 289.191 284.346 290.085C284.114 290.979 283.615 291.781 282.915 292.383C282.214 292.986 281.346 293.359 280.427 293.453C279.508 293.548 278.583 293.359 277.774 292.912C248.063 276.422 213.416 256.776 182.317 241.785C181.1 241.194 180.131 240.191 179.584 238.953C179.036 237.715 178.946 236.323 179.329 235.025C188.481 203.964 208.277 154.94 224.226 159.456ZM134.151 262.11C144.924 256.095 216.778 287.093 267.02 310.147C267.861 310.532 268.563 311.166 269.032 311.963C269.501 312.759 269.716 313.681 269.646 314.602C269.575 315.524 269.225 316.402 268.64 317.119C268.056 317.835 267.267 318.357 266.378 318.611C233.714 327.922 195.316 338.564 162.727 349.952C161.45 350.396 160.055 350.373 158.793 349.885C157.531 349.397 156.483 348.477 155.837 347.288C140.334 318.852 119.673 270.192 134.151 262.11ZM143.043 398.388C146.405 386.516 219.133 357.632 270.968 338.405C271.834 338.085 272.778 338.037 273.672 338.268C274.567 338.5 275.368 338.999 275.971 339.699C276.572 340.4 276.947 341.268 277.041 342.186C277.135 343.105 276.946 344.031 276.499 344.84C260 374.552 240.353 409.198 225.362 440.287C224.777 441.509 223.774 442.482 222.535 443.031C221.296 443.581 219.902 443.671 218.603 443.286C187.541 434.189 138.518 414.338 143.043 398.388ZM245.698 488.463C239.673 477.69 270.679 405.837 293.733 355.594C294.119 354.753 294.753 354.051 295.549 353.582C296.346 353.113 297.267 352.899 298.189 352.969C299.111 353.038 299.99 353.389 300.706 353.973C301.422 354.558 301.943 355.348 302.197 356.237C311.508 388.9 322.151 427.299 333.538 459.887C333.982 461.166 333.957 462.56 333.468 463.823C332.979 465.085 332.056 466.132 330.866 466.777C302.439 482.28 253.77 502.941 245.726 488.463H245.698ZM381.974 479.57C370.093 476.209 341.21 403.481 321.983 351.646C321.661 350.779 321.612 349.833 321.843 348.937C322.074 348.041 322.575 347.238 323.277 346.635C323.979 346.032 324.85 345.659 325.77 345.566C326.691 345.473 327.618 345.665 328.426 346.115C358.129 362.605 392.784 382.261 423.874 397.252C425.095 397.839 426.065 398.842 426.613 400.081C427.161 401.32 427.249 402.713 426.863 404.012C417.719 435.12 397.924 484.095 381.974 479.57ZM472.049 376.916C461.267 382.94 389.423 351.934 339.171 328.88C338.331 328.494 337.628 327.861 337.159 327.064C336.69 326.268 336.477 325.346 336.547 324.425C336.616 323.503 336.966 322.625 337.551 321.908C338.135 321.191 338.925 320.671 339.814 320.417C372.486 311.106 410.876 300.463 443.464 289.075C444.745 288.631 446.141 288.655 447.405 289.145C448.668 289.635 449.717 290.558 450.364 291.748C465.857 320.175 486.519 368.843 472.049 376.916ZM463.157 240.639C459.787 252.52 387.067 281.404 335.233 300.631C334.365 300.953 333.42 301.002 332.524 300.771C331.628 300.539 330.825 300.039 330.222 299.337C329.619 298.635 329.247 297.764 329.154 296.843C329.06 295.923 329.251 294.996 329.702 294.187C346.192 264.485 365.838 229.829 380.829 198.739C381.418 197.521 382.421 196.551 383.66 196.004C384.898 195.456 386.291 195.366 387.589 195.751C418.65 204.894 467.673 224.689 463.157 240.639Z";
pub const ROTOR_VIEWBOX: (f32, f32) = (613., 650.);

/// The rotor mark fitted into a `size`-px square at (`x`, `y`), like `<svg viewBox>` with the
/// default `xMidYMid meet`.
pub fn rotor<'a>(x: f32, y: f32, size: f32, fill: &'a str) -> Svgr<'a> {
    let (vw, vh) = ROTOR_VIEWBOX;
    let s = size / vw.max(vh);
    let place = Similarity { s, dx: x + (size - vw * s) / 2., dy: y + (size - vh * s) / 2. };
    svgr!(<path d={ROTOR_PATH} fill={fill} transform={place.attr()} />)
}

/// Path data of a rounded rectangle, for even-odd cutouts.
pub fn rounded_rect_d(r: Rect, radius: f32) -> String {
    let k = radius.min(r.w / 2.).min(r.h / 2.);
    format!(
        "M{} {}H{}A{k} {k} 0 0 1 {} {}V{}A{k} {k} 0 0 1 {} {}H{}A{k} {k} 0 0 1 {} {}V{}A{k} {k} 0 0 1 {} {}Z",
        r.x + k,
        r.y,
        r.right() - k,
        r.right(),
        r.y + k,
        r.bottom() - k,
        r.right() - k,
        r.bottom(),
        r.x + k,
        r.x,
        r.bottom() - k,
        r.y + k,
        r.x + k,
        r.y,
    )
}

/// A Gaussian blur filter over `region` (user space), padded so the blur is not cut off.
/// `sigma` is the CSS `blur()` value; CSS shadows blur by half their radius.
pub fn blur<'a>(id: String, sigma: f32, region: Rect) -> Svgr<'a> {
    let region = region.outset(3. * sigma);
    svgr!(
        <filter id={id} filterUnits="userSpaceOnUse" x={region.x} y={region.y} width={region.w} height={region.h} color-interpolation-filters="sRGB">
            <feGaussianBlur stdDeviation={sigma} />
        </filter>
    )
}

/// A CSS shadow without offset (`text-shadow: 0 0 <2 * sigma>px`): `shape`, already drawn in
/// the shadow colour, blurred over `area`.
pub fn glow<'a>(id: String, sigma: f32, area: Rect, shape: Svgr<'a>) -> Svgr<'a> {
    let url = format!("url(#{id})");
    svgr!(<g>{blur(id, sigma, area)}<g filter={url}>{shape}</g></g>)
}

/// Gradient stops in CSS terms (offsets may fall outside 0..1); SVG clamps offsets, so the
/// list is resampled at both edges to keep the colours a browser would show.
pub fn stops<'a>(css: &[(f32, Rgba)]) -> Svgr<'a> {
    // CSS moves a stop placed before an earlier one up to that earlier position.
    let css: Vec<(f32, Rgba)> = css
        .iter()
        .scan(f32::NEG_INFINITY, |max, &(offset, color)| {
            *max = max.max(offset);
            Some((*max, color))
        })
        .collect();
    let css = css.as_slice();
    let lerp = |a: Rgba, b: Rgba, t: f32| {
        let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
        Rgba(ch(a.0, b.0), ch(a.1, b.1), ch(a.2, b.2), a.3 + (b.3 - a.3) * t)
    };
    let at = |offset: f32| {
        let next = css.iter().position(|(o, _)| *o >= offset).unwrap_or(css.len() - 1);
        if next == 0 {
            return css[0].1;
        }
        let ((o0, c0), (o1, c1)) = (css[next - 1], css[next]);
        if o1 <= o0 { c1 } else { lerp(c0, c1, ((offset - o0) / (o1 - o0)).clamp(0., 1.)) }
    };
    let mut resolved = vec![(0., at(0.))];
    resolved.extend(css.iter().copied().filter(|(o, _)| *o > 0. && *o < 1.));
    resolved.push((1., at(1.)));
    resolved
        .into_iter()
        .map(|(offset, color)| svgr!(<stop offset={offset} stop-color={color.rgb()} stop-opacity={color.3} />))
        .collect()
}

/// A CSS `box-shadow` (no spread): offset, blur radius and colour.
#[derive(Debug, Clone, Copy)]
pub struct Shadow {
    pub dy: f32,
    pub blur: f32,
    pub color: Rgba,
}

impl Shadow {
    pub const fn new(dy: f32, blur: f32, color: Rgba) -> Self {
        Self { dy, blur, color }
    }

    pub fn draw<'a>(&self, id: String, rect: Rect, radius: f32) -> Svgr<'a> {
        let shape = rect.offset(0., self.dy);
        let filter = format!("url(#{id})");
        svgr!(
            <g>
                {blur(id, self.blur / 2., shape)}
                <rect x={shape.x} y={shape.y} width={shape.w} height={shape.h} rx={radius} fill={self.color.to_string()} filter={filter} />
            </g>
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Bottom,
}

/// A CSS `backdrop-filter: blur()`: the scene behind an element, blurred and shown through it.
#[derive(Clone, Copy)]
pub struct Frost<'s, 'a> {
    pub scene: &'s Svgr<'a>,
    pub sigma: f32,
}

impl<'a> Frost<'_, 'a> {
    /// The blurred scene over `area`, in the coordinates of a box placed by `to_screen`.
    pub fn draw(&self, id: &str, area: Rect, to_screen: Similarity) -> Svgr<'a> {
        let filter = format!("{id}-frost");
        let url = format!("url(#{filter})");
        svgr!(
            <g transform={to_screen.inverse().attr()}>
                {blur(filter, self.sigma, to_screen.apply(area))}
                <g filter={url}>{self.scene.clone()}</g>
            </g>
        )
    }
}

/// A rounded box with a 1px border, optionally one thicker accent edge, shadows and a frosted
/// backdrop: the shape of the window and of every overlay pill and panel.
pub struct Card<'s, 'a> {
    pub id: &'s str,
    pub rect: Rect,
    pub radius: f32,
    pub fill: Rgba,
    pub border: Rgba,
    pub accent: Option<(Edge, f32, Rgba)>,
    /// CSS order: the first shadow is drawn on top.
    pub shadows: &'s [Shadow],
    pub frost: Option<Frost<'s, 'a>>,
    /// Where the card is drawn on screen, for the frosted backdrop.
    pub to_screen: Similarity,
}

impl<'a> Card<'_, 'a> {
    pub fn draw(&self, content: Svgr<'a>) -> Svgr<'a> {
        let Card { id, rect: r, radius, fill, border, accent, shadows, frost, to_screen } = self;
        let clip_id = format!("{id}-clip");
        let clip = format!("url(#{clip_id})");
        let shadows = if shadows.is_empty() {
            Svgr::empty()
        } else {
            // A CSS box-shadow is only painted outside the box.
            let reach = shadows.iter().map(|s| s.dy.abs() + 1.5 * s.blur).fold(0., f32::max);
            let outer = r.outset(reach);
            let outside_id = format!("{id}-outside");
            let outside = format!("url(#{outside_id})");
            let d = format!(
                "M{} {}H{}V{}H{}Z{}",
                outer.x,
                outer.y,
                outer.right(),
                outer.bottom(),
                outer.x,
                rounded_rect_d(*r, *radius)
            );
            let drawn: Svgr = shadows
                .iter()
                .enumerate()
                .rev()
                .map(|(i, shadow)| shadow.draw(format!("{id}-shadow{i}"), *r, *radius))
                .collect();
            svgr!(
                <g>
                    <clipPath id={outside_id}>
                        <path d={d} clip-rule="evenodd" />
                    </clipPath>
                    <g clip-path={outside}>{drawn}</g>
                </g>
            )
        };
        let frost = frost.map_or_else(Svgr::empty, |frost| frost.draw(id, *r, *to_screen));
        let accent = match accent {
            Some((Edge::Left, width, color)) => {
                svgr!(<rect x={r.x} y={r.y} width={*width} height={r.h} fill={color.to_string()} />)
            }
            Some((Edge::Bottom, width, color)) => {
                svgr!(<rect x={r.x} y={r.bottom() - width} width={r.w} height={*width} fill={color.to_string()} />)
            }
            None => Svgr::empty(),
        };
        svgr!(
            <g>
                {shadows}
                <clipPath id={clip_id}>
                    <rect x={r.x} y={r.y} width={r.w} height={r.h} rx={*radius} />
                </clipPath>
                <g clip-path={clip}>
                    {frost}
                    <rect x={r.x} y={r.y} width={r.w} height={r.h} fill={fill.to_string()} />
                    {content}
                    <rect x={r.x + 0.5} y={r.y + 0.5} width={r.w - 1.} height={r.h - 1.} rx={radius - 0.5} fill="none" stroke={border.to_string()} stroke-width="1" />
                    {accent}
                </g>
            </g>
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

impl Anchor {
    fn attr(self) -> &'static str {
        match self {
            Anchor::Start => "start",
            Anchor::Middle => "middle",
            Anchor::End => "end",
        }
    }
}

/// A font at a size with CSS `letter-spacing` in em.
#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub font: Font,
    pub size: f32,
    pub letter_spacing: f32,
}

impl TextStyle {
    pub const fn new(font: Font, size: f32) -> Self {
        Self { font, size, letter_spacing: 0. }
    }

    pub const fn spaced(self, letter_spacing: f32) -> Self {
        Self { letter_spacing, ..self }
    }

    pub fn width(&self, text: &str) -> f32 {
        self.font.width(text, self.size, self.letter_spacing)
    }

    pub fn normal_line_height(&self) -> f32 {
        self.font.normal_line_height(self.size)
    }

    /// Baseline of a line box starting at `top`.
    pub fn baseline(&self, top: f32, line_height: f32) -> f32 {
        top + self.font.baseline(self.size, line_height)
    }

    pub fn draw<'a>(&self, content: String, x: f32, baseline: f32, fill: Rgba, anchor: Anchor) -> Svgr<'a> {
        // CSS letter-spacing trails the last glyph too, which shifts centred text left.
        let x = match anchor {
            Anchor::Start => x,
            Anchor::Middle => x - self.letter_spacing * self.size / 2.,
            Anchor::End => x - self.letter_spacing * self.size,
        };
        svgr!(
            <text x={x} y={baseline} font-family={self.font.family} font-weight={self.font.weight.to_string()} font-size={self.size}
                letter-spacing={self.letter_spacing * self.size} text-anchor={anchor.attr()} fill={fill.to_string()}>
                {content}
            </text>
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_take_css_alpha_suffixes() {
        assert_eq!(Rgba::hex("#EE6018").a8(0x80), Rgba(238, 96, 24, 128. / 255.));
        assert_eq!(Rgba::hex("#181818").alpha(0.5).to_string(), "rgba(24,24,24,0.5)");
    }

    #[test]
    fn similarity_composes_and_inverts() {
        let place = Similarity::scale_about(2., 10., 10.).then(Similarity::translate(5., 0.));
        let r = place.apply(Rect::new(10., 10., 4., 4.));
        assert_eq!(r, Rect::new(15., 10., 8., 8.));
        assert_eq!(place.inverse().apply(r), Rect::new(10., 10., 4., 4.));
    }

    #[test]
    fn rounded_rect_radius_is_capped_by_the_short_side() {
        let d = rounded_rect_d(Rect::new(0., 0., 10., 4.), 8.);
        assert!(d.starts_with("M2 0H8A2 2"), "{d}");
    }
}
