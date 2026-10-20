//! The runs behind the scenarios of the re-run and of the ledger screen:
//! trials that walk through named screens, and hesitate or find a thing
//! hard to see at some of them. The numbers of the feature (`I-0001`) each
//! stand for the issue of one screen.

use std::collections::VecDeque;
use std::iter;

use webqa_core::interview::ask::{EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS};
use webqa_core::interview::issue::{Kind, Number};
use webqa_core::interview::natural::{NATURAL, NO, PERSONA_TRAIT};
use webqa_core::judge::{Answers, JudgeError};

use crate::judge::Said;
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// What the persona guesses the start screen is for, unless a scenario says.
pub const GUESS: &str = "商品を選んで注文する";

/// A screen of the walks, with the one button on it. The names of the
/// buttons are none of the words the guard forbids.
#[derive(Debug, Clone, Copy)]
pub struct Spot {
    /// The number of the feature whose issue is on this screen.
    pub issue: &'static str,
    /// What the feature calls the screen.
    pub screen: &'static str,
    /// The path of the screen.
    pub path: &'static str,
    /// The name of its button.
    pub button: &'static str,
}

/// The screens, by the number of the feature.
const SPOTS: [Spot; 4] = [
    Spot {
        issue: "I-0001",
        screen: "注文確認",
        path: "/order/confirm",
        button: "内容を見る",
    },
    Spot {
        issue: "I-0002",
        screen: "カート",
        path: "/cart",
        button: "カートを開く",
    },
    Spot {
        issue: "I-0003",
        screen: "お届け先",
        path: "/address",
        button: "住所を入れる",
    },
    Spot {
        issue: "I-0005",
        screen: "ギフト",
        path: "/gift",
        button: "包装を選ぶ",
    },
];

/// The screen the number `issue` of the feature stands for.
pub fn spot(issue: impl AsRef<str>) -> Spot {
    let found_in = |s: &&Spot| s.issue == issue.as_ref();
    found(SPOTS.iter().find(found_in).copied(), "a known example")
}

/// The screen the number `issue` stands for, which the feature calls
/// `screen`.
pub fn at(issue: impl AsRef<str>, screen: impl AsRef<str>) -> Spot {
    let spot = spot(issue.as_ref());
    assert_eq!(
        spot.screen,
        screen.as_ref(),
        "the screen of {}",
        issue.as_ref()
    );
    spot
}

/// What the persona does on a screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// Nothing worth an issue.
    Quiet,
    /// Hesitates.
    Hesitant,
    /// Finds the button hard to see.
    Faint,
}

/// The screens one trial goes on to after the start screen, in order.
pub type Trial = Vec<(Spot, Mark)>;

fn page(spot: Spot) -> PageSpec {
    PageSpec::at(spot.path).with(PageSpec::button(spot.button))
}

/// The number of the issue of `kind` on `spot`.
pub fn number(spot: Spot, kind: Kind) -> Number {
    let target = Some(("button", spot.button, 0));
    number_of(spot.path, &[("button", spot.button)], target, kind)
}

/// The kind of issue a mark makes, if any.
const fn kind_of(mark: Mark) -> Option<Kind> {
    match mark {
        Mark::Quiet => None,
        Mark::Hesitant => Some(Kind::Hesitant),
        Mark::Faint => Some(Kind::Visibility),
    }
}

/// The pages a trial shows after the first one. An operation is made on a
/// page the core has looked at twice, before and after it asks the judge,
/// to see that the element is still there: so each page comes twice, and
/// the page the last operation leads to stays.
fn pages(first: &PageSpec, trial: &Trial) -> Vec<PageSpec> {
    let twice = trial.iter().flat_map(|&(spot, _)| [page(spot), page(spot)]);
    let walk: Vec<PageSpec> = iter::once(first.clone()).chain(twice).collect();
    if trial.is_empty() { Vec::new() } else { walk }
}

