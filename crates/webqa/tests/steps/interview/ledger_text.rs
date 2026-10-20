//! Reading the text of a ledger line by line, for the scenarios that
//! check what the text holds, not only what it reads back as.

use crate::scene::found;
use crate::world::InterviewWorld;

/// The prefix of a field line of an issue.
const FIELD: &str = "  - ";

/// The text of the ledger that the run wrote, which a scenario that needs it
/// says is there.
pub fn text(world: &InterviewWorld) -> String {
    let text = world.ran().store.ledger.clone();
    found(text, "the run wrote a ledger text")
}

/// An issue as the text writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The first line: the checkbox, the number and the instruction.
    pub first: String,
    /// The fields after it, by name, in order.
    pub fields: Vec<(String, String)>,
}

impl Block {
    /// The value of the field `name`.
    pub fn field(&self, name: &str) -> Option<&str> {
        let row = self.fields.iter().find(|(field, _)| field == name);
        row.map(|(_, value)| value.as_str())
    }

    /// The names of the fields, in order.
    pub fn names(&self) -> Vec<&str> {
        self.fields.iter().map(|(name, _)| name.as_str()).collect()
    }
}

/// Whether `line` is the first line of an issue: a checkbox and a number.
fn opens(line: &str) -> bool {
    let after = line.strip_prefix("- [");
    let rest = after.and_then(|rest| rest.get(1..));
    rest.is_some_and(|rest| rest.starts_with("] I-"))
}

/// The name and the value of a field line.
fn field(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix(FIELD)?;
    let (name, value) = rest.split_once(": ")?;
    Some((name.to_owned(), value.to_owned()))
}

/// The issues of the text, in the order it writes them. A field line
/// that comes before any issue belongs to none.
pub fn blocks(text: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    for line in text.lines() {
        if opens(line) {
            blocks.push(Block {
                first: line.to_owned(),
                fields: Vec::new(),
            });
        } else if let Some(row) = field(line) {
            blocks
                .last_mut()
                .into_iter()
                .for_each(|b| b.fields.push(row.clone()));
        }
    }
    blocks
}

/// Whether `line` has nothing to say: blank, or the mark of an empty
/// section.
fn is_blank(line: &str) -> bool {
    ["", "なし"].contains(&line)
}

/// Whether `line` is a heading, an item of the head or the first line of
/// an issue, or a field of an issue.
fn is_mark(line: &str) -> bool {
    ["# ", "## ", "- ", FIELD]
        .iter()
        .any(|mark| line.starts_with(mark))
}

/// The lines of the text that are of no kind a ledger has.
pub fn stray(text: &str) -> Vec<String> {
    let strays = text
        .lines()
        .filter(|line| !(is_blank(line) || is_mark(line)));
    strays.map(str::to_owned).collect()
}
