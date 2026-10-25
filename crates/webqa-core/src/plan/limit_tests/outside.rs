//! Leaving the allowed origins and coming back.

use crate::explore::Memory;
use crate::issue::{Issue, IssueKind};
use crate::machine::testing::{
    VIEWPORT, answer, ask_top, asking, away, counters, counts, id, idle, look, next, operated,
    page, picked, replies, rules, secs, seen, showed, shows, started, told, top, touch,
};
use crate::machine::{
    ExitReason, Facts, Material, Note, Phase, Rejection, Seen, State, Step, StuckKind, step,
};
use crate::plan::Pick;
use crate::redact::Redactor;
use crate::screen::Screen;

/// The issue of landing outside, with the redacted `url` as evidence.
fn left(url: &str) -> Note {
    Note::Issue(Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(Redactor::default().text(url)),
        label: None,
        p: None,
    })
}

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
    let issue = left("https://other.example.net/");
    assert_eq!(state.notes(), [issue], "recorded once");
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

#[test]
fn the_blank_page_returns_without_an_issue() {
    let returned = clicked_to(shows(&page("about:blank", &[])));
    let back_home = touch("return", None, 1.0);
    assert_eq!(returned.effect(), Some(&back_home), "goes back");
    assert!(returned.state().outside(), "outside");
    assert_eq!(returned.state().notes(), [], "the page sent nowhere");
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
    let issue = left("https://app.example.test/");
    assert_eq!(state.notes().last(), Some(&issue), "the redacted URL");
    let inside = idle(rules(), masked(&away(), &top()), Memory::default());
    assert!(!inside.outside(), "inside by the material");
}

#[test]
fn redirected_outside_ends_stuck() {
    let mut step = started(rules(), &away());
    for stuck in 1..3 {
        step = showed(step.state(), &away());
        let back_home = touch("return", None, 1.0);
        assert_eq!(step.effect(), Some(&back_home), "returns again");
        assert_eq!(counts(step.state()), (0, stuck), "no progress");
    }
    let last = showed(step.state(), &away());
    let stuck = Some(ExitReason::Stuck(StuckKind::NoProgress));
    assert_eq!(last.reason(), stuck, "stuck at the threshold");
    assert_eq!(last.state().budget().used(), 0, "never asked");
    let issues = vec![left("https://other.example.net/"); 3];
    assert_eq!(last.state().notes(), issues, "one issue per arrival");
}

/// After `step`, landing on an empty page outside counts as no progress.
fn lands_outside(step: &Step, expected: (u32, u32)) -> State {
    let outside = page("https://other.example.net/", &[]);
    let landed = showed(step.state(), &outside);
    let back_home = touch("return", None, 1.0);
    assert_eq!(landed.effect(), Some(&back_home), "returns");
    assert_eq!(
        counts(landed.state()),
        expected,
        "one more without progress"
    );
    let issue = left("https://other.example.net/");
    assert_eq!(landed.state().notes().last(), Some(&issue), "recorded");
    landed.state().clone()
}

#[test]
fn landing_outside_after_abort_or_reload_counts() {
    let frozen = idle(rules(), seen(&top()), Memory::default());
    let reloading = next(frozen.with_counters(counters(3, 0)), secs(0));
    assert_eq!(
        reloading.effect(),
        Some(&touch("reload", None, 1.0)),
        "reloads"
    );
    lands_outside(&reloading, (0, 1));
    let waiting = idle(rules(), seen(&top()), Memory::default());
    let asked = next(waiting.with_counters(counters(2, 0)), secs(0));
    let looking = told(asked.state(), answer("click", Some(0.9), Some("1")));
    let aborted = lands_outside(&looking, (2, 1));
    let missing = Note::Rejected(Rejection::Missing);
    let issue = left("https://other.example.net/");
    assert_eq!(aborted.notes(), [missing, issue], "aborted, then recorded");
}