/// The answers of one step: `op` on the first element.
fn said(op: &str, mark: Mark, guess: &str) -> Said {
    let hesitation = if mark == Mark::Hesitant { 0.9 } else { 0.0 };
    let faint = if mark == Mark::Faint {
        "1: 文字が薄い"
    } else {
        ""
    };
    Said::default()
        .choose("operation", op, 0.9)
        .choose("target", "1", 0.9)
        .choose("error_shown", "true", 0.0)
        .choose("goal_reached", "true", 0.0)
        .choose(HESITATION, "true", hesitation)
        .write(EXPECTATION, "次へ進む")
        .write(PURPOSE_GUESS, guess)
        .write(HARD_TO_SEE, faint)
}

/// The answers of the steps of `trial`: a click at each screen, then, for
/// a trial that went anywhere, a last look at the last screen where the
/// persona is done. A persona who goes nowhere is done at the start. A
/// hesitation needs a click to be about an element.
fn steps(trial: &Trial, guess: &str) -> Vec<Said> {
    let marks = iter::once(Mark::Quiet).chain(trial.iter().map(|&(_, mark)| mark));
    let walking = !trial.is_empty();
    let op = if walking { "click" } else { "done" };
    let end = walking.then(|| said("done", Mark::Quiet, guess));
    marks.map(|mark| said(op, mark, guess)).chain(end).collect()
}

/// The issues the trials make, each once: its screen and its kind.
fn issues_of(trials: &[Trial]) -> Vec<(&'static str, Kind)> {
    let marked = trials.iter().flatten();
    let made = marked.filter_map(|(spot, mark)| kind_of(*mark).map(|kind| (spot.path, kind)));
    let mut issues = Vec::new();
    for it in made {
        if !issues.contains(&it) {
            issues.push(it);
        }
    }
    issues
}

/// The replies of the judge: the steps of the trials, then the question
/// about the naturalness of each issue.
fn replies(trials: &[Trial], guess: &str) -> VecDeque<Result<Answers, JudgeError>> {
    let natural = Said::default()
        .choose(NATURAL, NO, 1.0)
        .choose(PERSONA_TRAIT, "ITの慣れ", 1.0);
    let asked = trials.iter().flat_map(|trial| steps(trial, guess));
    let after = iter::repeat_n(natural, issues_of(trials).len());
    asked.chain(after).map(|said| Ok(said.build())).collect()
}

/// Sets the scene to run `trials`, with the guess `guess` of the start
/// screen's purpose. The questions about the issues come after the
/// trials, one for each issue.
pub fn course(scene: &mut Scene, trials: &[Trial], guess: &str) {
    scene.trials = u32::try_from(trials.len()).unwrap_or(u32::MAX);
    scene.start.elements = vec![PageSpec::button("進む")];
    let first = scene.start.clone();
    scene.after = trials.iter().map(|trial| pages(&first, trial)).collect();
    let faint = |(_, mark): &(Spot, Mark)| *mark == Mark::Faint;
    scene.images = trials.iter().flatten().any(faint);
    scene.replies = replies(trials, guess);
}

/// `total` trials, of which `reached` go on to `at`, and `occurred` of
/// those hesitate there.
pub fn trials_at(total: u32, reached: u32, occurred: u32, at: Spot) -> Vec<Trial> {
    let trial = |n: u32| {
        let mark = if n < occurred {
            Mark::Hesitant
        } else {
            Mark::Quiet
        };
        if n < reached {
            vec![(at, mark)]
        } else {
            Vec::new()
        }
    };
    (0..total).map(trial).collect()
}

/// Keeps `value` under `key` for the steps after this one.
pub fn stash(world: &mut InterviewWorld, key: &'static str, value: &str) {
    world.kept.insert(key, value.to_owned());
}

/// The value `stash` kept under `key`. A step gets the world as `&mut`
/// whether it changes it or not, and a helper that only read it would make
/// every step that only calls the helper pass `&mut` for nothing, so this
/// reads through the mutable side too.
pub fn stashed(world: &mut InterviewWorld, key: &str) -> String {
    found(
        world.kept.get_mut(key).map(|v| v.clone()),
        "the value was stashed",
    )
}

/// The screen the feature calls `name`.
pub fn screen(name: impl AsRef<str>) -> Spot {
    let found_in = |s: &&Spot| s.screen == name.as_ref();
    found(SPOTS.iter().find(found_in).copied(), "a known screen")
}
