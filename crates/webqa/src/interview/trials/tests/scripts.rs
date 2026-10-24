//! What the scripted judge of the tests of the loop answers.

use webqa_core::interview::ask::{EXPECTATION, HESITATION, PURPOSE_GUESS};
use webqa_core::judge::QuestionId;
use webqa_judge::scripted::Scripted;

use super::GUESS;

/// The judge chooses the operations `ops` in turn, on the first element,
/// without hesitating.
pub fn script(ops: &[&str]) -> Scripted {
    hesitating(ops, 0.0)
}

/// As [`script`], with the probability `hesitation` that the judge hesitated.
pub fn hesitating(ops: &[&str], hesitation: f64) -> Scripted {
    expecting(ops, hesitation, "保存される")
}

/// As [`hesitating`], with the judge expecting `expectation` at each step.
pub fn expecting(ops: &[&str], hesitation: f64, expectation: &str) -> Scripted {
    let sure: Vec<(&str, f64)> = ops.iter().map(|&op| (op, 0.9)).collect();
    answering(&sure, hesitation, expectation)
}

/// As [`expecting`], with each operation chosen with its probability.
pub fn answering(ops: &[(&str, f64)], hesitation: f64, expectation: &str) -> Scripted {
    ops.iter().fold(Scripted::default(), |script, &chosen| {
        script
            .then(QuestionId::operation(), [chosen])
            .then(QuestionId::target(), [("1", 0.8)])
            .then(QuestionId::error_shown(), [("true", 0.0)])
            .then(QuestionId::goal_reached(), [("true", 0.0)])
            .then(QuestionId::new(HESITATION), [("true", hesitation)])
            .then_text(QuestionId::new(EXPECTATION), expectation)
            .then_text(QuestionId::new(PURPOSE_GUESS), GUESS)
    })
}

/// The judge says the purpose is reached after the first look, `times` times.
pub fn done(times: usize) -> Scripted {
    script(&vec!["done"; times])
}
