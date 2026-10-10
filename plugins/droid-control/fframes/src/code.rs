//! Syntax highlighting for code annotations in the colours of prism's vsDark theme, with
//! the palette's text colour for unclassified tokens.

use crate::svg::Rgba;
use std::str::FromStr;
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

/// prism-react-renderer's vsDark token colours, keyed by TextMate scope.
const VS_DARK: &[(&str, (u8, u8, u8))] = &[
    ("comment, punctuation.definition.comment", (106, 153, 85)),
    (
        "keyword, storage, constant.language.null, constant.language.undefined, variable.language, support.function.builtin",
        (86, 156, 214),
    ),
    ("keyword.operator, punctuation", (212, 212, 212)),
    ("constant.numeric", (181, 206, 168)),
    ("string, punctuation.definition.string", (206, 145, 120)),
    ("entity.name.tag", (78, 201, 176)),
    ("entity.other.attribute-name", (156, 220, 254)),
    ("entity.name.function, support.function, meta.function-call entity.name", (220, 220, 170)),
    ("entity.name.type, entity.name.class, support.class, support.type, entity.other.inherited-class", (78, 201, 176)),
    ("constant.character", (209, 105, 105)),
];

fn theme(plain: Rgba) -> Theme {
    let color = |(r, g, b): (u8, u8, u8)| Color { r, g, b, a: 255 };
    Theme {
        settings: ThemeSettings { foreground: Some(color((plain.0, plain.1, plain.2))), ..ThemeSettings::default() },
        scopes: VS_DARK
            .iter()
            .map(|(scope, rgb)| ThemeItem {
                scope: ScopeSelectors::from_str(scope).expect("valid scope selectors"),
                style: StyleModifier { foreground: Some(color(*rgb)), ..StyleModifier::default() },
            })
            .collect(),
        ..Theme::default()
    }
}

fn syntax(language: &str) -> &'static SyntaxReference {
    let token = match language.to_ascii_lowercase().as_str() {
        "shell" | "zsh" | "console" => "bash".to_owned(),
        "markup" | "svg" | "xml" => "html".to_owned(),
        other => other.to_owned(),
    };
    SYNTAXES.find_syntax_by_token(&token).unwrap_or_else(|| SYNTAXES.find_syntax_plain_text())
}

/// Coloured tokens per line of `code` (one trailing newline dropped, like the overlay).
pub fn highlight(code: &str, language: &str, plain: Rgba) -> Vec<Vec<(Rgba, String)>> {
    let theme = theme(plain);
    let mut highlighter = HighlightLines::new(syntax(language), &theme);
    let code = code.strip_suffix('\n').unwrap_or(code);
    LinesWithEndings::from(code)
        .map(|line| {
            let tokens = highlighter.highlight_line(line, &SYNTAXES).unwrap_or_default();
            tokens
                .into_iter()
                .map(|(style, text)| {
                    let c = style.foreground;
                    (Rgba(c.r, c.g, c.b, 1.), text.trim_end_matches(['\n', '\r']).to_owned())
                })
                .filter(|(_, text)| !text.is_empty())
                .collect()
        })
        .chain(code.ends_with('\n').then(Vec::new))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: Rgba = Rgba(240, 232, 224, 1.);

    fn color_of(lines: &[Vec<(Rgba, String)>], token: &str) -> Rgba {
        lines.iter().flatten().find(|(_, text)| text.contains(token)).unwrap().0
    }

    #[test]
    fn tsx_tokens_take_vs_dark_colours() {
        let lines = highlight("const answer = \"42\"; // why\n", "tsx", TEXT);
        assert_eq!(lines.len(), 1);
        assert_eq!(color_of(&lines, "const"), Rgba(86, 156, 214, 1.));
        assert_eq!(color_of(&lines, "42"), Rgba(206, 145, 120, 1.));
        assert_eq!(color_of(&lines, "why"), Rgba(106, 153, 85, 1.));
        assert_eq!(color_of(&lines, "answer"), TEXT);
    }

    #[test]
    fn unknown_languages_render_plain_and_keep_blank_lines() {
        let lines = highlight("a\n\nb", "klingon", TEXT);
        assert_eq!(lines.len(), 3);
        assert!(lines[1].is_empty());
        assert!(lines.iter().flatten().all(|(color, _)| *color == TEXT));
    }
}
