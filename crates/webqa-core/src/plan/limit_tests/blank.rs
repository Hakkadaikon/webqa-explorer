//! Leaving a paywall on the first page, with nothing before it in the
//! history: the `back` lands on the blank page.

use crate::machine::State;
use crate::machine::testing::{away, page, pricing, rules, showed, started, touch};

#[test]
fn a_back_off_the_first_page_to_the_blank_page_records_nothing() {
    let left = started(rules(), &pricing());
    assert_eq!(left.effect(), Some(&touch("back", None, 1.0)), "leaves");
    let blank = showed(left.state(), &page("about:blank", &[]));
    let elsewhere = showed(left.state(), &away());
    let back_home = touch("return", None, 1.0);
    assert_eq!(blank.effect(), Some(&back_home), "goes back to the start");
    assert_eq!(blank.state().notes(), [], "no issue for the blank page");
    let counted = [blank.state(), elsewhere.state()].map(State::counters);
    assert_eq!(counted[0], counted[1], "counted as any page outside");
}
