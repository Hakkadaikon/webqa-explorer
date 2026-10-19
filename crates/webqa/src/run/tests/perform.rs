//! Tests of single effects and of the record helpers.

use webqa_core::effect::{Target, Wake};
use webqa_core::screen::{ElementId, Screen};

use super::super::fakes::VIEWPORT;
use super::super::*;
use super::{Fakes, page};

#[tokio::test]
async fn sleeping_waits_on_the_clock() {
    let mut fakes = Fakes::new();
    let wake = Effect::Outside(Outside::Sleep(Wake {
        at: Duration::from_secs(5),
    }));
    let seen = observe(&mut fakes.ports()).await.unwrap();
    let facts = perform(wake, &seen, &mut fakes.ports()).await;
    assert_eq!(facts, Ok(Facts::Outside(Heard::Woke)), "woke up");
    assert_eq!(fakes.clock.now, Duration::from_secs(5), "slept until then");
}

#[tokio::test]
async fn looking_observes_the_page_again() {
    let mut fakes = Fakes::new();
    let seen = observe(&mut fakes.ports()).await.unwrap();
    let target = Target {
        element: ElementId {
            role: "button".to_owned(),
            name: "保存".to_owned(),
            nth: 0,
        },
    };
    let look = Effect::Page(PageEffect::Look(target));
    let facts = perform(look, &seen, &mut fakes.ports()).await;
    let observed = page("/");
    let again = observed.screen.clone();
    let outline = Screen {
        url: again.url.clone(),
        elements: observed.outline,
    };
    let expected = Seen {
        screen: fakes.redactor.screen(again.clone()),
        viewport: VIEWPORT,
        material: Material::plain(&again, VIEWPORT),
        whole: Whole {
            outline: fakes.redactor.screen(outline),
            text: fakes.redactor.text(&observed.text),
        },
    };
    assert_eq!(
        facts,
        Ok(Facts::Page(Box::new(expected))),
        "the next page, redacted"
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
