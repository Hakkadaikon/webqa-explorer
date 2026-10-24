//! The counters after a scroll shows an element not seen before, by mode.

use super::page_tests::{after, first, scrolled};
use crate::event::Mode;
use crate::machine::Rules;
use crate::machine::testing::{rules, started};

/// Under `rules`, the counts after scrolling without showing anything new,
/// then scrolling "通知設定" into the viewport.
fn revealing(rules: Rules) -> (u32, u32) {
    let state = started(rules, &first()).state().clone();
    let (state, still) = after(&state, "scroll_down", None, &first());
    assert_eq!(still, (1, 1), "nothing new");
    after(&state, "scroll_down", None, &scrolled()).1
}

#[test]
fn a_revealed_element_is_no_progress_in_an_interview() {
    let interview = Rules {
        mode: Mode::Interview,
        ..rules()
    };
    assert_eq!(
        revealing(interview),
        (0, 2),
        "changed, and still no progress"
    );
}

#[test]
fn a_revealed_element_still_resets_in_a_scenario() {
    assert_eq!(
        revealing(rules()),
        (0, 0),
        "changed, and something new shows"
    );
}
