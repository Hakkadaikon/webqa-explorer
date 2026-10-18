//! Reads the lines appended to the event log since the last read.

use webqa_core::event::{Line, parse_line};

/// Follows a growing event log, chunk by chunk.
#[derive(Debug, Default)]
pub struct Tail {
    /// The part after the last newline, kept for the next chunk.
    rest: String,
    skipped: usize,
}

impl Tail {
    /// Reads the lines completed by `chunk`.
    ///
    /// A last part without a newline waits for the next chunk.
    /// Empty lines are dropped; lines that do not parse are counted.
    pub fn feed(&mut self, chunk: &str) -> Vec<Line> {
        let text = format!("{}{chunk}", self.rest);
        let (done, rest) = text.rsplit_once('\n').unwrap_or(("", &text));
        rest.clone_into(&mut self.rest);
        done.lines()
            .filter(|text| !text.is_empty())
            .filter_map(|text| self.read(text))
            .collect()
    }

    /// The number of lines that did not parse.
    #[must_use]
    pub const fn skipped(&self) -> usize {
        self.skipped
    }

    fn read(&mut self, text: &str) -> Option<Line> {
        parse_line(text).map_err(|_| self.skipped += 1).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EXAMPLE;

    fn line(text: &str) -> Line {
        parse_line(text).unwrap()
    }

    #[test]
    fn partial_last_line_waits_for_newline() {
        let (head, tail) = EXAMPLE[1].split_at(10);
        let mut reader = Tail::default();
        assert_eq!(
            reader.feed(&format!("{}\n{head}", EXAMPLE[0])),
            vec![line(EXAMPLE[0])],
            "only the line ending in a newline is read"
        );
        assert_eq!(
            reader.feed(&format!("{tail}\n")),
            vec![line(EXAMPLE[1])],
            "the rest completes the waiting line"
        );
        assert_eq!(reader.feed(""), vec![], "an empty chunk reads nothing");
        assert_eq!(reader.skipped(), 0, "a split line is not counted as bad");
    }

    #[test]
    fn malformed_line_is_skipped_and_counted() {
        let mut reader = Tail::default();
        let chunk = format!("{}\n\nnot json\n{}\n", EXAMPLE[0], EXAMPLE[1]);
        assert_eq!(
            reader.feed(&chunk),
            vec![line(EXAMPLE[0]), line(EXAMPLE[1])],
            "the lines around the bad one are read"
        );
        assert_eq!(reader.skipped(), 1, "only the bad line is counted");
    }
}
