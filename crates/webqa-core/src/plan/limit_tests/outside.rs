//! Leaving the allowed origins and coming back.

use crate::explore::Memory;
use crate::issue::{Issue, IssueKind};
use crate::machine::testing::{
    VIEWPORT, answer, ask_top, asking, away, id, idle, look, operated, picked, replies, rules,
    secs, seen, shows, top, touch,
};
use crate::machine::{Facts, Material, Note, Phase, Seen, State, Step, step};
use crate::plan::Pick;
use crate::redact::Redactor;
use crate::screen::Screen;

/// Clicks "ヘルプ" and lands outside the allowed origins.
fn clicked_away() -> Step {
    clicked_to(shows(&away()))
}

/// Clicks "ヘルプ" and lands on the page in `facts`.
fn clicked_to(facts: Facts) -> Step {
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
    step(touched.state().clone(), facts, secs(0))
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

/// The redacted page `shown` with the material collected on `collected`.
fn masked(shown: &Screen, collected: &Screen) -> Seen {
    Seen {
        material: Material::plain(collected, VIEWPORT),
        ..seen(shown)
    }
}

#[test]
fn outside_is_judged_before_redaction() {
    let landed = clicked_to(Facts::Page(Box::new(masked(&top(), &away()))));
    let state = landed.state();
    assert!(state.outside(), "outside by the material");
    let back_home = touch("return", None, 1.0);
    assert_eq!(landed.effect(), Some(&back_home), "goes back");
    let issue = Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(Redactor::default().text("https://app.example.test/")),
        label: None,
        p: None,
    };
    let recorded = state.notes().last();
    assert_eq!(recorded, Some(&Note::Issue(issue)), "the redacted URL");
    let inside = idle(rules(), masked(&away(), &top()), Memory::default());
    assert!(!inside.outside(), "inside by the material");
}
