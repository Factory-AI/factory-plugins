//! Timed overlays on the clip clock: callouts, section sweeps and headers, keystrokes and
//! code annotations. Every panel frosts the scene behind it.

use crate::code::highlight;
use crate::fonts::Font;
use crate::motion::{Bezier, interpolate, interpolate_eased};
use crate::props::{CodeAnnotation, CodePosition, Effect, Keystroke, Section};
use crate::raster::Backdrop;
use crate::svg::{Anchor, Card, Edge, Frost, Rect, Rgba, Shadow, Similarity, TextStyle, blur, faded, stops};
use crate::theme::{Palette, PresetConfig};
use fframes::{Svgr, svgr};

/// Overshooting pop-in shared by callouts and keystrokes.
const POP: Bezier = Bezier(0.34, 1.56, 0.64, 1.);

fn frame_of(seconds: f32) -> f32 {
    (seconds * 30.).round()
}

pub struct Overlays<'s> {
    pub palette: &'s Palette,
    pub config: &'s PresetConfig,
    pub size: (f32, f32),
    /// The scene behind the overlays, for their frosted backdrops.
    pub backdrop: &'s Backdrop,
}

impl<'s> Overlays<'s> {
    fn frost(&self, sigma: f32) -> Option<Frost<'s>> {
        Some(Frost { backdrop: self.backdrop, sigma })
    }

    /// Text pills centred on percentage positions.
    pub fn callouts<'a>(&self, effects: &[Effect], frame: usize) -> Svgr<'a> {
        let (w, h) = self.size;
        let f = frame as f32;
        let style = TextStyle::new(Font::sans(500), 24.);
        let line_height = 24. * 1.4;
        effects
            .iter()
            .enumerate()
            .filter_map(|(i, effect)| {
                let Effect::Callout { t, dur, text, at } = effect else { return None };
                let (enter, exit) = (frame_of(*t), frame_of(t + dur));
                if f < enter || f >= exit {
                    return None;
                }
                let local = f - enter;
                let pop = interpolate_eased(local, &[0., 7.5], &[0., 1.], POP.f());
                let fade = interpolate(local, &[exit - enter - 6., exit - enter], &[1., 0.]);

                // Shrink-to-fit next to `left`, capped at 40% of the width.
                let (left, top) = (at.x.0 * w, at.y.0 * h);
                let (border_x, padding_x) = (1. + 3., 48.);
                let max_content = style.width(text);
                let min_content = text.split_whitespace().map(|word| style.width(word)).fold(0., f32::max);
                let available = w - left - padding_x - border_x;
                let content_w = max_content.min(min_content.max(available)).min(0.4 * w);
                let lines = style.font.wrap(text, style.size, 0., content_w);
                let rect_w = content_w + padding_x + border_x;
                let rect_h = lines.len() as f32 * line_height + 24. + 2.;
                let rect = Rect::new(left - rect_w / 2., top - rect_h / 2., rect_w, rect_h);
                let place = Similarity::scale_about(0.85 + 0.15 * pop, left, top);

                let text_x = rect.x + 3. + 24. + content_w / 2.;
                let fill = Rgba::hex(self.palette.text);
                let text: Svgr = lines
                    .into_iter()
                    .enumerate()
                    .map(|(row, line)| {
                        let baseline = style.baseline(rect.y + 1. + 12. + row as f32 * line_height, line_height);
                        style.draw(line, text_x, baseline, fill, Anchor::Middle)
                    })
                    .collect();
                let card = Card {
                    id: &format!("callout{i}"),
                    rect,
                    radius: 12.,
                    fill: Rgba::hex(self.palette.surface).a8(0xE6),
                    border: Rgba::hex(self.palette.border),
                    accent: Some((Edge::Left, 3., Rgba::hex(self.palette.accent))),
                    shadows: &[Shadow::new(8., 32., Rgba(0, 0, 0, 0.35))],
                    frost: self.frost(12.),
                    to_screen: place,
                    screen: self.size,
                };
                Some(faded(pop.min(fade), svgr!(<g transform={place.attr()}>{card.draw(text)}</g>)))
            })
            .collect()
    }

    /// A frosted band sweeping across the frame at every section boundary after the first.
    pub fn section_sweeps<'a>(&self, sections: &[Section], frame: usize) -> Svgr<'a> {
        let (w, h) = self.size;
        let f = frame as f32;
        let sweep = 15.;
        sections
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(i, section)| {
                let start = frame_of(section.t);
                if f < start || f > start + sweep {
                    return None;
                }
                let local = f - start;
                let x = interpolate_eased(local, &[0., sweep], &[-0.1, 1.1], Bezier(0.4, 0., 0.2, 1.).f());
                let opacity = interpolate(local, &[0., sweep * 0.15, sweep * 0.85, sweep], &[0., 1., 1., 0.]);
                let band = Rect::new(x * w - 0.1 * w, 0., 0.2 * w, h);
                let id = format!("sweep{i}");
                let clip = format!("url(#{id}-clip)");
                let sheen = format!("url(#{id}-sheen)");
                let frost = Frost { backdrop: self.backdrop, sigma: 28. }.draw(band, Similarity::IDENTITY);
                let white = Rgba::WHITE;
                Some(faded(
                    opacity,
                    svgr!(
                        <g>
                            <clipPath id={format!("{id}-clip")}>
                                <rect x={band.x} y={band.y} width={band.w} height={band.h} />
                            </clipPath>
                            <g clip-path={clip}>{frost}</g>
                            <linearGradient id={format!("{id}-sheen")} gradientUnits="userSpaceOnUse" x1={band.x} y1="0" x2={band.right()} y2="0">
                                {stops(&[(0., white.alpha(0.)), (0.5, white.alpha(0.04)), (1., white.alpha(0.))])}
                            </linearGradient>
                            <rect x={band.x} y={band.y} width={band.w} height={band.h} fill={sheen} />
                        </g>
                    ),
                ))
            })
            .collect()
    }

    /// Section titles at the top: each slides in and stays until the next one (or the end of
    /// the content, `end` frames on the clip clock).
    pub fn section_headers<'a>(&self, sections: &[Section], frame: usize, end: usize) -> Svgr<'a> {
        let (w, _) = self.size;
        let f = frame as f32;
        let end = end as f32;
        let style = TextStyle::new(Font::sans(600), 28.).spaced(-0.02);
        sections
            .iter()
            .enumerate()
            .filter_map(|(i, section)| {
                let enter = frame_of(section.t);
                let next = sections.get(i + 1).map_or(end, |next| frame_of(next.t));
                let exit = next.min(end);
                if f < enter || f >= exit {
                    return None;
                }
                let local = f - enter;
                let total = exit - enter;
                let entered = interpolate_eased(local, &[0., 12.], &[0., 1.], Bezier(0.22, 1., 0.36, 1.).f());
                let exited = interpolate_eased(local, &[total - 9., total], &[0., 1.], Bezier(0.32, 0., 0.67, 0.).f());
                let dy = -20. * (1. - entered) + 20. * exited;

                let line_height = style.normal_line_height();
                let rect_w = style.width(&section.title) + 64. + 2.;
                let rect = Rect::new(
                    (w - rect_w) / 2.,
                    (self.config.margin / 2. - 20.).max(30.),
                    rect_w,
                    line_height + 24. + 3.,
                );
                let place = Similarity::translate(0., dy);
                let baseline = style.baseline(rect.y + 1. + 12., line_height);
                let text =
                    style.draw(section.title.clone(), w / 2., baseline, Rgba::hex(self.palette.text), Anchor::Middle);
                let card = Card {
                    id: &format!("section{i}"),
                    rect,
                    radius: 16.,
                    fill: Rgba::hex(self.palette.surface).a8(0xE6),
                    border: Rgba::hex(self.palette.border),
                    accent: Some((Edge::Bottom, 2., Rgba::hex(self.palette.accent).a8(0x80))),
                    shadows: &[Shadow::new(8., 32., Rgba(0, 0, 0, 0.4))],
                    frost: self.frost(12.),
                    to_screen: place,
                    screen: self.size,
                };
                Some(faded(entered * (1. - exited), svgr!(<g transform={place.attr()}>{card.draw(text)}</g>)))
            })
            .collect()
    }

    /// Key pills above the bottom edge; a key gives way to the next one early.
    pub fn keystrokes<'a>(&self, keys: &[Keystroke], frame: usize) -> Svgr<'a> {
        let (w, h) = self.size;
        let f = frame as f32;
        let style = TextStyle::new(Font::mono(500), 22.);
        keys.iter()
            .enumerate()
            .filter_map(|(i, key)| {
                let enter = frame_of(key.t);
                let natural_exit = frame_of(key.t + key.dur.unwrap_or(1.2));
                let exit = keys.get(i + 1).map_or(natural_exit, |next| natural_exit.min(frame_of(next.t)));
                if f < enter || f >= exit {
                    return None;
                }
                let local = f - enter;
                let pop = interpolate_eased(local, &[0., 6.], &[0., 1.], POP.f());
                let fade = interpolate(local, &[exit - enter - 4.5, exit - enter], &[1., 0.]);

                let line_height = style.normal_line_height();
                let rect_w = style.width(&key.label) + 40. + 2.;
                let rect_h = line_height + 16. + 2.;
                let bottom = (self.config.margin + 40.).max(50.);
                let rect = Rect::new((w - rect_w) / 2., h - bottom - rect_h, rect_w, rect_h);
                let place = Similarity::scale_about(0.8 + 0.2 * pop, rect.cx(), rect.cy());
                let baseline = style.baseline(rect.y + 1. + 8., line_height);
                let text =
                    style.draw(key.label.clone(), w / 2., baseline, Rgba::hex(self.palette.text), Anchor::Middle);
                let card = Card {
                    id: &format!("key{i}"),
                    rect,
                    radius: 8.,
                    fill: Rgba::hex(self.palette.surface).a8(0xBF),
                    border: Rgba::hex(self.palette.border),
                    accent: None,
                    shadows: &[],
                    frost: self.frost(8.),
                    to_screen: place,
                    screen: self.size,
                };
                Some(faded(pop.min(fade), svgr!(<g transform={place.attr()}>{card.draw(text)}</g>)))
            })
            .collect()
    }

    pub fn code_cards<'a>(&self, cards: &[CodeCard], frame: usize) -> Svgr<'a> {
        cards.iter().enumerate().map(|(i, card)| self.code_card(i, card, frame as f32)).collect()
    }

    fn code_card<'a>(&self, index: usize, card: &CodeCard, f: f32) -> Svgr<'a> {
        if f < card.enter || f >= card.exit {
            return Svgr::empty();
        }
        let local = f - card.enter;
        let total = card.exit - card.enter;
        let entered = interpolate_eased(local, &[0., 10.5], &[0., 1.], Bezier::EXPO_OUT.f());
        let exited = interpolate(local, &[(total - 10.5).max(0.), total], &[0., 1.]);
        let rect = card.rect;
        let place = Similarity::scale_about(0.96 + 0.04 * entered, rect.cx(), rect.cy());
        let palette = self.palette;
        let muted = Rgba::hex(palette.muted);
        let accent = Rgba::hex(palette.accent);
        let id = format!("code{index}");

        let content_x = rect.x + 1. + 16.;
        let content_y = rect.y + 1. + 16.;
        let title: Svgr = card
            .title
            .iter()
            .enumerate()
            .map(|(row, line)| {
                let line_height = CODE_TITLE.normal_line_height();
                let baseline = CODE_TITLE.baseline(content_y + row as f32 * line_height, line_height);
                CODE_TITLE.draw(line.clone(), content_x, baseline, muted, Anchor::Start)
            })
            .collect();

        let code_y = content_y + card.title_height();
        let pre = Rect::new(content_x, code_y, card.content_w, card.lines.len() as f32 * CODE_LINE_HEIGHT);
        let advance = CODE.width("0");
        let gutter_w = (card.lines.len().to_string().len() + 1) as f32 * advance;
        let text_x = content_x + 2.;
        let rows: Svgr = card
            .lines
            .iter()
            .enumerate()
            .map(|(row, line)| {
                let top = code_y + row as f32 * CODE_LINE_HEIGHT;
                let baseline = CODE.baseline(top, CODE_LINE_HEIGHT);
                // The row's left border falls outside the clipped <pre>; only its tint shows.
                let tint = if line.highlighted {
                    svgr!(<rect x={pre.x} y={top} width={pre.w} height={CODE_LINE_HEIGHT} fill={accent.a8(0x22).to_string()} />)
                } else {
                    Svgr::empty()
                };
                let number = CODE.draw((row + 1).to_string(), text_x + gutter_w, baseline, muted, Anchor::End);
                let tokens: Svgr = line
                    .runs
                    .iter()
                    .map(|(column, color, run)| {
                        CODE.draw(run.clone(), text_x + gutter_w + 12. + *column as f32 * advance, baseline, *color, Anchor::Start)
                    })
                    .collect();
                let text = svgr!(<g>{number}{tokens}</g>);
                let text = if line.focused {
                    text
                } else {
                    let filter = format!("{id}-unfocused{row}");
                    let url = format!("url(#{filter})");
                    let area = Rect::new(pre.x, top, pre.w, CODE_LINE_HEIGHT);
                    svgr!(<g opacity="0.3">{blur(filter, 1.2, area)}<g filter={url}>{text}</g></g>)
                };
                svgr!(<g>{tint}{text}</g>)
            })
            .collect();
        let pre_clip = format!("{id}-pre");
        let pre_url = format!("url(#{pre_clip})");
        let ring = rect.outset(0.5);

        let panel = Card {
            id: &id,
            rect,
            radius: 12.,
            fill: Rgba::hex(palette.surface).a8(0xF2),
            border: Rgba::hex(palette.border),
            accent: Some((Edge::Bottom, 2., accent.a8(0xAA))),
            shadows: &[Shadow::new(18., 48., Rgba(0, 0, 0, 0.45))],
            frost: self.frost(10.),
            to_screen: place,
            screen: self.size,
        };
        let drawn = svgr!(
            <g>
                <rect x={ring.x} y={ring.y} width={ring.w} height={ring.h} rx="12.5" fill="none" stroke={Rgba::WHITE.alpha(0.04).to_string()} stroke-width="1" />
                {panel.draw(svgr!(
                    <g>
                        {title}
                        <clipPath id={pre_clip}>
                            <rect x={pre.x} y={pre.y} width={pre.w} height={pre.h} />
                        </clipPath>
                        <g clip-path={pre_url}>{rows}</g>
                    </g>
                ))}
            </g>
        );
        let sigma = 6. * (1. - entered);
        let drawn = if sigma > 0.01 {
            let filter = format!("{id}-enter");
            let url = format!("url(#{filter})");
            svgr!(<g>{blur(filter, sigma, rect.outset(60.))}<g filter={url}>{drawn}</g></g>)
        } else {
            drawn
        };
        faded(entered * (1. - exited), svgr!(<g transform={place.attr()}>{drawn}</g>))
    }
}

