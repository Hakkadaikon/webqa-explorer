//! The parts of a scene that the files of the Rule of the inputs share: what
//! the judge answers, which pages the trials visit, and how the runs end.

use std::iter;

use serde_json::{Value, json};
use webqa_core::interview::ask::{PURPOSE_MATCH, SAME};
use webqa_core::interview::natural::{NATURAL, PERSONA_TRAIT, YES};
use webqa_core::judge::{Answers, Question};

use crate::judge::{Asked, Said};
use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// What the judge guesses where the scenario does not say.
pub const GUESS: &str = "この画面でするべきこと";

/// The judge answers `answers` next.
pub fn say(scene: &mut Scene, answers: impl IntoIterator<Item = Answers>) {
    scene.replies.extend(answers.into_iter().map(Ok));
}

/// The scene with no answer of the judge and no page after the start page.
pub fn reset(scene: &mut Scene) {
    scene.replies.clear();
    scene.after.clear();
}

/// What the judge answers at a step: the operation `op` with probability
/// `p`, no hesitation, and the purpose `guess`. A step without a guess
/// leaves the question unanswered.
pub fn step(op: &str, p: f64, guess: Option<&str>) -> Answers {
    let said = Said::default()
        .choose("operation", op, p)
        .choose("target", "1", 0.9)
        .choose("error_shown", "true", 0.0)
        .choose("goal_reached", "true", 0.0)
        .choose("hesitation", "true", 0.0)
        .write("expectation", "押すと次の画面に進む")
        .write("hard_to_see", "");
    let said = guess
        .into_iter()
        .fold(said, |said, guess| said.write("purpose_guess", guess));
    said.build()
}

/// A page of the service at `path`, with one link to operate.
pub fn page(path: &str) -> PageSpec {
    PageSpec::at(path).with(PageSpec::link(&format!("次へ {path}")))
}

/// The answer to a mismatch question: one of its two options.
pub fn mismatch_answer(option: &str) -> Answers {
    Said::default().choose(PURPOSE_MATCH, option, 0.9).build()
}

/// The answers to the naturalness questions of one issue.
pub fn natural_answer(option: &str) -> Answers {
    let said = Said::default().choose(NATURAL, option, 0.9);
    said.choose(PERSONA_TRAIT, "ITの慣れ", 0.9).build()
}

/// The answers after the trials that stand in until a scenario gives its
/// own: the guesses mean the written purpose, so nothing more is asked.
pub fn same_answers() -> Vec<Answers> {
    vec![mismatch_answer(SAME)]
}

/// A visit of a trial: the path of the page it shows and the purpose the
/// judge guesses there. `None` leaves the guess unanswered.
pub type Visit = (String, Option<String>);

/// A visit of the page at `path`, where the judge guesses `guess`.
pub fn at(path: &str, guess: &str) -> Visit {
    (path.to_owned(), Some(guess.to_owned()))
}

/// A visit of the page at `path`, where the judge gives no guess.
pub fn unguessed(path: &str) -> Visit {
    (path.to_owned(), None)
}

/// The pages the browser shows after the start page, for a trial that
/// clicks on each page but the last: before a click the page is seen once
/// more, to see that the element is still there, and after it comes the next
/// page.
fn shown_after(first: &PageSpec, rest: &[PageSpec]) -> Vec<PageSpec> {
    let twice = rest.iter().flat_map(|page| [page.clone(), page.clone()]);
    iter::once(first.clone()).chain(twice).collect()
}

/// What the judge says at each of `visits`: it clicks at every page but the
/// last, where it says done.
fn said_at(visits: &[Visit]) -> Vec<Answers> {
    let clicks = iter::repeat_n("click", visits.len().saturating_sub(1));
    let ops = clicks.chain(iter::once("done"));
    let said = |((_, guess), op): (&Visit, &str)| step(op, 0.9, guess.as_deref());
    visits.iter().zip(ops).map(said).collect()
}

/// The pages the trial `trial` (from 0) visits, in order, and what the
/// judge says there. Trials are given in order, and each starts on the
/// first page of the first one.
pub fn journey(scene: &mut Scene, trial: usize, visits: &[Visit]) {
    let pages: Vec<PageSpec> = visits.iter().map(|(path, _)| page(path)).collect();
    let (first, rest) = found(pages.split_first(), "a trial visits a page");
    if trial == 0 {
        scene.start = first.clone();
    }
    assert_eq!(first.path, scene.start.path, "every trial starts alike");
    assert_eq!(scene.after.len(), trial, "the trials are given in order");
    scene.after.push(shown_after(first, rest));
    say(scene, said_at(visits));
}

/// Every trial of the scene visits `visits`, and the judge answers
/// `after` once they are over.
pub fn every_trial(scene: &mut Scene, visits: &[Visit], after: Vec<Answers>) {
    reset(scene);
    let trials = usize::try_from(scene.trials).unwrap_or(usize::MAX);
    for trial in 0..trials {
        journey(scene, trial, visits);
    }
    say(scene, after);
}

/// How many calls of the judge were about a step of a trial.
pub fn step_calls(world: &InterviewWorld) -> usize {
    let about_a_step = |asked: &&Asked| {
        let ids = asked.questions.iter().map(Question::id);
        ids.into_iter().any(|id| id.as_str() == "operation")
    };
    world.ran().judge.asked.iter().filter(about_a_step).count()
}

/// How many trials follow the first one.
pub fn others(scene: &Scene) -> usize {
    usize::try_from(scene.trials.saturating_sub(1)).unwrap_or(0)
}

/// The answers that end a run whose first trial the scene has scripted: the
/// other trials say done at once, and the naturalness question about the
/// issue of the first trial is answered.
pub fn settle(scene: &mut Scene) {
    let done = (0..others(scene)).map(|_| step("done", 0.9, Some("目的を果たす")));
    say(scene, done);
    say(scene, [natural_answer(YES)]);
}

/// How many steps the trials of the scene take: one answer each.
pub fn steps_scripted(scene: &Scene) -> usize {
    let steps = |later: &Vec<PageSpec>| 1 + later.len().saturating_sub(1) / 2;
    scene.after.iter().map(steps).sum()
}

/// Gives the answers after the trials in place of those the scene has.
pub fn answer_after(scene: &mut Scene, answers: Vec<Answers>) {
    scene.replies.truncate(steps_scripted(scene));
    say(scene, answers);
}

/// How the trial `trial` ended, as the log says.
pub fn trial_ending(world: &InterviewWorld, trial: u64) -> Value {
    let lines = world.lines();
    let ended = lines.iter().find(|line| {
        line.get("kind") == Some(&json!("trial_finished"))
            && line.get("trial") == Some(&json!(trial))
    });
    let ended = found(ended, "the trial finished");
    ended.get("ending").cloned().unwrap_or(Value::Null)
}
