//! Tests of an `enter` chosen in the interview on a page with a forbidden element.

use webqa_core::effect::Touch;

use super::unusable::complete;
use super::*;
use crate::run::tests::{Never, one};

/// Fakes that stay on a page whose one button is listed as forbidden.
fn on_forbidden() -> Fakes {
    let mut fakes = Fakes::new();
    let mut shown = one("実行", &[]);
    for candidate in &mut shown.candidates {
        candidate.forbidden_listed = true;
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

#[tokio::test]
async fn interview_does_not_press_enter_on_a_page_with_a_forbidden_element() {
    let mut fakes = on_forbidden();
    let mut judge = Spy::giving(pressing_enter());
    let mut ports = Ports::new(
        &mut fakes.browser,
        &mut judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    )
    .interviewing(interviewing(&[Op::Click, Op::Enter], false));
    let rules = scenario(&[Op::Click, Op::Enter]);
    explore(&mut ports, &mut Never, rules, Memory::default())
        .await
        .unwrap();
    assert_eq!(
        fakes.browser.touches,
        Vec::<Touch>::new(),
        "enter does not reach the browser"
    );
}