const CODE: TextStyle = TextStyle::new(Font::mono(400), 14.);
const CODE_LINE_HEIGHT: f32 = 14. * 1.5;
const CODE_TITLE: TextStyle = TextStyle::new(Font::sans(500), 11.).spaced(0.1);

pub struct CodeLine {
    /// Non-blank runs: column, colour, text.
    runs: Vec<(usize, Rgba, String)>,
    highlighted: bool,
    focused: bool,
}

/// A code annotation laid out once: wrapped title, highlighted lines and the panel rect.
pub struct CodeCard {
    enter: f32,
    exit: f32,
    title: Vec<String>,
    lines: Vec<CodeLine>,
    content_w: f32,
    rect: Rect,
}

impl CodeCard {
    pub fn new(annotation: &CodeAnnotation, palette: &Palette, config: &PresetConfig, (w, h): (f32, f32)) -> Self {
        let tokens = highlight(&annotation.code, &annotation.language, Rgba::hex(palette.text));
        let has_focus = !annotation.focus.is_empty();
        let lines: Vec<CodeLine> = tokens
            .into_iter()
            .enumerate()
            .map(|(i, tokens)| {
                let number = i as u32 + 1;
                CodeLine {
                    runs: columns(&tokens),
                    highlighted: annotation.highlight.iter().any(|range| range.contains(number)),
                    focused: !has_focus || annotation.focus.iter().any(|range| range.contains(number)),
                }
            })
            .collect();

        let advance = CODE.width("0");
        let gutter_w = (lines.len().to_string().len() + 1) as f32 * advance + 12.;
        let line_w = |line: &CodeLine| {
            line.runs.last().map_or(0., |(column, _, run)| (column + run.chars().count()) as f32 * advance)
        };
        let code_w = 2. + gutter_w + lines.iter().map(line_w).fold(0., f32::max);
        let title = annotation.title.to_uppercase();
        let max_content = code_w.max(CODE_TITLE.width(&title));
        let min_content = code_w.max(title.split_whitespace().map(|word| CODE_TITLE.width(word)).fold(0., f32::max));
        let edge = (config.margin + 24.).max(40.);
        let chrome = 32. + 2.;
        let available = match annotation.position {
            CodePosition::Center => w / 2. - chrome,
            CodePosition::TopRight | CodePosition::BottomLeft => w - edge - chrome,
        };
        let content_w = max_content.min(min_content.max(available)).min(0.42 * w).max(320.);
        let title = if title.is_empty() {
            Vec::new()
        } else {
            CODE_TITLE.font.wrap(&title, CODE_TITLE.size, CODE_TITLE.letter_spacing, content_w)
        };

        let mut card = CodeCard {
            enter: (annotation.t * 30.).round(),
            exit: ((annotation.t + annotation.dur) * 30.).round(),
            title,
            lines,
            content_w,
            rect: Rect::new(0., 0., content_w + chrome, 0.),
        };
        card.rect.h = card.title_height() + card.lines.len() as f32 * CODE_LINE_HEIGHT + 32. + 1. + 2.;
        (card.rect.x, card.rect.y) = match annotation.position {
            CodePosition::Center => ((w - card.rect.w) / 2., (h - card.rect.h) / 2.),
            CodePosition::TopRight => (w - edge - card.rect.w, edge),
            CodePosition::BottomLeft => (edge, h - edge - card.rect.h),
        };
        card
    }

