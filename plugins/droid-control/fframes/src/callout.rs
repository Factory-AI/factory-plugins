//! Callouts: a card of text, with optional keycaps, pointing at a frame region or at text on a
//! terminal clip's screen, or centred on a point. A pointing callout dims the frame around an
//! orange box on its target and grows a line from the card to the box; keycaps press once as
//! the card settles.

use crate::cast::{Cast, Span};
use crate::content::Stage;
use crate::fonts::Font;
use crate::motion::{cubic_out, interpolate_eased, quad_in, smoothstep};
use crate::overlays::Overlays;
use crate::props::Target;
use crate::svg::{Anchor, Card, Edge, Rect, Rgba, Similarity, TextStyle, faded, rounded_rect_d};
use crate::timing::{FPS, frame_of};
use fframes::{Svgr, svgr};
use std::collections::HashSet;

const TEXT: TextStyle = TextStyle::new(Font::sans(300), 30.);
const LINE_HEIGHT: f32 = 38.;
const TAG: TextStyle = TextStyle::new(Font::mono(400), 12.).spaced(0.25);
const TAG_LINE_HEIGHT: f32 = 16.;
/// Between the tag and the text under it.
const TAG_GAP: f32 = 8.;
const KEY: TextStyle = TextStyle::new(Font::mono(400), 30.);
const KEY_HEIGHT: f32 = 2.4 * KEY.size;
const KEY_GAP: f32 = 10.;
/// Around the card's content.
const PADDING: (f32, f32) = (30., 20.);
/// Above and below the keycaps, which run taller than the text beside them.
const KEY_MARGIN: f32 = 12.;
/// Between the keycaps and the text.
const GUTTER: f32 = 26.;
/// How far the frame around pointed-at targets darkens.
const DIM: f32 = 0.5;
/// The least space between the target's box and the card.
const GAP: f32 = 40.;
/// Between the card and the screen edge.
const EDGE: f32 = 24.;
/// The least distance from where the pointer leaves the card to the card's corners.
const INSET: f32 = 40.;
/// Frames a box takes to follow its text to a new place.
const GLIDE: f32 = 8.;
/// Frames into a callout when its keycaps press, how long they stay lit, and how long they
/// take to rise back up.
const PRESS: f32 = 18.;
const LIT: f32 = 18.;
const RELEASE: f32 = 7.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Right,
    Below,
    Left,
    Above,
}

enum Aim {
    Point(f32, f32),
    /// The target from `enter` on, in screen pixels (`clip: None`) or in fractions of a clip's
    /// frame; each entry holds until the next.
    Target {
        clip: Option<usize>,
        track: Vec<(f32, Rect)>,
        side: Side,
        gap: f32,
    },
}

/// A callout laid out once: its card and what it points at.
pub struct Callout {
    id: usize,
    enter: f32,
    exit: f32,
    keys: Vec<String>,
    lines: Vec<String>,
    size: (f32, f32),
    aim: Aim,
}

