//! Starting with a window the run already used.

use crate::effect::{Effect, Outside, Wake};
use crate::explore::Memory;
use crate::machine::testing::{nz, rules, secs, seen, top, window};
use crate::machine::{Rules, start, start_in};
use crate::window::RequestWindow;

#[test]
fn start_in_goes_on_with_the_window_it_is_given() {
    let used = window(2, 2, secs(0));
    let few = Rules {
        requests_per_hour: nz(9),
        ..rules()
    };
    let step = start_in(seen(&top()), few, Memory::default(), used, secs(0));
    let sleep = Effect::Outside(Outside::Sleep(Wake { at: secs(3600) }));
    assert_eq!(
        step.effect(),
        Some(&sleep),
        "a full window waits for the next"
    );
    assert_eq!(step.state().window(), used, "the window it was given");
}

#[test]
fn start_makes_a_new_window_from_the_rules() {
    let rules = Rules {
        requests_per_hour: nz(9),
        ..rules()
    };
    let new = RequestWindow::new(nz(9));
    let step = start(seen(&top()), rules, Memory::default(), secs(0));
    assert_eq!(
        step.state().window(),
        new,
        "empty, with the cap of the rules"
    );
}