    fn title_height(&self) -> f32 {
        if self.title.is_empty() { 0. } else { self.title.len() as f32 * CODE_TITLE.normal_line_height() + 10. }
    }
}

/// Splits highlighted tokens into non-blank runs at their column (tabs stop every 8).
fn columns(tokens: &[(Rgba, String)]) -> Vec<(usize, Rgba, String)> {
    let mut runs: Vec<(usize, Rgba, String)> = Vec::new();
    let mut column = 0;
    for (color, text) in tokens {
        let mut open = false;
        for c in text.chars() {
            match c {
                '\t' => {
                    column = (column / 8 + 1) * 8;
                    open = false;
                }
                c if c.is_whitespace() => {
                    column += 1;
                    open = false;
                }
                c => {
                    match runs.last_mut() {
                        Some((_, _, run)) if open => run.push(c),
                        _ => runs.push((column, *color, c.to_string())),
                    }
                    open = true;
                    column += 1;
                }
            }
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_split_on_blanks_and_colour_changes() {
        let red = Rgba(255, 0, 0, 1.);
        let blue = Rgba(0, 0, 255, 1.);
        let runs = columns(&[(red, "\tlet x".to_owned()), (blue, "=1".to_owned())]);
        assert_eq!(runs, [(8, red, "let".to_owned()), (12, red, "x".to_owned()), (13, blue, "=1".to_owned())]);
    }
}
