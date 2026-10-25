//! The elements operated since the screen last changed: kept while the
//! screen stays the same, each once, and passed over.

use super::{clicked, clicked_on, help, next_look, save, save_first};
use crate::machine::testing::{
    answer, ask_top, id, inside, look, ranked, rules, showed, started, told, top, touch, with,
};

#[test]
fn unchanged_target_is_kept_until_the_screen_changes() {
    let unchanged = clicked(&ask_top(rules()), save_first(), &save(), &top());
    assert_eq!(unchanged.state().unmoved(), [save()], "kept");
    let news = with(top(), inside("link", "新着"));
    let changed = clicked(&ask_top(rules()), save_first(), &save(), &news);
    assert_eq!(changed.state().unmoved(), [], "the screen changed");
}

#[test]
fn an_element_operated_twice_on_an_unchanged_screen_is_kept_once() {
    let only = || ranked("click", Some(0.9), &[("1", 0.8)]);
    let once = clicked(&ask_top(rules()), only(), &save(), &top());
    let twice = clicked(once.state(), only(), &save(), &top());
    assert_eq!(twice.state().unmoved(), [save()], "kept once");
}

#[test]
fn a_back_that_changes_nothing_keeps_the_elements_left_unmoved() {
    let unchanged = clicked(&ask_top(rules()), save_first(), &save(), &top());
    let backing = told(unchanged.state(), answer("back", Some(0.9), None));
    assert_eq!(
        backing.effect(),
        Some(&touch("back", None, 0.9)),
        "goes back"
    );
    let still = showed(backing.state(), &top());
    assert_eq!(still.state().unmoved(), [save()], "kept");
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
