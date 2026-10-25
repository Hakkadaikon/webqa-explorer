//! Switching to the next candidate when the screen did not change after an
//! operation on an element.

use crate::effect::Effect;
use crate::judge::Answers;
use crate::machine::testing::{
    answer, ask_top, asking, counters, id, inside, look, ranked, rules, secs, seen, showed,
    started, told, top, touch, with,
};
use crate::machine::{Facts, State, Step, step};
use crate::screen::{ElementId, Screen};

fn save() -> ElementId {
    id("button", "保存", 0)
}

fn help() -> ElementId {
    id("link", "ヘルプ", 0)
}

/// The judge ranks "保存" over "ヘルプ".
fn save_first() -> Answers {
    ranked("click", Some(0.9), &[("1", 0.8), ("2", 0.6)])
}

/// Clicks the element the answers point at, `element`, and observes `then`.
fn clicked(state: &State, answers: Answers, element: &ElementId, then: &Screen) -> Step {
    clicked_on(&top(), state, answers, element, then)
}

/// Clicks `element` on `page` as the answers point at it, and observes `then`.
fn clicked_on(
    page: &Screen,
    state: &State,
    answers: Answers,
    element: &ElementId,
    then: &Screen,
) -> Step {
    let looked = told(state, answers);
    assert_eq!(looked.effect(), Some(&look(element.clone())), "looks it up");
    let touched = showed(looked.state(), page);
    let expected = touch("click", Some(element.clone()), 0.9);
    assert_eq!(touched.effect(), Some(&expected), "clicks it");
    showed(touched.state(), then)
}

/// The effect of the next `answers` after `state`.
fn next_look(state: &State, answers: Answers) -> Option<Effect> {
    told(state, answers).effect().cloned()
}

#[test]
fn unchanged_target_switches_to_next() {
    let unchanged = clicked(&ask_top(rules()), save_first(), &save(), &top());
    assert_eq!(unchanged.effect(), Some(&asking(&rules(), &top())), "asks");
    let state = unchanged.state();
    clicked(state, save_first(), &help(), &top());
    let only = ranked("click", Some(0.9), &[("1", 0.8)]);
    assert_eq!(next_look(state, only), Some(look(save())), "only one");
    let unsure = told(state, answer("back", Some(0.5), None));
    let after_stop = next_look(unsure.state(), save_first());
    assert_eq!(after_stop, Some(look(help())), "a stop keeps it");
    nothing_to_pass_over();
}

/// After a change, or after an operation on no element, the judge's first
/// pick stands.
fn nothing_to_pass_over() {
    let news = with(top(), inside("link", "新着"));
    let changed = clicked(&ask_top(rules()), save_first(), &save(), &news);
    let again = next_look(changed.state(), save_first());
    assert_eq!(again, Some(look(save())), "the screen changed");
    let scrolled = told(&ask_top(rules()), answer("scroll_down", Some(0.9), None));
    let still = showed(scrolled.state(), &top());
    let after_scroll = next_look(still.state(), save_first());
    assert_eq!(after_scroll, Some(look(save())), "no element operated");
}

/// Clicks "保存" and observes the same top page, one step short of the
/// freeze threshold before the click.
fn unchanged_before_reload() -> Step {
    let looked = told(&ask_top(rules()), save_first());
    let touched = showed(looked.state(), &top());
    let frozen = touched.state().clone().with_counters(counters(2, 0));
    showed(&frozen, &top())
}

#[test]
fn reload_forgets_the_unchanged_target() {
    let reloading = unchanged_before_reload();
    let reload = touch("reload", None, 1.0);
    assert_eq!(reloading.effect(), Some(&reload), "reloads");
    let reloaded = showed(reloading.state(), &top());
    assert_eq!(reloaded.effect(), Some(&asking(&rules(), &top())), "asks");
    let after = next_look(reloaded.state(), save_first());
    assert_eq!(after, Some(look(save())), "the judge's first pick stands");
}

#[test]
fn aborted_look_up_forgets_the_unchanged_target() {
    let unchanged = clicked(&ask_top(rules()), save_first(), &save(), &top());
    let looked = told(unchanged.state(), save_first());
    assert_eq!(
        looked.effect(),
        Some(&look(help())),
        "looks the next one up"
    );
    let mut swapped = seen(&top());
    swapped.material.nodes[1].node = "x9".to_owned();
    let aborted = step(
        looked.state().clone(),
        Facts::Page(Box::new(swapped)),
        secs(0),
    );
    let after = next_look(aborted.state(), save_first());
    assert_eq!(after, Some(look(save())), "the judge's first pick stands");
}

#[test]
fn unchanged_target_is_kept_until_the_screen_changes() {
    let unchanged = clicked(&ask_top(rules()), save_first(), &save(), &top());
    assert_eq!(unchanged.state().unmoved(), [save()], "kept");
    let news = with(top(), inside("link", "新着"));
    let changed = clicked(&ask_top(rules()), save_first(), &save(), &news);
    assert_eq!(changed.state().unmoved(), [], "the screen changed");
}

#[test]
fn every_element_left_unmoved_is_passed_over_until_the_screen_changes() {
    let page = with(top(), inside("link", "新着"));
    let news = id("link", "新着", 0);
    let three = || ranked("click", Some(0.9), &[("1", 0.8), ("2", 0.6), ("3", 0.4)]);
    let asking = started(rules(), &page).state().clone();
    let first = clicked_on(&page, &asking, three(), &save(), &page);
    let second = clicked_on(&page, first.state(), three(), &help(), &page);
    assert_eq!(second.state().unmoved(), [save(), help()], "both kept");
    let third = next_look(second.state(), three());
    assert_eq!(third, Some(look(news)), "passes over both");
}
