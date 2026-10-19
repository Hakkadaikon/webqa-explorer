//! Who asked for the browser operation a state waits on.

use crate::effect::{Effect, Outside, Wake};
use crate::explore::Memory;
use crate::machine::testing::{
    answer, ask_top, away, counters, id, idle, look, next, nz, pricing, rules, secs, seen, showed,
    started, told, top, touch, window,
};
use crate::machine::{Cause, Rules, Step};

/// The effect of `step` and who asked for it.
fn asked_by(step: &Step) -> (Option<&Effect>, Option<Cause>) {
    (step.effect(), step.state().cause())
}

#[test]
fn cause_names_who_asked_for_the_touch() {
    let scroll = told(&ask_top(rules()), answer("scroll_down", Some(0.9), None));
    let scrolling = touch("scroll_down", None, 0.9);
    assert_eq!(
        asked_by(&scroll),
        (Some(&scrolling), Some(Cause::Judge)),
        "scroll"
    );
    let looked = told(&ask_top(rules()), answer("click", Some(0.9), Some("1")));
    let save = id("button", "保存", 0);
    assert_eq!(asked_by(&looked), (Some(&look(save.clone())), None), "look");
    let click = showed(looked.state(), &top());
    let clicking = touch("click", Some(save), 0.9);
    assert_eq!(
        asked_by(&click),
        (Some(&clicking), Some(Cause::Judge)),
        "click"
    );
    the_core_asks_on_its_own();
    no_touch_no_cause();
}

/// Leaving a paywall, returning to the allowed origin and reloading.
fn the_core_asks_on_its_own() {
    let leave = started(rules(), &pricing());
    let back = touch("back", None, 1.0);
    assert_eq!(
        asked_by(&leave),
        (Some(&back), Some(Cause::Paywall)),
        "paywall"
    );
    let outside = started(rules(), &away());
    let ret = touch("return", None, 1.0);
    assert_eq!(
        asked_by(&outside),
        (Some(&ret), Some(Cause::Outside)),
        "outside"
    );
    let frozen = idle(rules(), seen(&top()), Memory::default()).with_counters(counters(3, 0));
    let reload = next(frozen, secs(0));
    let reloading = touch("reload", None, 1.0);
    assert_eq!(
        asked_by(&reload),
        (Some(&reloading), Some(Cause::Frozen)),
        "frozen"
    );
}

/// Asking, sleeping and the end wait on no browser operation.
fn no_touch_no_cause() {
    assert_eq!(ask_top(rules()).cause(), None, "asking");
    let few = Rules {
        requests_per_hour: nz(1),
        ..rules()
    };
    let full = idle(few, seen(&top()), Memory::default()).with_window(window(1, 1, secs(0)));
    let sleeping = next(full, secs(0));
    let sleep = Effect::Outside(Outside::Sleep(Wake { at: secs(3600) }));
    assert_eq!(asked_by(&sleeping), (Some(&sleep), None), "sleeping");
    let stuck = told(&ask_top(rules()), answer("stuck", Some(0.9), None));
    assert_eq!(asked_by(&stuck), (None, None), "ended");
}
