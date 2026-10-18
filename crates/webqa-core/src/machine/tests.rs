//! The end of a run, interruption, facts that do not fit the phase, and the
//! values the state shows outside.

use serde_json::{Value, json};

use crate::config::raw::Op;
use crate::effect::Effect;
use crate::explore::Memory;
use crate::issue::{Issue, IssueKind};
use crate::machine::ops::offered;
use crate::machine::testing::{
    APP, answer, ask_top, asking, budget, counts, idle, next, nz, page, rules, secs, seen, showed,
    shows, started, told, top, window,
};
use crate::machine::{
    ExitReason, Facts, Heard, Note, Phase, Rules, State, Step, StuckKind, abort, step,
};
use crate::redact::Redactor;
use crate::screen::Screen;

/// What must not change once the run is over.
fn kept(state: &State) -> (u32, u32, (u32, u32), usize) {
    (
        state.budget().used(),
        state.window().used(secs(0)),
        counts(state),
        state.notes().len(),
    )
}

#[test]
fn finished_state_stays_finished() {
    let spent = idle(rules(), seen(&top()), Memory::default()).with_budget(budget(20, 20));
    let finished = next(spent, secs(0)).state().clone();
    for facts in [Facts::Outside(Heard::Woke), shows(&top())] {
        let again = step(finished.clone(), facts, secs(0));
        assert_eq!(again.reason(), Some(ExitReason::BudgetExceeded), "same end");
        assert_eq!(again.effect(), None, "nothing to do");
        assert_eq!(kept(again.state()), kept(&finished), "nothing changes");
    }
    assert_eq!(abort(finished).reason, ExitReason::BudgetExceeded, "kept");
    let aborted = abort(ask_top(rules()));
    assert_eq!(aborted.reason, ExitReason::UserAborted, "interrupted");
    let ended = aborted.state.ended();
    assert_eq!(ended, Some(ExitReason::UserAborted), "ends there");
}

/// `state` takes `facts` that do not fit, and goes back to waiting.
fn mismatched(state: &State, facts: Facts, expected: &Effect) {
    let notes = state.notes().len();
    let used = state.budget().used();
    let step: Step = step(state.clone(), facts, secs(3600));
    assert_eq!(step.effect(), Some(expected), "back to waiting");
    assert_eq!(step.state().notes().len(), notes, "nothing recorded");
    assert_eq!(counts(step.state()), (0, 0), "counts kept");
    let asked = step.state().budget().used();
    assert_eq!(asked, used + 1, "only the next question");
}

#[test]
fn mismatched_facts_return_to_idle() {
    let asks = asking(&rules(), &top());
    let awaiting = ask_top(rules());
    mismatched(&awaiting, shows(&top()), &asks);
    let looking = told(&awaiting, answer("click", Some(0.9), Some("1")));
    mismatched(looking.state(), Facts::Outside(Heard::Woke), &asks);
    let few = Rules {
        requests_per_hour: nz(1),
        ..rules()
    };
    let full =
        idle(few.clone(), seen(&top()), Memory::default()).with_window(window(1, 1, secs(0)));
    let sleeping = next(full, secs(0)).state().clone();
    let asks = asking(&few, &top());
    mismatched(&sleeping, shows(&top()), &asks);
    mismatched(&sleeping, Facts::Outside(Heard::Woke), &asks);
}

/// Every exit reason with its JSON.
fn reasons() -> [(ExitReason, Value); 7] {
    [
        (ExitReason::GoalAchieved, json!({"reason": "goal_achieved"})),
        (ExitReason::ReportedDone, json!({"reason": "reported_done"})),
        (
            ExitReason::Stuck(StuckKind::Declared),
            json!({"reason": "stuck", "kind": "declared"}),
        ),
        (
            ExitReason::Stuck(StuckKind::NoProgress),
            json!({"reason": "stuck", "kind": "no_progress"}),
        ),
        (
            ExitReason::BudgetExceeded,
            json!({"reason": "budget_exceeded"}),
        ),
        (ExitReason::TimeExceeded, json!({"reason": "time_exceeded"})),
        (ExitReason::UserAborted, json!({"reason": "user_aborted"})),
    ]
}

#[test]
fn exit_reasons_serialize_with_reason_and_kind() {
    for (reason, value) in reasons() {
        assert_eq!(serde_json::to_value(reason).unwrap(), value, "{reason:?}");
        let back: ExitReason = serde_json::from_value(value).unwrap();
        assert_eq!(back, reason, "{reason:?} reads back");
        let kind = reason.stuck_kind();
        assert_eq!(
            reason.is_stuck(),
            kind.is_some(),
            "{reason:?} is stuck or not"
        );
    }
    let declared = ExitReason::Stuck(StuckKind::Declared);
    assert_eq!(declared.stuck_kind(), Some(StuckKind::Declared), "the kind");
    assert_eq!(ExitReason::TimeExceeded.stuck_kind(), None, "not stuck");
}

#[test]
fn offered_follows_table_order() {
    let rules = Rules {
        ops: vec![Op::Stuck, Op::Click, Op::Back],
        ..rules()
    };
    let names: Vec<&str> = offered(&rules).iter().map(|row| row.name).collect();
    assert_eq!(
        names,
        ["click", "back", "stuck"],
        "in the order of the table"
    );
}

/// Started inside on `screen`, waiting for its first element to be looked up.
fn confirming_on(screen: &Screen) -> State {
    let asked = started(rules(), screen);
    let looking = told(asked.state(), answer("click", Some(0.9), Some("1")));
    let state = looking.state().clone();
    assert_eq!(state.phase(), Phase::Confirming, "holds the proposal");
    state
}

#[test]
fn debug_hides_page_text() {
    let screen = page("https://other.example.net/", &[("button", "山田太郎")]);
    let state = started(rules(), &screen).state().clone();
    let state = showed(&state, &screen).state().clone();
    let issue = Note::Issue(Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(Redactor::default().text("https://other.example.net/")),
        label: None,
        p: None,
    });
    let confirming = confirming_on(&page(&format!("{APP}/"), &[("button", "山田太郎")]));
    let all = [state, confirming].map(|s| format!("{s:?}"));
    for shown in all.into_iter().chain([format!("{issue:?}")]) {
        assert!(!shown.contains("山田太郎"), "no name in {shown}");
        assert!(!shown.contains("other.example.net"), "no URL in {shown}");
    }
}
