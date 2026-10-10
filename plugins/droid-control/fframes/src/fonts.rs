//! The embedded Geist families: registered with fframes for drawing and measured here for
//! layout, so boxes are sized from the same font files the renderer draws with.

use fframes::RawFontData;
use fframes::ttf_parser::{Face, GlyphId};
use std::sync::{Arc, LazyLock};

pub const SANS: &str = "Geist";
pub const MONO: &str = "Geist Mono";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Font {
    pub family: &'static str,
    pub weight: u16,
}

struct Embedded {
    file: &'static str,
    font: Font,
    bytes: &'static [u8],
}

macro_rules! embedded {
    ($file:literal, $family:expr, $weight:literal) => {
        Embedded {
            file: $file,
            font: Font { family: $family, weight: $weight },
            bytes: include_bytes!(concat!("../media/", $file)),
        }
    };
}

const EMBEDDED: [Embedded; 7] = [
    embedded!("Geist-Light.ttf", SANS, 300),
    embedded!("Geist-Regular.ttf", SANS, 400),
    embedded!("Geist-Medium.ttf", SANS, 500),
    embedded!("Geist-SemiBold.ttf", SANS, 600),
    embedded!("Geist-Bold.ttf", SANS, 700),
    embedded!("GeistMono-Regular.ttf", MONO, 400),
    embedded!("GeistMono-Medium.ttf", MONO, 500),
];

static FACES: LazyLock<Vec<(Font, Face<'static>)>> = LazyLock::new(|| {
    EMBEDDED.iter().map(|e| (e.font, Face::parse(e.bytes, 0).expect("embedded fonts parse"))).collect()
});

/// The font files for the fframes media provider.
pub fn font_data() -> Vec<RawFontData> {
    EMBEDDED.iter().map(|e| RawFontData { file_name: e.file.to_owned(), data: Arc::new(e.bytes) }).collect()
}

impl Font {
    pub const fn sans(weight: u16) -> Self {
        Self { family: SANS, weight }
    }

    pub const fn mono(weight: u16) -> Self {
        Self { family: MONO, weight }
    }

    fn face(self) -> &'static Face<'static> {
        FACES
            .iter()
            .find(|(font, _)| *font == self)
            .map(|(_, face)| face)
            .unwrap_or_else(|| panic!("{self:?} is not an embedded font"))
    }

    fn em(self, units: i16) -> f32 {
        f32::from(units) / f32::from(self.face().units_per_em())
    }

    /// Advance width of `text` at `size` px with CSS `letter-spacing` (in em) after every
    /// character.
    pub fn width(self, text: &str, size: f32, letter_spacing: f32) -> f32 {
        let face = self.face();
        let scale = size / f32::from(face.units_per_em());
        text.chars()
            .map(|c| {
                let glyph = face.glyph_index(c).unwrap_or(GlyphId(0));
                f32::from(face.glyph_hor_advance(glyph).unwrap_or(0)) * scale + letter_spacing * size
            })
            .sum()
    }

    /// CSS `line-height: normal`.
    pub fn normal_line_height(self, size: f32) -> f32 {
        let face = self.face();
        (self.em(face.ascender()) - self.em(face.descender()) + self.em(face.line_gap())) * size
    }

    /// Distance from the top of a CSS line box `line_height` px tall to the text baseline.
    pub fn baseline(self, size: f32, line_height: f32) -> f32 {
        let face = self.face();
        let ascent = self.em(face.ascender()) * size;
        let descent = -self.em(face.descender()) * size;
        (line_height - ascent - descent) / 2. + ascent
    }

    /// Greedy word wrap at `max_width`, the way a browser breaks `white-space: normal` text.
    pub fn wrap(self, text: &str, size: f32, letter_spacing: f32, max_width: f32) -> Vec<String> {
        let space = self.width(" ", size, letter_spacing);
        let mut lines: Vec<(String, f32)> = Vec::new();
        for word in text.split_whitespace() {
            let width = self.width(word, size, letter_spacing);
            match lines.last_mut() {
                Some((line, line_width)) if *line_width + space + width <= max_width => {
                    line.push(' ');
                    line.push_str(word);
                    *line_width += space + width;
                }
                _ => lines.push((word.to_owned(), width)),
            }
        }
        lines.into_iter().map(|(line, _)| line).collect()
    }
}

/// Runs of non-blank characters with their column, for monospaced text whose spacing must
/// survive SVG whitespace collapsing (code, the ASCII wordmark).
pub fn mono_runs(line: &str) -> Vec<(usize, String)> {
    let mut runs: Vec<(usize, String)> = Vec::new();
    let mut column = 0;
    for c in line.chars() {
        match c {
            '\t' => column = (column / 8 + 1) * 8,
            c if c.is_whitespace() => column += 1,
            c => {
                match runs.last_mut() {
                    Some((start, run)) if *start + run.chars().count() == column => run.push(c),
                    _ => runs.push((column, c.to_string())),
                }
                column += 1;
            }
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geist_metrics_give_the_css_normal_line_height() {
        let font = Font::sans(400);
        assert!((font.normal_line_height(100.) - 130.).abs() < 0.5);
        assert!(font.baseline(100., 130.) > 95. && font.baseline(100., 130.) < 105.);
    }

    #[test]
    fn width_scales_with_size_and_letter_spacing() {
        let font = Font::sans(700);
        let base = font.width("Droid", 100., 0.);
        assert!((font.width("Droid", 50., 0.) * 2. - base).abs() < 0.01);
        assert!((font.width("Droid", 100., -0.04) - (base - 5. * 4.)).abs() < 0.01);
    }

    #[test]
    fn wrap_breaks_between_words_only() {
        let font = Font::sans(500);
        let one_word = font.width("autonomous", 24., 0.);
        let lines = font.wrap("autonomous engineering droid", 24., 0., one_word + 1.);
        assert_eq!(lines, ["autonomous", "engineering", "droid"]);
        assert_eq!(font.wrap("short", 24., 0., 1.), ["short"]);
    }

    #[test]
    fn geist_mono_draws_the_wordmark_blocks() {
        let face = Font::mono(400).face();
        assert!(face.glyph_index('█').is_some());
    }

    #[test]
    fn mono_runs_keep_columns_through_spaces_and_tabs() {
        assert_eq!(mono_runs("  ab  c\td"), [(2, "ab".to_owned()), (6, "c".to_owned()), (8, "d".to_owned())]);
    }
}
