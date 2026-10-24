//! A proposal the guardrails stop, counted by mode.

use crate::event::Mode;
use crate::judge::{QuestionId, Unusable};
use crate::machine::testing::{
    APP, answer, ask_top, away, counts, nz, page, rules, secs, seen, told,
};
use crate::machine::{Anomaly, ExitReason, Facts, Note, Rules, StuckKind, step};

/// The rules of an interview.
fn interview() -> Rules {
    Rules {
        mode: Mode::Interview,
        ..rules()
    }
}

/// Under `rules`, the counts after a doubtful "back", and after a click
/// whose element is gone when looked up again.
fn stops(rules: &Rules) -> [(u32, u32); 2] {
    let back = told(&ask_top(rules.clone()), answer("back", Some(0.5), None));
    let looked = told(
        &ask_top(rules.clone()),
        answer("click", Some(0.9), Some("1")),
    );
    let gone = Facts::Page(Box::new(seen(&page(
        &format!("{APP}/"),
        &[("link", "ヘルプ")],
    ))));
    let aborted = step(looked.state().clone(), gone, secs(0));
    [counts(back.state()), counts(aborted.state())]
}

#[test]
fn a_stopped_proposal_counts_as_no_progress_in_an_interview() {
    assert_eq!(stops(&interview()), [(0, 1), (0, 1)], "one more stuck step");
}

#[test]
fn a_stopped_proposal_leaves_the_counters_in_a_scenario() {
    assert_eq!(stops(&rules()), [(0, 0), (0, 0)], "the counts stay");
}

#[test]
fn stopped_backs_end_an_interview_stuck_before_the_budget() {
    let rules = Rules {
        stuck_threshold: nz(3),
        max_decisions: nz(30),
        ..interview()
    };
    let back = || answer("back", Some(0.5), None);
    let first = told(&ask_top(rules), back());
    let second = told(first.state(), back());
    let third = told(second.state(), back());
    let stuck = Some(ExitReason::Stuck(StuckKind::NoProgress));
    let reasons = [first.reason(), second.reason(), third.reason()];
    assert_eq!(reasons, [None, None, stuck], "stuck at the third");
    assert_eq!(
        third.state().budget().used(),
        3,
        "three of thirty decisions"
    );
}

#[test]
fn a_stop_that_lands_outside_counts_once_in_an_interview() {
    let looked = told(&ask_top(interview()), answer("click", Some(0.9), Some("1")));
    let outside = Facts::Page(Box::new(seen(&away())));
    let aborted = step(looked.state().clone(), outside, secs(0));
    assert!(aborted.state().outside(), "landed outside");
    assert_eq!(counts(aborted.state()), (0, 1), "one step without progress");
}

#[test]
fn an_unusable_answer_is_not_counted_as_stuck_in_an_interview() {
    let no_element = answer("click", Some(0.9), None);
    let unusable = told(&ask_top(interview()), no_element);
    let anomaly = Anomaly::Unusable(Unusable::Missing(QuestionId::target()));
    let note = Note::Anomaly(anomaly);
    assert_eq!(unusable.state().notes().last(), Some(&note), "an anomaly");
    assert_eq!(counts(unusable.state()), (0, 0), "not a stopped proposal");
    let doubtful = told(&ask_top(interview()), answer("back", Some(0.5), None));
    assert_eq!(counts(doubtful.state()), (0, 1), "a stopped one counts");
}
