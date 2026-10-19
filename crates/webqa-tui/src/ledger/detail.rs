//! The text of the ledger screen: one row per issue, and the lines of one
//! open issue.

use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{Act, Detail, clue_label};
use webqa_core::interview::ledger::{Entry, instruction};
use webqa_core::judge::op_name;

/// The list row of `entry`.
pub fn row(entry: &Entry, under_cursor: bool) -> String {
    let issue = &entry.issue;
    format!(
        "{}[{}] {} {} {}",
        if under_cursor { "> " } else { "  " },
        if entry.selected { 'x' } else { ' ' },
        issue.number,
        issue.kind().label(),
        instruction(issue),
    )
}

/// The lines of an open issue, `-` for what it lacks.
pub fn lines(entry: &Entry) -> Vec<String> {
    let issue = &entry.issue;
    let path = issue.path.iter().map(act).collect::<Vec<_>>().join(" → ");
    let persona = entry
        .persona_trait
        .as_ref()
        .map(|persona| format!("{}: {}", persona.which.label(), dash(&persona.content)));
    vec![
        format!("指示 {}", instruction(issue)),
        format!("種類 {}", issue.kind().label()),
        format!("操作 {}", dash(&path)),
        format!("性質 {}", persona.as_deref().unwrap_or("-")),
        format!("理由 {}", dash(&reason(&issue.detail))),
        format!("期待 {}", dash(&expectation(&issue.detail))),
    ]
}

/// One operation: its name, then its element.
fn act(act: &Act) -> String {
    let name = op_name(act.op);
    act.target.as_ref().map_or_else(
        || name.clone(),
        |target| format!("{name} {} \"{}\"", target.role, target.name),
    )
}

fn reason(detail: &Detail) -> String {
    match detail {
        Detail::Hesitant { clues, .. } => clue_names(clues),
        Detail::Visibility { reason, .. } => reason.clone(),
        Detail::Unreached { halt, .. } => halt.label().to_owned(),
        Detail::Mismatch { purpose, guesses } => {
            format!("書いた目的 {purpose} / 推測 {}", dash(&guesses.join("、")))
        }
    }
}

fn clue_names(clues: &[Clue]) -> String {
    let names: Vec<_> = clues.iter().map(|clue| clue_label(*clue)).collect();
    names.join("、")
}

fn expectation(detail: &Detail) -> String {
    match detail {
        Detail::Hesitant { expectation, .. } | Detail::Visibility { expectation, .. } => {
            expectation.clone()
        }
        Detail::Unreached { expectation, .. } => expectation.clone().unwrap_or_default(),
        Detail::Mismatch { .. } => String::new(),
    }
}

/// `-` for an empty text.
const fn dash(text: &str) -> &str {
    if text.is_empty() { "-" } else { text }
}
