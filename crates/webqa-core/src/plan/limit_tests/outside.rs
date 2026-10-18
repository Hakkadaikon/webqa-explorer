//! Leaving the allowed origins and coming back.

use crate::explore::Memory;
use crate::issue::{Issue, IssueKind};
use crate::machine::testing::{
    answer, ask_top, asking, away, id, idle, look, operated, picked, replies, rules, secs, seen,
    shows, top, touch,
};
use crate::machine::{Note, Phase, State, Step, step};
use crate::plan::Pick;
use crate::redact::Redactor;

/// Clicks "ヘルプ" and lands outside the allowed origins.
fn clicked_away() -> Step {
    let looked = step(
        ask_top(rules()),
        replies(Ok(answer("click", Some(0.9), Some("2")))),
        secs(0),
    );
    let help = id("link", "ヘルプ", 0);
    assert_eq!(looked.effect(), Some(&look(help.clone())), "looks it up");
    let touched = step(looked.state().clone(), shows(&top()), secs(0));
    let click = touch("click", Some(help), 0.9);
    assert_eq!(touched.effect(), Some(&click), "clicks it");
    step(touched.state().clone(), shows(&away()), secs(0))
}

/// Landed outside: going back without asking, with the issue recorded and
/// the element explored.
fn returning(state: &State) {
    assert_eq!(state.phase(), Phase::Returning, "returning");
    assert!(state.outside(), "outside");
    assert_eq!(state.budget().used(), 1, "not asked outside");
    let issue = Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(Redactor::default().text("https://other.example.net/")),
        label: None,
        p: None,
    };
    assert_eq!(state.notes().last(), Some(&Note::Issue(issue)), "recorded");
    let help = id("link", "ヘルプ", 0);
    assert!(operated(state, &top(), &help), "explored");
}

#[test]
fn outside_domain_returns_without_asking() {
    let returned = clicked_away();
    let back_home = touch("return", None, 1.0);
    assert_eq!(returned.effect(), Some(&back_home), "goes back");
    returning(returned.state());
    let back = step(returned.state().clone(), shows(&top()), secs(0));
    assert_eq!(back.effect(), Some(&asking(&rules(), &top())), "asks again");
    assert!(!back.state().outside(), "inside again");
    assert_eq!(back.state().window().used(secs(0)), 2, "two accesses");
    let outside = idle(rules(), seen(&away()), Memory::default());
    let pick = picked(&outside, secs(0));
    assert_eq!(pick, Pick::Return, "returns from outside");
}
