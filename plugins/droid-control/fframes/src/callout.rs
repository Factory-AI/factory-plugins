//! Callouts: a card of text, with optional keycaps, pointing at a frame region or at text on a
//! terminal clip's screen, or centred on a point. A pointing callout stages in (its box draws
//! around the target, the leader extends, the card slides in) and leaves in reverse.

use crate::cast::{Cast, Span};
use crate::content::Stage;
use crate::fonts::Font;
use crate::motion::{cubic_out, interpolate_eased};
use crate::overlays::{Overlays, frame_of};
use crate::props::Target;
use crate::svg::{Anchor, Card, Edge, Rect, Rgba, Shadow, Similarity, TextStyle, faded, rounded_rect_d};
use crate::timing::FPS;
use fframes::{Svgr, svgr};
use std::collections::HashSet;

const TEXT: TextStyle = TextStyle::new(Font::sans(500), 22.);
const LINE_HEIGHT: f32 = 31.;
const KEY: TextStyle = TextStyle::new(Font::mono(500), 16.);
const KEY_HEIGHT: f32 = 27.;
const KEY_PADDING: f32 = 8.;
const KEY_GAP: f32 = 6.;
const PADDING: (f32, f32) = (18., 12.);
const ACCENT: f32 = 3.;
/// The least space between the target's box and the card.
const GAP: f32 = 40.;
/// Between the card and the screen edge.
const EDGE: f32 = 24.;
/// Frames a box takes to follow its text to a new place.
const GLIDE: f32 = 8.;

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
        let keys: Vec<String> = key.into_iter().flat_map(|key| key.split('+')).filter_map(keycap).collect();
        let lines = TEXT.font.wrap(text, TEXT.size, 0., (0.3 * w).max(240.));
        let text_w = lines.iter().map(|line| TEXT.width(line)).fold(0., f32::max);
        let size =
            (2. * PADDING.0 + ACCENT + keys_width(&keys) + text_w, 2. * PADDING.1 + lines.len() as f32 * LINE_HEIGHT);
        let (enter, exit) = (frame_of(t), frame_of(t + dur));
        let pointing = |clip: Option<usize>, track: Vec<(f32, Rect)>, ink: Vec<Rect>| {
            let rects = stage.clip_rects(enter as usize);
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
                let shown = stage.clip_rects(enter as usize).len();
                let casts = stage.clips[..shown]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, clip)| Some((i, clip, clip.cast.as_ref()?)));
                let found = casts.clone().find_map(|(i, clip, cast)| {
                    // The clip frame shown at `f` (see `Stage::clip_frame`), in cast seconds.
                    let time =
                        |f: f32| (f64::from(f) * stage.speed + 1e-9).floor().min(clip.last_frame as f64) / FPS as f64;
                    let mut replay = cast.replay();
                    replay.advance(time(enter));
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
                        if !replay.advance(time(f as f32)) {
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
fn keycap(name: &str) -> Option<String> {
    let mut chars = name.trim().chars();
    let first = chars.next()?;
    Some(first.to_uppercase().chain(chars).collect())
}

fn keys_width(keys: &[String]) -> f32 {
    if keys.is_empty() {
        return 0.;
    }
    let caps: f32 = keys.iter().map(|key| KEY.width(key) + 2. * KEY_PADDING).sum();
    caps + KEY_GAP * (keys.len() - 1) as f32 + 12.
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
    Rect::new(target.x - 6., target.y - 2., target.w + 12., target.h + 4.)
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

/// The card `gap` away from `target` on `side`, unclamped.
fn beside(side: Side, gap: f32, target: Rect, (w, h): (f32, f32)) -> Rect {
    let (x, y) = match side {
        Side::Right => (target.right() + gap, target.cy() - h / 2.),
        Side::Left => (target.x - gap - w, target.cy() - h / 2.),
        Side::Below => (target.x, target.bottom() + gap),
        Side::Above => (target.x, target.y - gap - h),
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
/// clear of the target while covering, and leading across, the least `ink`. Covered text
/// weighs most, then crossed text, then leader length; ties go to the nearer card and to
/// the earlier of right, below, left, above.
fn placement(target: Rect, size: (f32, f32), ink: &[Rect], screen: (f32, f32)) -> (Side, f32) {
    let safe = Rect::new(EDGE, EDGE, screen.0 - 2. * EDGE, screen.1 - 2. * EDGE);
    let inked = |r: Rect| ink.iter().map(|ink| area(r.intersect(*ink))).sum::<f32>();
    let cost = |side: Side, gap: f32| {
        let wanted = beside(side, gap, target, size);
        let card = place(side, gap, target, size, screen);
        let clear = area(card.intersect(target.outset(GAP / 2.))) <= 0.;
        let crossed: f32 = leader(target, card, side)
            .windows(2)
            .map(|pair| {
                let ((ax, ay), (bx, by)) = (pair[0], pair[1]);
                inked(Rect::new(ax.min(bx), ay.min(by), (bx - ax).abs(), (by - ay).abs()).outset(2.))
            })
            .sum();
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

/// The leader from the box to the card: straight across the gap, or with one jog where the
/// card was pushed off the target's line by the screen edge.
fn leader(target: Rect, card: Rect, side: Side) -> Vec<(f32, f32)> {
    let horizontal = matches!(side, Side::Right | Side::Left);
    let (from, to, along) = match side {
        Side::Right => (target.right(), card.x, target.cy()),
        Side::Left => (target.x, card.right(), target.cy()),
        Side::Below => (target.bottom(), card.y, target.cx()),
        Side::Above => (target.y, card.bottom(), target.cx()),
    };
    let (lo, hi) = if horizontal { (card.y + 14., card.bottom() - 14.) } else { (card.x + 20., card.right() - 20.) };
    let landing = along.clamp(lo, hi.max(lo));
    let point = |a: f32, b: f32| if horizontal { (a, b) } else { (b, a) };
    if (landing - along).abs() < 0.5 {
        return vec![point(from, along), point(to, along)];
    }
    let middle = (from + to) / 2.;
    vec![point(from, along), point(middle, along), point(middle, landing), point(to, landing)]
}

/// The first `fraction` of a polyline's length.
fn partial(points: &[(f32, f32)], fraction: f32) -> Vec<(f32, f32)> {
    let length = |(ax, ay): (f32, f32), (bx, by): (f32, f32)| (bx - ax).hypot(by - ay);
    let mut left = fraction * points.windows(2).map(|pair| length(pair[0], pair[1])).sum::<f32>();
    let mut drawn = vec![points[0]];
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let segment = length(a, b);
        if left >= segment {
            drawn.push(b);
            left -= segment;
        } else {
            let t = if segment > 0. { left / segment } else { 0. };
            drawn.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            break;
        }
    }
    drawn
}

/// Progress through frames `a..b` of a stage, eased.
fn staged(x: f32, a: f32, b: f32) -> f32 {
    interpolate_eased(x, &[a, b], &[0., 1.], cubic_out)
}

impl Overlays<'_> {
    /// Every callout showing at `frame`; `clips` are where the shown clips are on screen.
    pub fn callouts<'a>(&self, callouts: &[Callout], clips: &[Rect], frame: usize) -> Svgr<'a> {
        callouts.iter().map(|callout| self.callout(callout, clips, frame as f32)).collect()
    }

    fn callout<'a>(&self, callout: &Callout, clips: &[Rect], f: f32) -> Svgr<'a> {
        if f < callout.enter || f >= callout.exit {
            return Svgr::empty();
        }
        let (local, remain) = (f - callout.enter, callout.exit - 1. - f);
        let (w, h) = callout.size;
        let (clip, track, side, gap) = match &callout.aim {
            Aim::Point(x, y) => {
                let shown = staged(local, 0., 10.).min(staged(remain, 0., 8.));
                let rect = Rect::new(x - w / 2., y - h / 2., w, h);
                return faded(shown, self.card(callout, rect, Similarity::scale_about(0.96 + 0.04 * shown, *x, *y)));
            }
            Aim::Target { clip, track, side, gap } => (clip, track, *side, *gap),
        };
        let target = glide(track, f);
        let target = boxed(clip.map_or(target, |i| within(target, clips[i])));
        let rect = place(side, gap, target, callout.size, self.size);
        let accent = Rgba::hex(self.palette.accent);

        let drawn = staged(local, 0., 9.).min(staged(remain, 0., 7.));
        let radius = 6_f32.min(target.h / 2.);
        let perimeter = 2. * (target.w + target.h) - (8. - 2. * std::f32::consts::PI) * radius;
        let outline = svgr!(
            <path d={rounded_rect_d(target, radius)} fill={accent.alpha(0.1 * drawn).to_string()} stroke={accent.to_string()}
                stroke-width="2" stroke-dasharray={format!("{} {perimeter}", perimeter * drawn)} />
        );

        let extended = staged(local, 6., 14.).min(staged(remain, 5., 11.));
        let route = partial(&leader(target, rect, side), extended);
        let points = route.iter().map(|(x, y)| format!("{x},{y}")).collect::<Vec<_>>().join(" ");
        let (dot_x, dot_y) = route[0];
        let line = faded(
            4. * extended,
            svgr!(
                <g>
                    <polyline points={points} fill="none" stroke={accent.to_string()} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                    <circle cx={dot_x} cy={dot_y} r="3.5" fill={accent.to_string()} />
                </g>
            ),
        );

        let slid = staged(local, 10., 20.).min(staged(remain, 9., 16.));
        let back = 10. * (1. - slid);
        let (dx, dy) = match side {
            Side::Right => (-back, 0.),
            Side::Left => (back, 0.),
            Side::Below => (0., -back),
            Side::Above => (0., back),
        };
        let card = faded(slid, self.card(callout, rect, Similarity::translate(dx, dy)));
        svgr!(<g>{faded(drawn, outline)}{line}{card}</g>)
    }

    /// The card at `rect`, drawn on screen by `place`: keycaps, then the wrapped text.
    fn card<'a>(&self, callout: &Callout, rect: Rect, place: Similarity) -> Svgr<'a> {
        let palette = self.palette;
        let text = Rgba::hex(palette.text);
        let top = rect.y + PADDING.1;
        let mut x = rect.x + ACCENT + PADDING.0;
        let key_y = top + (LINE_HEIGHT - KEY_HEIGHT) / 2.;
        let caps: Svgr = callout
            .keys
            .iter()
            .map(|key| {
                let cap = Rect::new(x, key_y, KEY.width(key) + 2. * KEY_PADDING, KEY_HEIGHT - 2.);
                x = cap.right() + KEY_GAP;
                let baseline = KEY.baseline(cap.y, cap.h);
                svgr!(
                    <g>
                        <rect x={cap.x} y={cap.y + 2.} width={cap.w} height={cap.h} rx="5" fill={Rgba::BLACK.alpha(0.45).to_string()} />
                        <rect x={cap.x + 0.5} y={cap.y + 0.5} width={cap.w - 1.} height={cap.h - 1.} rx="4.5" fill={Rgba::hex(palette.surface).to_string()}
                            stroke={Rgba::WHITE.alpha(0.22).to_string()} stroke-width="1" />
                        {KEY.draw(key.clone(), cap.cx(), baseline, text, Anchor::Middle)}
                    </g>
                )
            })
            .collect();
        let text_x = rect.x + ACCENT + PADDING.0 + keys_width(&callout.keys);
        let lines: Svgr = callout
            .lines
            .iter()
            .enumerate()
            .map(|(row, line)| {
                let baseline = TEXT.baseline(top + row as f32 * LINE_HEIGHT, LINE_HEIGHT);
                TEXT.draw(line.clone(), text_x, baseline, text, Anchor::Start)
            })
            .collect();
        let card = Card {
            id: &format!("callout{}", callout.id),
            rect,
            radius: 10.,
            fill: Rgba::hex(palette.surface).a8(0xE6),
            border: Rgba::hex(palette.border),
            accent: Some((Edge::Left, ACCENT, Rgba::hex(palette.accent))),
            shadows: &[Shadow::new(8., 32., Rgba(0, 0, 0, 0.35))],
            frost: self.frost(12.),
            to_screen: place,
            screen: self.size,
        };
        svgr!(<g transform={place.attr()}>{card.draw(svgr!(<g>{caps}{lines}</g>))}</g>)
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
    fn leaders_run_straight_and_jog_only_when_the_card_is_pushed_off_line() {
        let target = Rect::new(100., 500., 200., 40.);
        let card = place(Side::Right, GAP, target, (300., 52.), SCREEN);
        assert_eq!(leader(target, card, Side::Right), [(300., 520.), (340., 520.)]);
        let low = Rect::new(100., 1040., 200., 30.);
        let card = place(Side::Right, GAP, low, (300., 52.), SCREEN);
        assert_eq!(card.bottom(), SCREEN.1 - EDGE);
        let route = leader(low, card, Side::Right);
        assert_eq!(route, [(300., 1055.), (320., 1055.), (320., card.bottom() - 14.), (340., card.bottom() - 14.)]);
        assert_eq!(partial(&route, 0.), [(300., 1055.), (300., 1055.)]);
        assert_eq!(partial(&route, 1.), route);
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
