//! asciinema recordings (v2 and v3): the terminal size and the output stream, replayed
//! through avt, the emulator agg renders with, so text on screen can be found by time.

use avt::Vt;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Cast {
    pub cols: usize,
    pub rows: usize,
    /// Terminal output at absolute seconds, in order.
    output: Vec<(f64, String)>,
}

/// A run of cells on one screen row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub row: usize,
    pub col: usize,
    pub len: usize,
}

#[derive(Deserialize)]
struct Header {
    version: u8,
    width: Option<usize>,
    height: Option<usize>,
    term: Option<Term>,
}

#[derive(Deserialize)]
struct Term {
    cols: usize,
    rows: usize,
}

impl Cast {
    pub fn read(path: &Path) -> Result<Cast, String> {
        let text =
            fs::read_to_string(path).map_err(|err| format!("clip is not readable: {}: {err}", path.display()))?;
        Cast::parse(&text).map_err(|err| format!("not an asciinema cast: {}: {err}", path.display()))
    }

    pub(crate) fn parse(text: &str) -> Result<Cast, String> {
        let mut lines = text.lines().filter(|line| !line.trim().is_empty());
        let header: Header = serde_json::from_str(lines.next().ok_or("empty file")?).map_err(|err| err.to_string())?;
        // v3 times are intervals since the previous event; v2 times are absolute.
        let relative = match header.version {
            2 => false,
            3 => true,
            version => return Err(format!("unsupported version {version}")),
        };
        let (cols, rows) = match header.term {
            Some(Term { cols, rows }) => (cols, rows),
            None => (header.width.unwrap_or(120), header.height.unwrap_or(36)),
        };
        let mut clock = 0.;
        let mut output = Vec::new();
        for line in lines {
            let event: (f64, String, Value) = serde_json::from_str(line).map_err(|err| format!("{err}: {line}"))?;
            clock = if relative { clock + event.0 } else { event.0 };
            if let ("o", Value::String(data)) = (event.1.as_str(), event.2) {
                output.push((clock, data));
            }
        }
        Ok(Cast { cols, rows, output })
    }

    pub fn replay(&self) -> Replay<'_> {
        Replay { cast: self, vt: Vt::new(self.cols, self.rows), next: 0 }
    }
}

/// The screen of a cast as it plays, moved forward with [`Replay::advance`].
pub struct Replay<'c> {
    cast: &'c Cast,
    vt: Vt,
    next: usize,
}

impl Replay<'_> {
    /// Feeds the output up to `time` seconds; whether any was fed.
    pub fn advance(&mut self, time: f64) -> bool {
        let start = self.next;
        while let Some((_, data)) = self.cast.output.get(self.next).filter(|(at, _)| *at <= time) {
            self.vt.feed_str(data);
            self.next += 1;
        }
        self.next > start
    }

    /// The first occurrence of `text` on screen, top to bottom.
    pub fn find(&self, text: &str) -> Option<Span> {
        let width = text.chars().count();
        self.vt.view().enumerate().find_map(|(row, line)| {
            let (chars, cols): (String, Vec<usize>) = visible(line).map(|(col, c, _)| (c, col)).unzip();
            let at = chars.find(text)?;
            let first = chars[..at].chars().count();
            let last = line.cells()[cols[first + width - 1]].width().max(1) as usize;
            Some(Span { row, col: cols[first], len: cols[first + width - 1] + last - cols[first] })
        })
    }

    /// The text on screen: runs of non-blank cells, joined across the one or two blank cells
    /// between words.
    pub fn ink(&self) -> Vec<Span> {
        let mut runs: Vec<Span> = Vec::new();
        for (row, line) in self.vt.view().enumerate() {
            for (col, c, width) in visible(line) {
                if c.is_whitespace() {
                    continue;
                }
                match runs.last_mut() {
                    Some(run) if run.row == row && run.col + run.len + 2 >= col => run.len = col + width - run.col,
                    _ => runs.push(Span { row, col, len: width }),
                }
            }
        }
        runs
    }
}

/// The column, character and width of every cell that starts a character (the tail cell of a
/// wide character has width 0).
fn visible(line: &avt::Line) -> impl Iterator<Item = (usize, char, usize)> + '_ {
    line.cells()
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.width() > 0)
        .map(|(col, cell)| (col, cell.char(), cell.width() as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_and_v3_casts_give_size_and_absolute_output_times() {
        let v2 = Cast::parse(
            "{\"version\":2,\"width\":20,\"height\":5}\n[0.5,\"o\",\"a\"]\n[0.7,\"i\",\"x\"]\n[1.5,\"o\",\"b\"]\n",
        )
        .unwrap();
        let v3 = Cast::parse("{\"version\":3,\"term\":{\"cols\":20,\"rows\":5}}\n[0.5,\"o\",\"a\"]\n[0.2,\"m\",\"\"]\n[0.8,\"o\",\"b\"]\n").unwrap();
        assert_eq!(v2, v3);
        assert_eq!((v2.cols, v2.rows), (20, 5));
        assert_eq!(v2.output, [(0.5, "a".to_owned()), (1.5, "b".to_owned())]);
        assert!(Cast::parse("{\"version\":1}").is_err());
    }

    #[test]
    fn replay_finds_text_by_cell_through_wide_characters() {
        let cast = Cast::parse(
            "{\"version\":2,\"width\":20,\"height\":3}\n[1,\"o\",\"> \\u001b[1mok\\u001b[0m\\r\\n\"]\n[2,\"o\",\"漢字 mode: auto\"]\n",
        )
        .unwrap();
        let mut replay = cast.replay();
        assert!(!replay.advance(0.5));
        assert_eq!(replay.find("ok"), None);
        assert!(replay.advance(1.));
        assert_eq!(replay.find("ok"), Some(Span { row: 0, col: 2, len: 2 }));
        assert!(replay.advance(5.));
        assert_eq!(replay.find("字 mode"), Some(Span { row: 1, col: 2, len: 7 }));
        assert_eq!(replay.find("auto"), Some(Span { row: 1, col: 11, len: 4 }));
        assert_eq!(replay.ink(), [Span { row: 0, col: 0, len: 4 }, Span { row: 1, col: 0, len: 15 }]);
    }
}
