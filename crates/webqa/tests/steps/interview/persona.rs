//! Steps of this Rule of the feature, and the scenarios that have them.

use webqa_core::interview::natural::{NATURAL, Section};

use crate::judge::Asked;
use crate::questions::replies::{entry_of, ledger_of};
use crate::world::InterviewWorld;

#[path = "persona_asked.rs"]
mod asked;
#[path = "persona_cap.rs"]
mod cap;
#[path = "persona_sections.rs"]
mod sections;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "試行をまとめたあと、課題の番号ごとに1回だけ、そのペルソナとして自然かを別の問いで確かめる",
    "自然さの問いの回数の上限を超えた課題は、確かめられなかった区分に入れる",
    "自然だと判断した課題は、台帳の本体に入れる",
    "自然でないと判断した課題は、本体に入れず、最後の別の節にまとめる",
    "自然さの問いの応答が使えないときは、本体に入れず確かめられなかった区分に入れる",
    "台帳の各課題に、ペルソナのどの性質からそう判断したかを書く",
];

/// The heading of the section of the issues that are natural.
const BODY: &str = "## 課題";
/// The heading of the section of the issues not checked.
const UNCHECKED: &str = "## 確かめられなかった課題";
/// The heading of the section of the issues that are not natural.
const OFF_PERSONA: &str = "## ペルソナから外れると判断した課題";

/// The calls of the judge that asked whether an issue is natural.
fn naturalness(world: &InterviewWorld) -> Vec<&Asked> {
    let asked = world.ran().judge.asked.iter();
    asked
        .filter(|call| call.questions.iter().any(|q| q.id().as_str() == NATURAL))
        .collect()
}

/// The lines of the ledger the run wrote.
fn ledger_lines(world: &InterviewWorld) -> Vec<String> {
    let text = world.ran().store.ledger.clone().unwrap_or_default();
    text.lines().map(str::to_owned).collect()
}

/// The line of the ledger that is `heading`. Nothing there is past the end.
fn heading_at(world: &InterviewWorld, heading: &str) -> usize {
    let lines = ledger_lines(world);
    lines
        .iter()
        .position(|line| line == heading)
        .unwrap_or(usize::MAX)
}

/// The line of the ledger on which the issue `number` starts.
fn issue_at(world: &InterviewWorld, number: &str) -> usize {
    let first = format!("- [ ] {number} ");
    let lines = ledger_lines(world);
    lines
        .iter()
        .position(|line| line.starts_with(&first))
        .unwrap_or(usize::MAX)
}

/// The section of the issue of the scene.
fn section_of(world: &InterviewWorld) -> Section {
    let ledger = ledger_of(world);
    entry_of(&ledger, world.scene.aliases.number("その課題")).section
}