impl Callout {
    /// Text targets are found on the shown cast clips' screens (the first clip showing the text
    /// at `t` wins) and followed until the callout ends; the card goes where it covers the least
    /// text while the callout shows (see `placement`).
    pub fn new(
        id: usize,
        t: f32,
        dur: f32,
        text: &str,
        at: &Target,
        key: Option<&str>,
        stage: &Stage<'_>,
    ) -> Result<Self, String> {
        let (w, h) = stage.size;
        let keys: Vec<String> = key.into_iter().flat_map(|key| key.split('+')).filter_map(cap_label).collect();
        let lines = TEXT.font.wrap(text, TEXT.size, 0., (0.3 * w).max(240.));
        let text_w = lines.iter().map(|line| TEXT.width(line)).fold(0., f32::max);
        let keyed = !keys.is_empty();
        let height = (text_height(keyed, lines.len()) + 2. * PADDING.1).max(if keyed {
            KEY_HEIGHT + 2. * KEY_MARGIN
        } else {
            0.
        });
        let size = (2. * PADDING.0 + keys_width(&keys) + text_w, height);
        let (enter, exit) = (frame_of(t), frame_of(t + dur));
        let rects = stage.clip_rects(enter as usize);
        let pointing = |clip: Option<usize>, track: Vec<(f32, Rect)>, ink: Vec<Rect>| {
            let to_screen = |r: Rect| clip.map_or(r, |i| within(r, rects[i]));
            let target = boxed(to_screen(track[0].1));
            let ink: Vec<Rect> = ink.into_iter().map(to_screen).collect();
            let (side, gap) = placement(target, size, &ink, stage.size);
            Aim::Target { clip, track, side, gap }
        };
        let aim = match at {
            Target::Point(point) => Aim::Point(point.x.0 * w, point.y.0 * h),
            Target::Region(region) => {
                let rect = Rect::new(region.x.0 * w, region.y.0 * h, region.w.0 * w, region.h.0 * h);
                pointing(None, vec![(enter, rect)], Vec::new())
            }
            Target::Text { text: needle } => {
                let casts = stage.clips[..rects.len()]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, clip)| Some((i, clip, clip.cast.as_ref()?)));
                let found = casts.clone().find_map(|(i, clip, cast)| {
                    // The clip frame shown at `f`, in cast seconds.
                    let time = |f: usize| stage.source_frame(clip, f) as f64 / FPS as f64;
                    let mut replay = cast.replay();
                    replay.advance(time(enter as usize));
                    let first = replay.find(needle)?;
                    // Text where it is relative to the target (the card moves with the target);
                    // text that would sit off the grid is out of the card's reach anyway.
                    let mut ink = HashSet::new();
                    let mut seen = |at: Span, now: Vec<Span>| {
                        let moved = |from: usize, by: (usize, usize)| (from + by.0).checked_sub(by.1);
                        ink.extend(now.into_iter().filter_map(|span| {
                            let row = moved(span.row, (first.row, at.row))?;
                            let col = moved(span.col, (first.col, at.col))?;
                            Some(Span { row, col, ..span })
                        }));
                    };
                    seen(first, replay.ink());
                    let mut track = vec![(enter, first)];
                    for f in (enter as usize + 1)..exit as usize {
                        if !replay.advance(time(f)) {
                            continue;
                        }
                        let at = replay.find(needle).unwrap_or(track[track.len() - 1].1);
                        seen(at, replay.ink());
                        if at != track[track.len() - 1].1 {
                            track.push((f as f32, at));
                        }
                    }
                    let track = track.into_iter().map(|(f, span)| (f, cells(span, cast))).collect();
                    Some(pointing(Some(i), track, ink.into_iter().map(|span| cells(span, cast)).collect()))
                });
                found.ok_or_else(|| match casts.count() {
                    0 => format!("callout target text {needle:?} needs a .cast clip"),
                    _ => format!("callout target text {needle:?} is not on a cast clip's screen at {t}s"),
                })?
            }
        };
        Ok(Callout { id, enter, exit, keys, lines, size, aim })
    }
}

/// A key name as its cap reads, e.g. `"shift"` -> `"Shift"`, `"k"` -> `"K"`.
fn cap_label(name: &str) -> Option<String> {
    let mut chars = name.trim().chars();
    let first = chars.next()?;
    Some(first.to_uppercase().chain(chars).collect())
}

fn cap_width(key: &str) -> f32 {
    1.6 * KEY.size + KEY.width(key)
}

/// The keycaps and the gutter after them.
fn keys_width(keys: &[String]) -> f32 {
    if keys.is_empty() {
        return 0.;
    }
    let caps: f32 = keys.iter().map(|key| cap_width(key)).sum();
    caps + KEY_GAP * (keys.len() - 1) as f32 + GUTTER
}

/// The text column: a tag over the wrapped text when there are keycaps.
fn text_height(keyed: bool, lines: usize) -> f32 {
    let tag = if keyed { TAG_LINE_HEIGHT + TAG_GAP } else { 0. };
    tag + lines as f32 * LINE_HEIGHT
}

