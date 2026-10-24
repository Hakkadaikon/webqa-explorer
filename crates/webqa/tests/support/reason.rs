//! The exit reasons as the feature names them.

use std::str::FromStr;

use webqa_core::machine::{ExitReason, StuckKind};

/// The feature's name of each exit reason.
const NAMES: [(&str, ExitReason); 7] = [
    ("ゴール達成", ExitReason::GoalAchieved),
    ("未検証の終了", ExitReason::ReportedDone),
    ("行き詰まり", ExitReason::Stuck(StuckKind::NoProgress)),
    ("判断予算切れ", ExitReason::BudgetExceeded),
    ("時間切れ", ExitReason::TimeExceeded),
    ("利用者による中断", ExitReason::UserAborted),
    ("判断モデルの停止", ExitReason::JudgeUnavailable),
];

/// An exit reason as the feature names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, cucumber::Parameter)]
#[param(
    name = "reason",
    regex = "ゴール達成|未検証の終了|行き詰まり|判断予算切れ|時間切れ|利用者による中断|判断モデルの停止"
)]
pub struct Named(pub ExitReason);

/// A name that is not one of the seven.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0} is not an exit reason of the feature")]
pub struct Unknown(pub String);

impl FromStr for Named {
    type Err = Unknown;

    fn from_str(name: &str) -> Result<Self, Unknown> {
        let known = NAMES.iter().find(|(known, _)| *known == name);
        known
            .map(|&(_, reason)| Self(reason))
            .ok_or_else(|| Unknown(name.to_owned()))
    }
}

impl Named {
    /// Whether the run ended for this reason. "行き詰まり" is either kind.
    pub fn matches(self, reason: Option<ExitReason>) -> bool {
        if self.0.is_stuck() {
            return reason.is_some_and(ExitReason::is_stuck);
        }
        reason == Some(self.0)
    }
}

/// Checks the seven names and an unknown one.
pub fn check() {
    let names: Vec<_> = NAMES
        .iter()
        .map(|(name, _)| name.parse::<Named>().map(|named| named.0))
        .collect();
    let expected: Vec<_> = NAMES.iter().map(|&(_, reason)| Ok(reason)).collect();
    assert_eq!(names, expected, "every name maps to its reason");
    let unknown = Unknown("どこか".to_owned());
    assert_eq!("どこか".parse::<Named>(), Err(unknown), "an unknown name");
    let sentence = Unknown("x".to_owned()).to_string();
    assert_eq!(
        sentence, "x is not an exit reason of the feature",
        "the sentence"
    );
    check_matches();
}

/// The match of a stuck run is either kind, the others are one reason.
fn check_matches() {
    let stuck = Named(ExitReason::Stuck(StuckKind::NoProgress));
    let declared = Some(ExitReason::Stuck(StuckKind::Declared));
    assert!(stuck.matches(declared), "either kind is stuck");
    assert!(
        !stuck.matches(Some(ExitReason::TimeExceeded)),
        "not another reason"
    );
    let goal = Named(ExitReason::GoalAchieved);
    assert!(
        goal.matches(Some(ExitReason::GoalAchieved)),
        "the same reason"
    );
    assert!(!goal.matches(None), "no reason");
}
