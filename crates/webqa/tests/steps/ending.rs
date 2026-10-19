//! Steps of the Rule 終了の判定: the parts of a scene that the files of the
//! Rule use. No scenario of the Rule belongs to this file alone.

use std::time::Duration;

use webqa_core::event::Mode;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::remembered::remember;
use crate::scene::Scene;
use crate::world::QaWorld;

/// The scenarios that have their steps: none, the Rule's files below have them.
pub const READY: &[&str] = &[];

/// The text a screen shows when the goal is reached.
const WELCOME: &str = "ようこそ";

/// The time limit of the scenes that run out of time.
const LIMIT: Duration = Duration::from_secs(60);

fn page(save_below: bool, notice_below: bool) -> PageSpec {
    let save = PageSpec::button("保存");
    let notice = PageSpec::link("通知設定");
    PageSpec::at("/")
        .with(if save_below { save.below() } else { save })
        .with(PageSpec::link("ヘルプ"))
        .with(if notice_below { notice.below() } else { notice })
}

/// The start screen of a stuck scene: "保存" and "ヘルプ" in view.
fn shown() -> PageSpec {
    page(false, true)
}

/// The same screen with "保存" scrolled out of view.
fn hidden() -> PageSpec {
    page(true, true)
}

/// The same screen with "通知設定" scrolled into view.
pub fn revealed() -> PageSpec {
    page(false, false)
}

/// Whether the last screen the browser will show has "保存" out of view.
fn last_hidden(scene: &Scene) -> bool {
    let last = scene.after.back().unwrap_or(&scene.start);
    last.elements.first().is_some_and(|e| e.below)
}

/// The start screen of a stuck scene, with "保存" already operated in the
/// memory of the previous run. "ヘルプ" is in view from the start, so it
/// does not count as newly seen.
pub fn stuck_screen(world: &mut QaWorld) {
    world.scene.start = shown();
    let all: &[(&str, &str)] = &[("button", "保存"), ("link", "ヘルプ"), ("link", "通知設定")];
    remember(&mut world.scene, &[("/", all)], &[(0, "保存", 0)]);
}

/// `times` scrolls that change the screen but reveal nothing new. The
/// screens go back and forth from the last one.
pub fn no_progress(world: &mut QaWorld, times: usize) {
    let mut hide = !last_hidden(&world.scene);
    for _ in 0..times {
        world.scene.replies.push_back(Reply::op("scroll_down", 0.9));
        world
            .scene
            .after
            .push_back(if hide { hidden() } else { shown() });
        hide = !hide;
    }
}

/// `times` answers that stop the step: the judge is not sure enough to go
/// back. Each uses one decision of the budget and nothing else.
pub fn held(world: &mut QaWorld, times: usize) {
    let stops = (0..times).map(|_| Reply::op("back", 0.5));
    world.scene.replies.extend(stops);
}

/// Shows the welcome text on the screen the run ends on, once a step has
/// marked the goal. An answer that stops the step shows nothing new, so
/// without screens after operations the text is on the start screen only
/// when there are no answers.
pub fn apply_goal(world: &mut QaWorld) {
    let scene = &mut world.scene;
    if !scene.marks.contains_key("goal") {
        return;
    }
    let text = Some(WELCOME.to_owned());
    if let Some(last) = scene.after.back_mut() {
        last.text = text;
    } else if scene.replies.is_empty() {
        scene.start.text = text;
    }
}

/// Puts the time limit on a scene that has run out of time. With answers,
/// the last one comes late; without, the run starts late.
pub fn settle_time(world: &mut QaWorld) {
    let scene = &mut world.scene;
    if !scene.marks.contains_key("time_up") {
        return;
    }
    scene.max_duration = Some(LIMIT);
    scene.clock_start = Duration::ZERO;
    match scene.replies.pop_back() {
        Some(last) => scene.replies.push_back(last.at(LIMIT)),
        None => scene.clock_start = LIMIT,
    }
}

/// The exploring scene: the start screen has "保存" and "ヘルプ" when it
/// has nothing, so that the options of the judge are not narrowed to moves.
pub fn explore(scene: &mut Scene) {
    scene.mode = Mode::Explore;
    if scene.start.elements.is_empty() {
        scene.start.elements.push(PageSpec::button("保存"));
        scene.start.elements.push(PageSpec::link("ヘルプ"));
    }
}

/// The reasons the lines of the log give.
pub fn written_reasons(world: &QaWorld) -> Vec<String> {
    let reasons = world
        .lines()
        .into_iter()
        .filter_map(|line| line.get("reason").cloned());
    reasons
        .filter_map(|r| r.as_str().map(str::to_owned))
        .collect()
}
