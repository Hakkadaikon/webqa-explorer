//! Planning the trials of a scene: what the judge answers, what the browser
//! shows, and how long each call of the judge takes.

use std::iter;
use std::time::Duration;

use webqa_core::config::purpose::{Aim, Written};
use webqa_core::config::raw::{DoneWhen, Element};
use webqa_core::interview::ask::{EXPECTATION, HESITATION, PURPOSE_GUESS};
use webqa_core::judge::{Answers, JudgeError};

use crate::judge::Said;
use crate::page::PageSpec;
use crate::scene::Scene;

/// The button the purpose waits for.
const GOAL: &str = "完了";
/// The name of the button the judge presses, unless the scene shows another.
const STEP: &str = "進む";
/// The empty replies after a trial: the questions about naturalness take
/// them.
const PADS: usize = 3;
/// The length of a window.
pub(super) const HOUR: Duration = Duration::from_secs(3600);
/// The length of a window, in milliseconds.
pub(super) const WINDOW_MS: u64 = 3_600_000;

/// `m` minutes.
pub(super) const fn minutes(m: u64) -> Duration {
    Duration::from_secs(m * 60)
}

/// How a planned trial ends.
#[derive(Clone, Copy)]
pub(super) enum End {
    /// The last page shows the button of the purpose.
    Goal,
    /// The judge gives up.
    GaveUp,
    /// The judge says it is done.
    Done,
    /// Nothing ends it: the replies run out, and the judge's wait ends the run.
    Open,
    /// Nothing ends it, but something else will cut it.
    Cut,
}

impl End {
    /// The operation of the reply that ends the trial, if one does.
    const fn reply(self) -> Option<&'static str> {
        match self {
            Self::GaveUp => Some("stuck"),
            Self::Done => Some("done"),
            Self::Goal | Self::Open | Self::Cut => None,
        }
    }

    /// How many empty replies follow the trial.
    const fn pads(self) -> usize {
        match self {
            Self::Open => 0,
            Self::Goal | Self::GaveUp | Self::Done | Self::Cut => PADS,
        }
    }
}

/// The answers of a step whose operation is `op`, on the first element,
/// with the hesitation `hesitation`.
pub(super) fn said(op: &str, hesitation: f64) -> Answers {
    Said::default()
        .choose("operation", op, 0.9)
        .choose("target", "1", 0.9)
        .choose("error_shown", "true", 0.0)
        .choose(
            "goal_reached",
            "true",
            f64::from(u8::from(op == "done")) * 0.9,
        )
        .choose(HESITATION, "true", hesitation)
        .write(EXPECTATION, "進む")
        .write(PURPOSE_GUESS, "買う")
        .build()
}

/// The name of the button the trials press.
pub(super) fn step_name(scene: &Scene) -> String {
    let first = scene.start.elements.first();
    first.map_or_else(|| STEP.to_owned(), |element| element.name.clone())
}

/// The written purpose: the button of the goal is on the page.
fn written() -> Aim {
    let element = Element {
        role: "button".to_owned(),
        name: GOAL.to_owned(),
    };
    let done_when = DoneWhen {
        url: None,
        element: Some(element),
        text: None,
    };
    let goal = "目的を果たす".to_owned();
    Aim::Written(Written { goal, done_when })
}

/// How many replies are queued, without the empty ones at the end.
fn queued(scene: &mut Scene) -> usize {
    let empty =
        |reply: &Result<Answers, _>| reply.as_ref().is_ok_and(|answers| answers.table.is_empty());
    while scene.replies.back().is_some_and(empty) {
        scene.replies.pop_back();
    }
    scene.replies.len()
}

/// The page after the `k`-th operation of a trial: a new screen, with the
/// button of the goal when it is the last of a trial that reaches it.
fn page(name: &str, trial: usize, k: usize, goal: bool) -> PageSpec {
    let page = PageSpec::at(&format!("/t{trial}/{k}")).with(PageSpec::button(name));
    let button = goal.then(|| PageSpec::button(GOAL));
    button.into_iter().fold(page, PageSpec::with)
}

/// What a trial's browser shows after its start page. It looks at the page
/// before each operation, and again after it; the last page stays.
fn browsed(start: &PageSpec, screens: &[PageSpec]) -> Vec<PageSpec> {
    let before = iter::once(start).chain(screens);
    let pairs = before
        .zip(screens)
        .map(|(was, now)| [was.clone(), now.clone()]);
    let shown: Vec<PageSpec> = pairs.flatten().collect();
    let last = shown.last().unwrap_or(start).clone();
    shown.into_iter().chain(iter::repeat_n(last, 2)).collect()
}

/// The replies of a trial: `clicks` operations, its end, then the empty
/// ones.
fn replies(clicks: usize, end: End) -> Vec<Result<Answers, JudgeError>> {
    let clicks = (0..clicks).map(|_| said("click", 0.0));
    let ending = end.reply().map(|op| said(op, 0.0));
    let pads = (0..end.pads()).map(|_| Answers::default());
    clicks.chain(ending).chain(pads).map(Ok).collect()
}

/// Plans the next trial: `clicks` operations, then its end. The number of
/// the first call of the judge for it.
pub(super) fn course(scene: &mut Scene, clicks: usize, end: End) -> usize {
    let name = step_name(scene);
    scene.start = PageSpec::at("/").with(PageSpec::button(&name));
    scene.purpose.aim = written();
    let goal = matches!(end, End::Goal);
    let screens: Vec<PageSpec> = (1..=clicks)
        .map(|k| page(&name, scene.after.len(), k, goal && k == clicks))
        .collect();
    scene.after.push(browsed(&scene.start, &screens));
    let first = queued(scene);
    scene.replies.extend(replies(clicks, end));
    first
}

/// Plans from the first trial again, as the first trial of the plan.
pub(super) fn first(scene: &mut Scene, clicks: usize, end: End) -> usize {
    scene.replies.clear();
    scene.after.clear();
    scene.thinks.clear();
    course(scene, clicks, end)
}

/// Plans the trials that are left, each ending at its first operation.
pub(super) fn fill(scene: &mut Scene) {
    let trials = usize::try_from(scene.trials).unwrap_or(usize::MAX);
    for _ in scene.after.len()..trials {
        course(scene, 1, End::Goal);
    }
}

/// Plans the second trial to press `clicks` times and say it is done, then
/// the trials that are left.
pub(super) fn second_and_rest(scene: &mut Scene, clicks: usize) {
    course(scene, clicks, End::Done);
    fill(scene);
}

/// Makes the `call`-th call of the judge take `time`.
pub(super) fn think(scene: &mut Scene, call: usize, time: Duration) {
    let longer = (scene.thinks.len()..=call).map(|_| Duration::ZERO);
    scene.thinks.extend(longer);
    if let Some(slot) = scene.thinks.get_mut(call) {
        *slot = time;
    }
}

/// Plans every trial to end with the judge answering that the goal is met.
pub(super) fn claim_goal(scene: &mut Scene) {
    first(scene, 0, End::Done);
    let trials = usize::try_from(scene.trials).unwrap_or(usize::MAX);
    for _ in 1..trials {
        course(scene, 0, End::Done);
    }
}
