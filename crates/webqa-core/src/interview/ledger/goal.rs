//! The purposes the judge guessed, for a run without a written purpose.

use crate::interview::issue::{Record, Run};

/// The purposes the judge guessed at the first step of each finished
/// trial: trimmed, without blanks, each once, in the order first seen.
#[must_use]
pub fn inferred_goal(run: &Run<'_>) -> Vec<String> {
    let guesses = run
        .trials
        .iter()
        .filter(|trial| trial.outcome.finished())
        .filter_map(first_guess);
    let mut goal: Vec<String> = Vec::new();
    for guess in guesses {
        if !goal.contains(&guess) {
            goal.push(guess);
        }
    }
    goal
}

/// The guess at the first step of `trial`, trimmed. `None` when it is
/// blank, unusable or missing.
fn first_guess(trial: &Record<'_>) -> Option<String> {
    let reply = trial.steps.first().and_then(|step| step.reply)?;
    let guess = reply.purpose_guess.trim();
    (!guess.is_empty()).then(|| guess.to_owned())
}
