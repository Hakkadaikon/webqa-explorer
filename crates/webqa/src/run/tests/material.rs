//! Tests of the material the guardrails judge by.

use webqa_core::effect::Touch;
use webqa_core::machine::{Note, Rejection};

use super::super::*;
use super::{Fakes, one};

/// Runs on a page whose one button `実行` is `listed` as forbidden.
async fn on_forbidden(listed: bool) -> (Fakes, Outcome) {
    let mut fakes = Fakes::new();
    let mut shown = one("実行", &[]);
    for candidate in &mut shown.candidates {
        candidate.forbidden_listed = listed;
    }
    fakes.stay_on(&shown);
    let outcome = fakes.explore().await.unwrap();
    (fakes, outcome)
}

#[tokio::test]
async fn material_comes_from_observation() {
    let (fakes, outcome) = on_forbidden(true).await;
    let last = outcome.state.notes().last();
    let forbidden = Note::Rejected(Rejection::Forbidden);
    assert_eq!(last, Some(&forbidden), "the page says it is forbidden");
    assert_eq!(
        fakes.browser.touches,
        Vec::<Touch>::new(),
        "nothing touched"
    );
    let nodes = &outcome.state.last().material.nodes;
    assert_eq!(nodes, &one("実行", &[]).nodes, "the references of the page");
    let (fakes, _) = on_forbidden(false).await;
    assert_eq!(fakes.browser.touches.len(), 1, "the same page, allowed");
}
