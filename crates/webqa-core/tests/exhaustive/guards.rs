//! The guardrails: forbidden elements, unsure going back, and when to ask.

use webqa_core::machine::Phase;

use super::candidates::pages::{BUY, CART_MARKS, FORBIDDEN, TOP_MARKS};
use super::candidates::{PRICING, STOP, TWO};
use super::view::{Property, Transition, all, any, asks, implies, looks, sleeps};

/// Never operating a forbidden element, and acting only after the checks.
pub static ACTING: [Property; 6] = [
    Property {
        name: "an operation never targets the forbidden button",
        holds: |t| t.target().is_none_or(|id| id.name != BUY),
    },
    Property {
        name: "the forbidden buttons are never marked operated",
        holds: forbidden_untouched,
    },
    Property {
        name: "enter is never pressed on a page with a forbidden button",
        holds: |t| implies(t.touches("enter"), !FORBIDDEN.map(Some).contains(&t.last)),
    },
    Property {
        name: "an operation on an element comes only from confirming",
        holds: |t| implies(t.target().is_some(), t.before.phase() == Phase::Confirming),
    },
    Property {
        name: "an element is looked up only after the answers",
        holds: |t| implies(looks(t), t.before.phase() == Phase::Awaiting),
    },
    Property {
        name: "going back on the answers is sure",
        holds: sure_back,
    },
];

/// Asking only when nothing comes first, and returning when outside.
pub static ASKING: [Property; 2] = [
    Property {
        name: "the judge is asked only when free to explore",
        holds: |t| implies(asks(t), free(t)),
    },
    Property {
        name: "outside the allowed origin the run returns, reloads or sleeps",
        holds: leaves_outside,
    },
];

fn forbidden_untouched(t: &Transition) -> bool {
    let marked = |page: usize, element: usize| {
        t.after_marks.get(page).and_then(|marks| marks.get(element)) == Some(&true)
    };
    !(marked(TOP_MARKS, 2) || marked(CART_MARKS, 0))
}

fn sure_back(t: &Transition) -> bool {
    let answered = t.before.phase() == Phase::Awaiting;
    let sure = t.touch().is_some_and(|touch| touch.confidence >= 0.7);
    implies(all(&[answered, t.touches("back")]), sure)
}

/// Not frozen, outside, on a paywall, stuck, out of time or of the window.
fn free(t: &Transition) -> bool {
    let after = t.after();
    all(&[
        !after.counters().frozen(TWO),
        !after.outside(),
        t.last != Some(PRICING),
        !after.counters().is_stuck(TWO),
        t.at < STOP,
        !after.window().full(t.at),
    ])
}

fn leaves_outside(t: &Transition) -> bool {
    let away = all(&[t.after().outside(), t.step.effect().is_some()]);
    implies(
        away,
        any(&[t.touches("return"), t.touches("reload"), sleeps(t)]),
    )
}
