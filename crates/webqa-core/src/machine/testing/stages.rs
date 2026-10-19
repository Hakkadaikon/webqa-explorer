//! Stages, explore mode and memories for the tests of the state machine.

use crate::config::raw::{DoneWhen, Element as Wanted, Stage};
use crate::event::Mode;
use crate::explore::{ElementOp, Memory, ScreenKey};
use crate::machine::material::forbidden_indices;
use crate::machine::{Rules, Seen};
use crate::screen::{ElementId, Screen};

use super::rules;
use super::screens::seen;

/// The background rules in explore mode.
pub fn explore() -> Rules {
    Rules {
        mode: Mode::Explore,
        ..rules()
    }
}

/// The background rules with the stages `stages`.
pub fn staged(stages: Vec<Stage>) -> Rules {
    Rules { stages, ..rules() }
}

/// The stage `name` with the goal `goal`, done when `done_when` holds.
pub fn stage(name: &str, goal: &str, done_when: DoneWhen) -> Stage {
    Stage {
        name: name.to_owned(),
        goal: goal.to_owned(),
        done_when,
    }
}

fn wanted(role: &str, name: &str) -> Wanted {
    Wanted {
        role: role.to_owned(),
        name: name.to_owned(),
    }
}

/// Done at the path `path`.
pub fn done_url(path: &str) -> DoneWhen {
    DoneWhen {
        url: Some(path.to_owned()),
        element: None,
        text: None,
    }
}

/// Done when the page shows `text`.
pub fn done_text(text: &str) -> DoneWhen {
    DoneWhen {
        url: None,
        element: None,
        text: Some(text.to_owned()),
    }
}

/// Done when the page has the element `role` `name`.
pub fn done_element(role: &str, name: &str) -> DoneWhen {
    DoneWhen {
        url: None,
        element: Some(wanted(role, name)),
        text: None,
    }
}

/// Done at the path `path` with the element `role` `name`.
pub fn done_both(path: &str, role: &str, name: &str) -> DoneWhen {
    DoneWhen {
        element: Some(wanted(role, name)),
        ..done_url(path)
    }
}

/// Logging in at "/", then posting until "投稿しました" shows.
pub fn two() -> Vec<Stage> {
    vec![
        stage("ログイン", "ログインする", done_url("/")),
        stage("投稿", "投稿する", done_text("投稿しました")),
    ]
}

/// A memory of the `screens`, their forbidden elements marked by the
/// background rules, with the clicks `edges`: from the screen at the first
/// position, on the element, to the screen at the last.
pub fn memory_of(screens: &[&Screen], edges: &[(usize, ElementId, usize)]) -> Memory {
    let mut memory = Memory::default();
    let keys: Vec<ScreenKey> = screens
        .iter()
        .map(|screen| visited(&mut memory, &seen(screen)))
        .collect();
    for (from, element, to) in edges {
        memory.record(keys[*from], element, ElementOp::Click, keys[*to]);
    }
    memory
}

fn visited(memory: &mut Memory, seen: &Seen) -> ScreenKey {
    memory.visit(&seen.screen, &forbidden_indices(&rules(), seen))
}
