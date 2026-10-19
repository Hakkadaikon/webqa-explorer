//! The mismatch questions asked after the trials: whether the purposes the
//! judge guessed mean the written screen purposes.

use url::Url;

use super::PURPOSE_MATCH;
use crate::config::purpose::ScreenPurpose;
use crate::interview::outcome::Outcome;
use crate::judge::{Question, QuestionId};

/// The option meaning the guesses mean the written purpose.
pub const SAME: &str = "同じ";
/// The option meaning at least one guess means something else.
pub const DIFFERENT: &str = "同じでない";

/// A step of a trial, as the mismatch questions read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visit<'a> {
    /// The URL of the screen the judge looked at.
    pub url: &'a Url,
    /// The purpose the judge guessed there. `None` when the answers of the
    /// step were unusable.
    pub guess: Option<&'a str>,
}

/// A trial, as the mismatch questions read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trial<'a> {
    /// How the trial ended. Only finished trials count.
    pub outcome: Outcome,
    /// Its steps in order.
    pub visits: &'a [Visit<'a>],
}

/// The mismatch question of one screen purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MismatchQuestion<'a> {
    /// The screen purpose asked about.
    pub screen: &'a ScreenPurpose,
    /// The guesses passed: trimmed, each once, in the order first seen.
    pub guesses: Vec<String>,
    /// The question.
    pub question: Question,
}

/// One question per screen purpose, in the order written, for those with
/// at least one usable guess from a finished trial at a matching screen.
#[must_use]
pub fn mismatch_questions<'a>(
    screens: &'a [ScreenPurpose],
    trials: &[Trial<'_>],
) -> Vec<MismatchQuestion<'a>> {
    screens
        .iter()
        .filter_map(|screen| mismatch_question(screen, trials))
        .collect()
}

/// The question of `screen`, or `None` when no usable guess reached it.
fn mismatch_question<'a>(
    screen: &'a ScreenPurpose,
    trials: &[Trial<'_>],
) -> Option<MismatchQuestion<'a>> {
    let guesses = guesses(screen, trials);
    (!guesses.is_empty()).then(|| MismatchQuestion {
        screen,
        question: question(screen, &guesses),
        guesses,
    })
}

/// The usable guesses at screens matching `screen` in finished trials,
/// trimmed, each once, in the order first seen.
fn guesses(screen: &ScreenPurpose, trials: &[Trial<'_>]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for guess in reached(screen, trials) {
        if !kept.iter().any(|seen| seen == guess) {
            kept.push(guess.to_owned());
        }
    }
    kept
}

/// The trimmed, non-empty guesses at screens matching `screen` in finished
/// trials, in order.
fn reached<'t>(
    screen: &'t ScreenPurpose,
    trials: &'t [Trial<'t>],
) -> impl Iterator<Item = &'t str> {
    finished_visits(trials)
        .filter(|visit| screen.pattern.matches(visit.url))
        .filter_map(|visit| usable(visit.guess))
}

/// The visits of the finished trials, in order.
fn finished_visits<'t>(trials: &'t [Trial<'t>]) -> impl Iterator<Item = &'t Visit<'t>> {
    trials
        .iter()
        .filter(|trial| trial.outcome.finished())
        .flat_map(|trial| trial.visits)
}

/// The guess trimmed, unless there is none or it is blank.
fn usable(guess: Option<&str>) -> Option<&str> {
    guess.map(str::trim).filter(|guess| !guess.is_empty())
}

/// Asks whether `guesses` mean the purpose written for `screen`.
fn question(screen: &ScreenPurpose, guesses: &[String]) -> Question {
    let head = [
        format!(
            "URL の型 {} の画面には、次の目的を書いた。",
            screen.pattern.as_str()
        ),
        format!("書いた目的: {}", screen.purpose),
        "判断モデルは、この型に合う画面の目的を次のように推測した。".to_owned(),
    ];
    let listed = guesses.iter().map(|guess| format!("- {guess}"));
    let tail = "推測は、書いた目的と同じことを指すか。言い回しの違いは問わない。どれか1つでも違うことを指すなら「同じでない」を選ぶ。".to_owned();
    let lines: Vec<String> = head.into_iter().chain(listed).chain([tail]).collect();
    Question::Choice {
        id: QuestionId::new(PURPOSE_MATCH),
        prompt: lines.join("\n"),
        options: vec![SAME.to_owned(), DIFFERENT.to_owned()],
    }
}

#[cfg(test)]
mod tests;
