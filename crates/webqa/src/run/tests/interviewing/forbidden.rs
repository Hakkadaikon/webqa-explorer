//! Tests of how the interview treats `enter` on a page with a forbidden element.

use webqa_core::effect::Touch;
use webqa_core::judge::Unusable;
use webqa_core::machine::{Anomaly, Note};

use super::unusable::complete;
use super::*;
use crate::run::tests::{Never, one};

/// Fakes that stay on a page whose one button is `listed` as forbidden.
fn on_forbidden(listed: bool) -> Fakes {
    let mut fakes = Fakes::new();
    let mut shown = one("実行", &[]);
    for candidate in &mut shown.candidates {
        candidate.forbidden_listed = listed;
    }
    fakes.stay_on(&shown);
    fakes
}

/// The answers that choose `enter`, which asks for no element.
fn pressing_enter() -> Answers {
    let mut said = complete();
    said.table
        .insert(QuestionId::operation(), vec![("enter".to_owned(), 0.9)]);
    said.table.remove(&QuestionId::target());
    said
}

/// Interviews on the page, choosing `enter`, and says what reached the browser
/// and the last note.
async fn pressed(listed: bool) -> (Vec<Touch>, Option<Note>) {
    let mut fakes = on_forbidden(listed);
    let mut judge = Spy::giving(pressing_enter());
    let mut ports = Ports::new(
        &mut fakes.browser,
        &mut judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    )
    .interviewing(interviewing(false));
    let rules = scenario(&[Op::Click, Op::Enter]);
    let outcome = explore(&mut ports, &mut Never, rules, Memory::default())
        .await
        .unwrap();
    (fakes.browser.touches, outcome.state.notes().last().cloned())
}

#[tokio::test]
async fn interview_does_not_press_enter_on_a_page_with_a_forbidden_element() {
    let (touches, last) = pressed(true).await;
    let refused = Note::Anomaly(Anomaly::Unusable(Unusable::NotAnOption(
        QuestionId::operation(),
    )));
    assert_eq!(
        touches,
        Vec::<Touch>::new(),
        "enter does not reach the browser"
    );
    assert_eq!(last, Some(refused), "enter is not an option");
    let (touches, _) = pressed(false).await;
    let mut ops: Vec<_> = touches.iter().map(|touch| touch.op.as_str()).collect();
    ops.dedup();
    assert_eq!(ops, ["enter"], "the same page, allowed: enter is pressed");
}

/// The operation kinds the interview's first question offers on the page.
async fn first_offered(listed: bool) -> Option<Vec<String>> {
    let mut fakes = on_forbidden(listed);
    let mut judge = Spy::giving(pressing_enter());
    let mut ports = Ports::new(
        &mut fakes.browser,
        &mut judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    )
    .interviewing(interviewing(false));
    let rules = scenario(&[Op::Click, Op::Enter]);
    explore(&mut ports, &mut Never, rules, Memory::default())
        .await
        .unwrap();
    let first = options(judge.asked.first().unwrap());
    let operation = first
        .into_iter()
        .find(|(id, _)| *id == QuestionId::operation().as_str());
    operation.map(|(_, options)| options)
}

#[tokio::test]
async fn the_interview_leaves_out_enter_where_a_forbidden_element_shows() {
    let offered = (first_offered(true).await, first_offered(false).await);
    let kinds = |kinds: &[&str]| Some(kinds.iter().map(|k| (*k).to_owned()).collect());
    assert_eq!(
        offered,
        (
            kinds(&["click", "done"]),
            kinds(&["click", "done", "enter"])
        ),
        "the operation kinds asked, with and without a forbidden element"
    );
}