/// A run of cells as fractions of an agg frame: agg pads the grid by one cell on the left and
/// right and half a row above and below, so a frame is `cols + 2` cells by `rows + 1`.
fn cells(span: Span, cast: &Cast) -> Rect {
    let (cw, ch) = (1. / (cast.cols + 2) as f32, 1. / (cast.rows + 1) as f32);
    Rect::new((span.col + 1) as f32 * cw, (span.row as f32 + 0.5) * ch, span.len as f32 * cw, ch)
}

/// `r`, in fractions of a frame, where the frame is drawn at `to`.
fn within(r: Rect, to: Rect) -> Rect {
    Rect::new(to.x + r.x * to.w, to.y + r.y * to.h, r.w * to.w, r.h * to.h)
}

fn lerp(a: Rect, b: Rect, t: f32) -> Rect {
    let mix = |x: f32, y: f32| x + (y - x) * t;
    Rect::new(mix(a.x, b.x), mix(a.y, b.y), mix(a.w, b.w), mix(a.h, b.h))
}

fn area(r: Rect) -> f32 {
    r.w.max(0.) * r.h.max(0.)
}

/// The box drawn around a target.
fn boxed(target: Rect) -> Rect {
    Rect::new(target.x - 7., target.y - 1., target.w + 14., target.h + 2.)
}

/// Where the target is at `f`: easing from where it was to where it moved.
fn glide(track: &[(f32, Rect)], f: f32) -> Rect {
    let i = track.partition_point(|(at, _)| *at <= f).max(1) - 1;
    let (at, rect) = track[i];
    match i.checked_sub(1) {
        Some(previous) => lerp(track[previous].1, rect, cubic_out(((f - at) / GLIDE).clamp(0., 1.))),
        None => rect,
    }
}

/// The card `gap` away from `target` on `side`, unclamped: level with the target beside it,
/// and above or below it no further left than lets the pointer drop straight down or up.
fn beside(side: Side, gap: f32, target: Rect, (w, h): (f32, f32)) -> Rect {
    let x = target.x.min(target.cx() - INSET.min(w / 2.));
    let (x, y) = match side {
        Side::Right => (target.right() + gap, target.cy() - h / 2.),
        Side::Left => (target.x - gap - w, target.cy() - h / 2.),
        Side::Below => (x, target.bottom() + gap),
        Side::Above => (x, target.y - gap - h),
    };
    Rect::new(x, y, w, h)
}

/// The card `gap` away from `target` on `side`, kept `EDGE` inside the screen.
fn place(side: Side, gap: f32, target: Rect, size: (f32, f32), (sw, sh): (f32, f32)) -> Rect {
    let card = beside(side, gap, target, size);
    let x = card.x.min(sw - EDGE - card.w).max(EDGE);
    let y = card.y.min(sh - EDGE - card.h).max(EDGE);
    Rect::new(x, y, card.w, card.h)
}

/// Where the card goes: the side and distance from the target that keep it on screen and
/// clear of the target while covering, and pointing across, the least `ink`. Covered text
/// weighs most, then crossed text, then pointer length; ties go to the nearer card and to
/// the earlier of right, below, left, above.
fn placement(target: Rect, size: (f32, f32), ink: &[Rect], screen: (f32, f32)) -> (Side, f32) {
    let safe = Rect::new(EDGE, EDGE, screen.0 - 2. * EDGE, screen.1 - 2. * EDGE);
    let inked = |r: Rect| ink.iter().map(|ink| area(r.intersect(*ink))).sum::<f32>();
    let cost = |side: Side, gap: f32| {
        let wanted = beside(side, gap, target, size);
        let card = place(side, gap, target, size, screen);
        let clear = area(card.intersect(target.outset(GAP / 2.))) <= 0.;
        let [(ax, ay), (bx, by)] = pointer(target, card, side);
        let crossed = inked(Rect::new(ax.min(bx), ay.min(by), (bx - ax).abs(), (by - ay).abs()).outset(2.));
        let offscreen = area(wanted) - area(wanted.intersect(safe));
        (clear, offscreen + 4. * inked(card) + 10. * crossed + 8. * (gap - GAP))
    };
    let reach = screen.0.max(screen.1) / 2.;
    [Side::Right, Side::Below, Side::Left, Side::Above]
        .into_iter()
        .flat_map(|side| {
            (0..).map(|step| GAP + 24. * step as f32).take_while(|gap| *gap <= reach).map(move |gap| (side, gap))
        })
        .map(|(side, gap)| (side, gap, cost(side, gap)))
        .min_by(|(_, _, a), (_, _, b)| b.0.cmp(&a.0).then(a.1.total_cmp(&b.1)))
        .map(|(side, gap, _)| (side, gap))
        .expect("four sides")
}

