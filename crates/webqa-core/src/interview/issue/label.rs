//! The names of the kinds, clues and halts, shared by the questions and
//! the ledger.

use super::{Halt, Kind};
use crate::interview::clue::Clue;

/// The kinds with their names in the ledger, in this order.
pub const KINDS: [(Kind, &str); 4] = [
    (Kind::Hesitant, "判断に迷う"),
    (Kind::Visibility, "視認性が悪い"),
    (Kind::Unreached, "目的に届かない"),
    (Kind::Mismatch, "目的のずれ"),
];

/// The clues with their names.
pub const CLUES: [(Clue, &str); 4] = [
    (Clue::Hesitant, "迷いの度合いが閾値以上"),
    (Clue::LowConfidence, "確信度が閾値未満の操作を選んだ"),
    (Clue::WentBack, "引き返した"),
    (Clue::PingPong, "同じ2つの画面を行き来した"),
];

/// The halts with their names.
pub const HALTS: [(Halt, &str); 3] = [
    (Halt::GaveUp, "ペルソナが諦めた"),
    (Halt::NoProgress, "進めなくなった"),
    (Halt::BudgetSpent, "判断予算を使い切った"),
];

impl Kind {
    /// The name of the kind in the ledger.
    #[must_use]
    pub fn label(self) -> &'static str {
        lookup(&KINDS, self)
    }
}

impl Halt {
    /// The name of the halt.
    #[must_use]
    pub fn label(self) -> &'static str {
        lookup(&HALTS, self)
    }
}

/// The name of a clue.
#[must_use]
pub fn clue_label(clue: Clue) -> &'static str {
    lookup(&CLUES, clue)
}

/// The name of `key` in `table`, or `""` when the table lacks it.
fn lookup<T: PartialEq + Copy>(table: &[(T, &'static str)], key: T) -> &'static str {
    table
        .iter()
        .find(|(row, _)| *row == key)
        .map_or("", |(_, name)| name)
}
