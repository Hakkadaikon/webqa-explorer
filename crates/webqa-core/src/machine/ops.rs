//! The operation kinds and what each one does.

#[cfg(test)]
mod tests;

use super::rules::Rules;
use super::state::{ExitReason, StuckKind};
use crate::config::raw::Op;
use crate::explore::ElementOp;

/// One operation kind.
#[derive(Debug, PartialEq)]
pub(super) struct OpRow {
    pub(super) op: Op,
    /// The name the judge and the browser use.
    pub(super) name: &'static str,
    /// What the memory records, for an operation on an element.
    pub(super) element: Option<ElementOp>,
    /// How the run ends, for an operation that is not done.
    pub(super) ends: Option<ExitReason>,
}

/// Every operation kind, in the order offered to the judge.
pub(super) static OPS: [OpRow; 8] = [
    OpRow {
        op: Op::Click,
        name: "click",
        element: Some(ElementOp::Click),
        ends: None,
    },
    OpRow {
        op: Op::Input,
        name: "input",
        element: Some(ElementOp::Input),
        ends: None,
    },
    OpRow {
        op: Op::ScrollDown,
        name: "scroll_down",
        element: None,
        ends: None,
    },
    OpRow {
        op: Op::ScrollUp,
        name: "scroll_up",
        element: None,
        ends: None,
    },
    OpRow {
        op: Op::Back,
        name: "back",
        element: None,
        ends: None,
    },
    OpRow {
        op: Op::Wait,
        name: "wait",
        element: None,
        ends: None,
    },
    OpRow {
        op: Op::Done,
        name: "done",
        element: None,
        ends: Some(ExitReason::ReportedDone),
    },
    OpRow {
        op: Op::Stuck,
        name: "stuck",
        element: None,
        ends: Some(ExitReason::Stuck(StuckKind::Declared)),
    },
];

/// The rows of the operation kinds in `rules`, in the order of the table.
pub(super) fn offered(rules: &Rules) -> Vec<&'static OpRow> {
    OPS.iter()
        .filter(|row| rules.ops.contains(&row.op))
        .collect()
}