/// The pointer from the card to the box: it leaves the card's edge facing the box, as near
/// the box's middle as `INSET` from the card's corners allows, and ends at the middle of the
/// box's facing edge.
fn pointer(target: Rect, card: Rect, side: Side) -> [(f32, f32); 2] {
    let along = |from: f32, length: f32, to: f32| {
        let inset = INSET.min(length / 2.);
        to.clamp(from + inset, from + length - inset)
    };
    match side {
        Side::Right => [(card.x, along(card.y, card.h, target.cy())), (target.right(), target.cy())],
        Side::Left => [(card.right(), along(card.y, card.h, target.cy())), (target.x, target.cy())],
        Side::Below => [(along(card.x, card.w, target.cx()), card.y), (target.cx(), target.bottom())],
        Side::Above => [(along(card.x, card.w, target.cx()), card.bottom()), (target.cx(), target.y)],
    }
}

/// Progress through `frames` frames from `start`, shaped by `easing`.
fn eased(x: f32, start: f32, frames: f32, easing: fn(f32) -> f32) -> f32 {
    interpolate_eased(x, &[start, start + frames], &[0., 1.], easing)
}

/// A drawn callout and, when it points at a target, the box the frame stays bright around
/// and how far the rest is dimmed for it.
type Drawn<'a> = (Option<(Rect, f32)>, Svgr<'a>);

