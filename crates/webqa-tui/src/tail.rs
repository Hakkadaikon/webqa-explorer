//! Reads the lines appended to the event log since the last read.

use webqa_core::event::{Line, parse_line};

/// Follows a growing event log, chunk by chunk.
#[derive(Debug, Default)]
pub struct Tail {
    /// The bytes after the last newline, kept for the next chunk.
    ///
    /// Bytes, not text: a chunk may end inside a character.
    rest: Vec<u8>,
    skipped: usize,
}

impl Tail {
    /// Reads the lines completed by `chunk`.
    ///
    /// A last part without a newline waits for the next chunk.
    /// Blank lines are dropped; lines that do not parse are counted.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<Line> {
        let bytes = [self.rest.as_slice(), chunk].concat();
        let mut lines: Vec<&[u8]> = bytes.split(|byte| *byte == b'\n').collect();
        // `split` yields at least one piece: the part after the last newline.
        self.rest = lines.pop().unwrap_or_default().to_vec();
        self.read_all(lines)
    }

    fn read_all(&mut self, lines: Vec<&[u8]>) -> Vec<Line> {
        lines
            .into_iter()
            .filter(has_content)
            .filter_map(|line| self.read(line))
            .collect()
    }

    /// The number of lines that did not parse.
    #[must_use]
    pub const fn skipped(&self) -> usize {
        self.skipped
    }

    fn read(&mut self, line: &[u8]) -> Option<Line> {
        // Bytes that are not UTF-8 become U+FFFD and fail to parse.
        parse_line(&String::from_utf8_lossy(line))
            .map_err(|_| self.skipped += 1)
            .ok()
    }
}

/// Whether the line has more than ASCII whitespace.
const fn has_content(line: &&[u8]) -> bool {
    !line.trim_ascii().is_empty()
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
            reader.feed(format!("{}\n{head}", EXAMPLE[0]).as_bytes()),
            vec![line(EXAMPLE[0])],
            "only the line ending in a newline is read"
        );
        assert_eq!(
            reader.feed(format!("{tail}\n").as_bytes()),
            vec![line(EXAMPLE[1])],
            "the rest completes the waiting line"
        );
        assert_eq!(reader.feed(b""), vec![], "an empty chunk reads nothing");
        assert_eq!(reader.skipped(), 0, "a split line is not counted as bad");
    }

    #[test]
    fn chunk_split_inside_a_character_reads_one_line() {
        let text = format!("{}\n", EXAMPLE[1]);
        // One byte into the three bytes of 保.
        let at = text.find('保').unwrap() + 1;
        let (head, tail) = text.as_bytes().split_at(at);
        let mut reader = Tail::default();
        assert_eq!(reader.feed(head), vec![], "half a character waits");
        assert_eq!(
            reader.feed(tail),
            vec![line(EXAMPLE[1])],
            "the joined bytes read as one line"
        );
        assert_eq!(reader.skipped(), 0, "the split line is not counted as bad");
    }

    #[test]
    fn malformed_line_is_skipped_and_counted() {
        let mut reader = Tail::default();
        let chunk = format!("{}\n\n \t\nnot json\n{}\n", EXAMPLE[0], EXAMPLE[1]);
        assert_eq!(
            reader.feed(chunk.as_bytes()),
            vec![line(EXAMPLE[0]), line(EXAMPLE[1])],
            "the lines around the bad one are read"
        );
        assert_eq!(
            reader.skipped(),
            1,
            "only the bad line is counted, not the blank ones"
        );
    }
}
