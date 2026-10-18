//! The rows from waiting for the judge on: proposals that pass, finish or stop.

use std::collections::BTreeSet;

use crate::explore::{ElementOp, Memory};
use crate::machine::testing::{
    answer, ask_top, asking, counters, id, idle, inside, look, next, nz, operated, rules, secs,
    seen, showed, started, told, top, touch, with,
};
use crate::machine::{ExitReason, Note, Phase, Rejection, State, StuckKind};

/// `op` on "保存" is looked up, touched, and "保存" is then operated.
fn touches_save(op: &str) {
    let looked = told(&ask_top(rules()), answer(op, Some(0.9), Some("1")));
    let save = id("button", "保存", 0);
    assert_eq!(
        looked.effect(),
        Some(&look(save.clone())),
        "{op} looks it up"
    );
    assert_eq!(looked.state().phase(), Phase::Confirming, "{op} confirms");
    let touched = showed(looked.state(), &top());
    let expected = touch(op, Some(save), 0.9);
    assert_eq!(touched.effect(), Some(&expected), "{op} touches it");
    assert_eq!(touched.state().phase(), Phase::Acting, "{op} acts");
    after_touch(touched.state(), op, true);
}

/// `op` is touched at once, and "保存" stays untouched.
fn touches_page(op: &str) {
    let touched = told(&ask_top(rules()), answer(op, Some(0.9), Some("1")));
    assert_eq!(
        touched.effect(),
        Some(&touch(op, None, 0.9)),
        "{op} at once"
    );
    assert_eq!(touched.state().phase(), Phase::Acting, "{op} acts");
    after_touch(touched.state(), op, false);
}

/// Observing the start page after the touch asks again.
fn after_touch(state: &State, op: &str, save: bool) {
    let back = showed(state, &top());
    assert_eq!(
        back.effect(),
        Some(&asking(&rules(), &top())),
        "{op} asks again"
    );
    let save_operated = operated(back.state(), &top(), &id("button", "保存", 0));
    assert_eq!(save_operated, save, "{op} explores 保存 or not");
    assert_eq!(
        back.state().window().used(secs(0)),
        1,
        "{op} used one access"
    );
}

#[test]
fn passed_proposal_becomes_touch() {
    for op in ["click", "input"] {
        touches_save(op);
    }
    for op in ["scroll_down", "scroll_up", "back", "wait"] {
        touches_page(op);
    }
}

#[test]
fn self_reported_done_is_unverified() {
    let step = told(&ask_top(rules()), answer("done", Some(0.9), None));
    assert_eq!(step.reason(), Some(ExitReason::ReportedDone), "unverified");
    assert_eq!(step.state().window().used(secs(0)), 0, "no access");
}

#[test]
fn stuck_answer_finishes_stuck() {
    let step = told(&ask_top(rules()), answer("stuck", Some(0.9), None));
    let reason = step.reason();
    assert_eq!(
        reason,
        Some(ExitReason::Stuck(StuckKind::Declared)),
        "the judge gave up"
    );
    assert_eq!(reason.map(ExitReason::is_stuck), Some(true), "stuck");
}

/// A memory with "保存" and "ヘルプ" of the start page operated.
fn explored_top() -> Memory {
    let mut memory = Memory::default();
    let at = memory.visit(&seen(&top()).screen, &BTreeSet::new());
    for (role, name) in [("button", "保存"), ("link", "ヘルプ")] {
        memory.record(at, &id(role, name, 0), ElementOp::Click, at);
    }
    memory
}

#[test]
fn reload_after_freeze_keeps_memory() {
    let state = idle(rules(), seen(&top()), explored_top()).with_counters(counters(3, 0));
    let reloaded = next(state, secs(0));
    assert_eq!(
        reloaded.effect(),
        Some(&touch("reload", None, 1.0)),
        "reload"
    );
    let back = showed(reloaded.state(), &top());
    assert_eq!(back.effect(), Some(&asking(&rules(), &top())), "asks");
    let state = back.state();
    for (role, name) in [("button", "保存"), ("link", "ヘルプ")] {
        let kept = operated(state, &top(), &id(role, name, 0));
        assert!(kept, "{name} stays operated");
    }
    let counts = (state.counters().freeze(), state.counters().stuck());
    assert_eq!(counts, (0, 0), "counts after the reload");
    assert_eq!(state.budget().used(), 1, "asked once after the reload");
}

#[test]
fn rejected_proposal_uses_no_access() {
    let mut rules = rules();
    rules.requests_per_hour = nz(2);
    let screen = with(top(), inside("button", "購入する"));
    let asked = started(rules.clone(), &screen);
    let step = told(asked.state(), answer("click", Some(0.9), Some("3")));
    assert_eq!(step.effect(), Some(&asking(&rules, &screen)), "asks again");
    let state = step.state();
    assert_eq!(state.window().used(secs(0)), 0, "no access");
    let rejected = vec![Note::Rejected(Rejection::Forbidden)];
    assert_eq!(state.notes(), rejected.as_slice(), "recorded");
    let buy = id("button", "購入する", 0);
    assert!(!operated(state, &screen, &buy), "not operated");
}