impl Overlays<'_> {
    /// Every callout showing at `frame`, over one dimmed frame with a hole at each target's
    /// box; `clips` are where the shown clips are on screen.
    pub fn callouts<'a>(&self, callouts: &[Callout], clips: &[Rect], frame: usize) -> Svgr<'a> {
        let (w, h) = self.size;
        let drawn: Vec<Drawn<'a>> =
            callouts.iter().filter_map(|callout| self.callout(callout, clips, frame as f32)).collect();
        let holes: Vec<(Rect, f32)> = drawn.iter().filter_map(|(hole, _)| *hole).collect();
        let dim = holes.iter().map(|(_, dim)| *dim).fold(0., f32::max);
        let mut d = format!("M0 0H{w}V{h}H0Z");
        for (hole, _) in &holes {
            d.push_str(&rounded_rect_d(*hole, 5.));
        }
        let callouts: Svgr = drawn.into_iter().map(|(_, drawn)| drawn).collect();
        let dimmed = faded(DIM * dim, svgr!(<path d={d} fill="black" fill-rule="evenodd" />));
        svgr!(<g>{dimmed}{callouts}</g>)
    }

    fn callout<'a>(&self, callout: &Callout, clips: &[Rect], f: f32) -> Option<Drawn<'a>> {
        if f < callout.enter || f >= callout.exit {
            return None;
        }
        let (local, total) = (f - callout.enter, callout.exit - callout.enter);
        let shown = eased(local, 0., 9., smoothstep) * (1. - eased(local, total - 10.5, 10.5, quad_in));
        let (w, h) = callout.size;
        let (clip, track, side, gap) = match &callout.aim {
            Aim::Point(x, y) => {
                let rect = Rect::new(x - w / 2., y - h / 2., w, h);
                return Some((None, self.card(callout, rect, local, shown)));
            }
            Aim::Target { clip, track, side, gap } => (clip, track, *side, *gap),
        };
        let target = glide(track, f);
        let target = boxed(clip.map_or(target, |i| within(target, clips[i])));
        let card = place(side, gap, target, callout.size, self.size);
        let pointed = eased(local, 3., 9., smoothstep) * (1. - eased(local, total - 10.5, 9., quad_in));
        let grow = eased(local, 4.5, 12., smoothstep);
        let [from, to] = pointer(target, card, side);
        let end = (from.0 + (to.0 - from.0) * grow, from.1 + (to.1 - from.1) * grow);
        let accent = Rgba::hex(self.palette.accent).to_string();
        let pointing = faded(
            pointed,
            svgr!(
                <g>
                    <path d={rounded_rect_d(target, 5.)} fill={accent.clone()} fill-opacity="0.12" stroke={accent.clone()} stroke-width="2" />
                    <path d={format!("M{} {}L{} {}", from.0, from.1, end.0, end.1)} stroke={accent.clone()} stroke-width="2" />
                    <circle cx={from.0} cy={from.1} r="4" fill={accent.clone()} />
                    <circle cx={end.0} cy={end.1} r={(4. * grow).max(0.5)} fill={accent} />
                </g>
            ),
        );
        let card = self.card(callout, card, local, shown);
        Some((Some((target, pointed)), svgr!(<g>{pointing}{card}</g>)))
    }

    /// The card at `rect`, `opacity` shown and `local` frames into its callout: a solid panel
    /// under an accent bar, holding the keycaps, then a tag over the wrapped text. It pops up
    /// to full size as it fades in.
    fn card<'a>(&self, callout: &Callout, rect: Rect, local: f32, opacity: f32) -> Svgr<'a> {
        if opacity <= 0. {
            return Svgr::empty();
        }
        let palette = self.palette;
        let (ink, muted) = (Rgba::hex(palette.text), Rgba::hex(palette.muted));
        let keyed = !callout.keys.is_empty();
        let lit = (PRESS..PRESS + LIT).contains(&local);
        let press = if lit { (1. - (local - PRESS) / RELEASE).max(0.) } else { 0. };
        let mut x = rect.x + PADDING.0;
        let caps: Svgr = callout
            .keys
            .iter()
            .map(|key| {
                let cap = self.keycap(key, x, rect.cy(), lit, press);
                x += cap_width(key) + KEY_GAP;
                cap
            })
            .collect();
        let text_x = rect.x + PADDING.0 + keys_width(&callout.keys);
        let mut top = rect.cy() - text_height(keyed, callout.lines.len()) / 2.;
        let tag = if keyed {
            let tag = TAG.draw("SHORTCUT".to_owned(), text_x, TAG.baseline(top, TAG_LINE_HEIGHT), muted, Anchor::Start);
            top += TAG_LINE_HEIGHT + TAG_GAP;
            tag
        } else {
            Svgr::empty()
        };
        let lines: Svgr = callout
            .lines
            .iter()
            .enumerate()
            .map(|(row, line)| {
                let baseline = TEXT.baseline(top + row as f32 * LINE_HEIGHT, LINE_HEIGHT);
                TEXT.draw(line.clone(), text_x, baseline, ink, Anchor::Start)
            })
            .collect();
        let place = Similarity::scale_about(0.96 + 0.04 * eased(local, 0., 12., smoothstep), rect.cx(), rect.cy());
        let card = Card {
            id: &format!("callout{}", callout.id),
            rect,
            radius: 6.,
            fill: Rgba::hex(palette.surface).alpha(0.96),
            border: Rgba::hex(palette.border),
            accent: Some((Edge::Top, 2., Rgba::hex(palette.accent))),
            shadows: &[],
            frost: None,
            to_screen: place,
            screen: self.size,
        };
        faded(opacity, svgr!(<g transform={place.attr()}>{card.draw(svgr!(<g>{caps}{tag}{lines}</g>))}</g>))
    }

    /// A keycap of `key` from `x`, centred on `cy`: a cap on a slab it is pushed `press` of
    /// the way down onto, outlined and labelled in the accent while `lit`.
    fn keycap<'a>(&self, key: &str, x: f32, cy: f32, lit: bool, press: f32) -> Svgr<'a> {
        let palette = self.palette;
        let (surface, border) = (Rgba::hex(palette.surface).to_string(), Rgba::hex(palette.border));
        let (edge, label) = if lit {
            (Rgba::hex(palette.accent), Rgba::hex(palette.accent))
        } else {
            (border, Rgba::hex(palette.text))
        };
        let slab = Rect::new(x, cy - KEY_HEIGHT / 2. + 4., cap_width(key), KEY_HEIGHT);
        let cap = Rect::new(x, cy - KEY_HEIGHT / 2. + 3. * press, cap_width(key), KEY_HEIGHT);
        svgr!(
            <g>
                <path d={rounded_rect_d(slab, 4.)} fill={border.to_string()} opacity={1. - press} />
                <path d={rounded_rect_d(cap, 4.)} fill={surface} stroke={edge.to_string()} stroke-width="1" />
                {KEY.draw(key.to_owned(), cap.cx(), KEY.baseline(cap.y, cap.h), label, Anchor::Middle)}
            </g>
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f32, f32) = (1920., 1080.);

    #[test]
    fn cards_go_where_they_cover_the_least_text_nearest_the_target() {
        let target = Rect::new(100., 500., 200., 40.);
        assert_eq!(placement(target, (300., 52.), &[], SCREEN), (Side::Right, GAP));
        // Text beside the target from y = 400 to 700: the nearest clear spot is just above it.
        let text_right = Rect::new(320., 400., 1000., 300.);
        assert_eq!(placement(target, (300., 52.), &[text_right], SCREEN), (Side::Above, GAP + 72.));
        // Near the right edge, a card on the right would be pushed back over the target.
        let edge = Rect::new(1700., 500., 180., 40.);
        assert_eq!(placement(edge, (300., 52.), &[], SCREEN), (Side::Left, GAP));
    }

    #[test]
    fn pointers_run_straight_and_slant_only_when_the_card_is_pushed_off_line() {
        let target = Rect::new(100., 500., 200., 40.);
        let card = place(Side::Right, GAP, target, (300., 100.), SCREEN);
        assert_eq!(pointer(target, card, Side::Right), [(340., 520.), (300., 520.)]);
        // A narrow target: the card shifts left so the pointer still drops straight down.
        let narrow = Rect::new(500., 500., 30., 20.);
        let card = place(Side::Below, GAP, narrow, (300., 100.), SCREEN);
        assert_eq!(pointer(narrow, card, Side::Below), [(515., 560.), (515., 520.)]);
        let low = Rect::new(100., 1040., 200., 30.);
        let card = place(Side::Right, GAP, low, (300., 100.), SCREEN);
        assert_eq!(card.bottom(), SCREEN.1 - EDGE);
        assert_eq!(pointer(low, card, Side::Right), [(340., card.bottom() - INSET), (300., 1055.)]);
    }

    #[test]
    fn boxes_glide_to_where_their_text_moved() {
        let (a, b) = (Rect::new(0., 0., 10., 10.), Rect::new(100., 0., 10., 10.));
        let track = [(0., a), (30., b)];
        assert_eq!(glide(&track, 29.), a);
        assert!((glide(&track, 34.).x - 100. * cubic_out(0.5)).abs() < 1e-3);
        assert_eq!(glide(&track, 38.), b);
    }

    #[test]
    fn agg_cells_sit_inside_its_padding() {
        let cast = Cast::parse(r#"{"version":2,"width":100,"height":28}"#).unwrap();
        let rect = within(cells(Span { row: 2, col: 4, len: 3 }, &cast), Rect::new(0., 0., 102. * 9.6, 29. * 22.4));
        assert!((rect.x - 5. * 9.6).abs() < 1e-3 && (rect.y - 2.5 * 22.4).abs() < 1e-3);
        assert!((rect.w - 3. * 9.6).abs() < 1e-3 && (rect.h - 22.4).abs() < 1e-3);
    }
}
