//! What the ledger screen drew, as the text on it, and what it should say
//! of an issue.

use webqa_core::interview::issue::{Act, Detail, clue_label};
use webqa_core::interview::ledger::{Entry, instruction};
use webqa_core::judge::op_name;

use crate::scene::found;

/// What clears the screen before a frame.
pub const CLEAR: &str = "\x1b[2J";

/// The text of the frame after the last clearing of `drawn`.
pub fn frame_of(drawn: &str) -> String {
    let last = found(drawn.rsplit(CLEAR).next(), "a frame");
    let step = |escaped: &mut bool, c: char| {
        let inside = *escaped || c == '\x1b';
        *escaped = inside && !c.is_ascii_alphabetic();
        Some((!inside).then_some(c))
    };
    let text: String = last.chars().scan(false, step).flatten().collect();
    squeezed(&text)
}

/// `text` without its blanks.
pub fn squeezed(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Whether `frame` shows `text`.
pub fn shows(frame: &str, text: &str) -> bool {
    frame.contains(&squeezed(text))
}

/// What the screen of an open hesitation says of it, line by line, as far
/// as the width of the screen shows it. `None` for another kind.
pub fn detail_lines(entry: &Entry) -> Option<Vec<String>> {
    let issue = &entry.issue;
    let Detail::Hesitant { clues, expectation } = &issue.detail else {
        return None;
    };
    let acts: Vec<String> = issue.path.iter().map(act).collect();
    assert!(!acts.is_empty(), "the issue has the operations before it");
    // The row is cut at the width of the screen: its first words show.
    let said: String = instruction(issue).chars().take(20).collect();
    let persona = entry.persona_trait.as_ref();
    let trait_line = persona.map(|t| format!("性質 {}: {}", t.which.label(), t.content));
    let reasons: Vec<&str> = clues.iter().map(|c| clue_label(*c)).collect();
    Some(vec![
        format!("指示 {said}"),
        format!("操作 {}", acts.join(" → ")),
        found(trait_line, "the persona trait is known"),
        format!("理由 {}", reasons.join("、")),
        format!("期待 {expectation}"),
    ])
}

fn act(act: &Act) -> String {
    let name = op_name(act.op);
    let target = act.target.as_ref();
    target.map_or_else(
        || name.clone(),
        |t| format!("{name} {} \"{}\"", t.role, t.name),
    )
}
