//! Tests of single effects and of the record helpers.

use std::time::Duration;

use serde_json::json;
use webqa_core::effect::{Effect, Outside, PageEffect, Target, Wake};
use webqa_core::machine::{Facts, Heard, Material, Seen, Whole};
use webqa_core::screen::Screen;

use super::super::fakes::VIEWPORT;
use super::super::log::millis;
use super::super::perform::{observe, perform};
use super::super::*;
use super::{Fakes, page, rules, save};

/// The state the run is in after observing the first page.
async fn started(fakes: &mut Fakes) -> State {
    let seen = observe(&mut fakes.ports()).await.unwrap();
    let start = machine::start(seen, rules(), Memory::default(), Duration::ZERO);
    start.state().clone()
}

#[tokio::test]
async fn sleeping_waits_on_the_clock() {
    let mut fakes = Fakes::new();
    let wake = Effect::Outside(Outside::Sleep(Wake {
        at: Duration::from_secs(5),
    }));
    let state = started(&mut fakes).await;
    let facts = perform(wake, &state, &mut fakes.ports()).await;
    assert_eq!(facts, Ok(Facts::Outside(Heard::Woke)), "woke up");
    assert_eq!(fakes.clock.now, Duration::from_secs(5), "slept until then");
}

#[tokio::test]
async fn looking_observes_the_page_again() {
    let mut fakes = Fakes::new();
    let state = started(&mut fakes).await;
    let look = Effect::Page(PageEffect::Look(Target { element: save() }));
    let facts = perform(look, &state, &mut fakes.ports()).await;
    let observed = page("/");
    let again = observed.screen.clone();
    let outline = Screen {
        url: again.url.clone(),
        elements: observed.outline,
    };
    let expected = Seen {
        screen: fakes.redactor.screen(again),
        viewport: VIEWPORT,
        material: Material {
            paywall: observed.paywall,
            candidates: observed.candidates,
            nodes: observed.nodes,
        },
        whole: Whole {
            outline: fakes.redactor.screen(outline),
            text: fakes.redactor.text(&observed.text),
        },
    };
    assert_eq!(
        facts,
        Ok(Facts::Page(Box::new(expected))),
        "the next page, redacted, with the material of the observation"
    );
}

#[test]
fn run_errors_tell_what_failed() {
    let messages = [
        RunError::Browser(BrowserError).to_string(),
        RunError::Store(StoreError).to_string(),
    ];
    assert_eq!(
        messages,
        [
            "ブラウザを操作できない".to_owned(),
            "記録を書けない".to_owned(),
        ],
        "fixed messages"
    );
}

#[test]
fn elapsed_time_is_logged_in_milliseconds() {
    assert_eq!(
        millis(Duration::from_micros(2_500)),
        2,
        "whole milliseconds"
    );
    assert_eq!(millis(Duration::MAX), u64::MAX, "too long saturates");
}

#[tokio::test]
async fn ports_lend_the_store_and_the_redactor() {
    let mut fakes = Fakes::new();
    let mut ports = fakes.ports();
    let masked = ports.redactor().json(json!("taro@example.test"));
    let written = ports.store().write_event(&masked).await;
    assert_eq!(written, Ok(()), "written through the run's store");
    let lines = fakes.store.lines;
    assert_eq!(lines, [json!("[[メールアドレス]]")], "masked");
}
