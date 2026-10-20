//! The answers a scripted judge gives, and the pieces of a scene that the
//! Rules of the questions, the endings and the persona stand on.

use webqa_core::interview::ask::{EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS};
use webqa_core::interview::issue::{Kind, Number};
use webqa_core::interview::ledger::{Entry, Ledger};
use webqa_core::interview::natural::{NATURAL, PERSONA_TRAIT, YES};
use webqa_core::judge::{Answers, JudgeError, Question, QuestionId};

use crate::judge::Asked;

use crate::judge::Said;
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// A hesitation above the threshold of the Background.
pub const HESITANT: f64 = 0.9;
/// No hesitation.
pub const CALM: f64 = 0.0;
/// The one button of the pages of most scenes.
pub const SAVE: &str = "保存";
/// The trait of the persona the natural answers name.
pub const IT_LITERACY: &str = "ITの慣れ";

/// The answers of one step: the operation `op` on the element `target`,
/// with the hesitation `hesitation`. Every question of the interview is
/// answered.
pub fn step(op: &str, target: Option<&str>, hesitation: f64) -> Answers {
    let said = Said::default()
        .choose("operation", op, 0.9)
        .choose("error_shown", "true", 0.0)
        .choose("goal_reached", "true", 0.0)
        .choose(HESITATION, "true", hesitation)
        .write(EXPECTATION, "進む")
        .write(HARD_TO_SEE, "")
        .write(PURPOSE_GUESS, "買い物をする");
    let said = target
        .iter()
        .fold(said, |said, n| said.choose("target", n, 0.9));
    said.build()
}

/// `answers` with the answer to `id` written as `text`.
pub fn written(mut answers: Answers, id: &str, text: &str) -> Answers {
    let option = (text.to_owned(), f64::NAN);
    answers.table.insert(QuestionId::new(id), vec![option]);
    answers
}

/// `answers` without the answer to `id`.
pub fn without(mut answers: Answers, id: &str) -> Answers {
    answers.table.remove(&QuestionId::new(id));
    answers
}

/// The answers to the naturalness questions: `natural` is `自然` or
/// `自然でない`, and `which` the trait of the persona.
pub fn natural(natural: &str, which: &str) -> Answers {
    Said::default()
        .choose(NATURAL, natural, 0.9)
        .choose(PERSONA_TRAIT, which, 0.9)
        .build()
}

/// The answers of a judge that finds the issue natural.
pub fn natural_yes() -> Answers {
    natural(YES, IT_LITERACY)
}

/// Queues `answers` as the next reply of the judge.
pub fn reply(scene: &mut Scene, answers: Answers) {
    scene.replies.push_back(Ok(answers));
}

/// Queues the `steps` once for every trial of the scene.
pub fn each_trial(scene: &mut Scene, steps: &[Answers]) {
    for _ in 0..scene.trials {
        scene.replies.extend(steps.iter().cloned().map(Ok));
    }
}

/// Adds the button `name` to the start page.
pub fn button(scene: &mut Scene, name: &str) {
    scene.start.elements.push(PageSpec::button(name));
}

/// The number of the hesitation issue on the start page with the buttons
/// `names`, about the button `target` if there is one.
pub fn hesitation_number(names: &[&str], target: Option<&str>) -> Number {
    let elements: Vec<(&str, &str)> = names.iter().map(|name| ("button", *name)).collect();
    let spot = target.map(|name| ("button", name, 0));
    number_of("/", &elements, spot, Kind::Hesitant)
}

/// The prompt of `question`.
pub const fn prompt(question: &Question) -> &String {
    match question {
        Question::Noul { prompt, .. }
        | Question::Choice { prompt, .. }
        | Question::Score { prompt, .. }
        | Question::Text { prompt, .. } => prompt,
    }
}

/// One hesitation issue at the button of the start page, where the persona
/// expected `expectation`: every trial clicks the button, hesitating, and
/// then is done. The judge's answer to the naturalness question is
/// `natural`. The number of the issue stands for `その課題`.
pub fn one_issue(scene: &mut Scene, expectation: &str, natural: Result<Answers, JudgeError>) {
    button(scene, SAVE);
    let hesitant = written(step("click", Some("1"), HESITANT), EXPECTATION, expectation);
    each_trial(scene, &[hesitant, step("done", None, CALM)]);
    scene.replies.push_back(natural);
    let number = hesitation_number(&[SAVE], Some(SAVE));
    scene.aliases.bind("その課題", number);
}

/// The ledger the run wrote.
pub fn ledger_of(world: &InterviewWorld) -> Ledger {
    found(world.ledger(), "the run wrote a ledger")
}

/// The entry of the issue `number`.
pub fn entry_of(ledger: &Ledger, number: Number) -> &Entry {
    let entry = ledger
        .entries
        .iter()
        .find(|entry| entry.issue.number == number);
    found(entry, "the ledger has the issue")
}

/// The first call of the judge of the scene.
pub fn first_call(world: &InterviewWorld) -> &Asked {
    found(world.ran().judge.asked.first(), "the judge was asked")
}

/// The options of the question `id` of the first call.
pub fn options_of(asked: &Asked, id: &str) -> Vec<String> {
    let question = asked.questions.iter().find(|q| q.id().as_str() == id);
    let options = found(question, "the question was asked");
    match options {
        Question::Choice { options, .. } => options.clone(),
        Question::Noul { .. } | Question::Score { .. } | Question::Text { .. } => Vec::new(),
    }
}
