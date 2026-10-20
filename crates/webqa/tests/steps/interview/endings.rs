//! Steps of this Rule of the feature, and the scenarios that have them.

use serde_json::Value;
use webqa_core::config::purpose::{Aim, Written};
use webqa_core::config::raw::{DoneWhen, Element};
use webqa_core::interview::issue::Kind;
use webqa_core::judge::Answers;

use crate::questions::replies::{SAVE, button, each_trial, ledger_of, natural_yes, reply};
use crate::scene::Scene;
use crate::world::InterviewWorld;

#[path = "endings_claimed.rs"]
mod claimed;
#[path = "endings_goal.rs"]
mod goal;
#[path = "endings_unreached.rs"]
mod unreached;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "目的の done_when を満たしたら、判断モデルに問い合わせずにゴール達成で試行を終える",
    "目的の確定のボタンが禁止要素でも押さず、確定の操作ができる画面に来たところでゴール達成にする",
    "目的を推測させたとき、判断モデルの done は推測した目的に届いたという申告として記録する",
    "目的を書いたのに done_when を満たさないまま done が返ったら、未検証の終了として記録する",
    "判断モデルが stuck を返したら、ペルソナが諦めたものとして行き詰まりで試行を終える",
    "打ち切りでない試行のうち、ゴール達成と申告による終了と判断予算切れのほかで終わった試行は、目的に届かない試行に数える",
    "ゴール達成や申告による終了で終わった試行は、目的に届かない試行に数えない",
    "判断予算切れで終わった試行は、試行の数に数えるが、目的に届かない課題にしない",
];

/// The name of the button the written purpose waits for.
const CONFIRM: &str = "注文を確定する";

/// Writes the purpose with a `done_when` that waits for the button `name`.
fn wait_for_button(scene: &mut Scene, name: &str) {
    let element = Element {
        role: "button".to_owned(),
        name: name.to_owned(),
    };
    let done_when = DoneWhen {
        url: None,
        element: Some(element),
        text: None,
    };
    let goal = "目的を果たす".to_owned();
    scene.purpose.aim = Aim::Written(Written { goal, done_when });
}

/// The `done_when` holds on the first page of every trial.
fn goal_at_start(scene: &mut Scene) {
    button(scene, SAVE);
    wait_for_button(scene, SAVE);
}

/// The judge says `answers` at the first step of every trial, and the
/// issues the trials leave are natural.
fn says(world: &mut InterviewWorld, answers: Answers) {
    each_trial(&mut world.scene, &[answers]);
    reply(&mut world.scene, natural_yes());
}

/// How each trial ended, as the log says.
fn endings(world: &InterviewWorld) -> Vec<String> {
    let finished = Value::from("trial_finished");
    let lines = world.lines();
    let ended = lines
        .iter()
        .filter(|line| line.get("kind") == Some(&finished));
    let ending = |line: &Value| {
        let ending = line.get("ending").and_then(Value::as_str);
        ending.unwrap_or_default().to_owned()
    };
    ended.map(ending).collect()
}

/// For each issue of the ledger about a purpose not reached: in how many
/// trials it happened, and how many trials finished.
fn unreached_counts(world: &InterviewWorld) -> Vec<(usize, usize)> {
    let ledger = ledger_of(world);
    let issues = ledger.entries.iter().map(|entry| &entry.issue);
    let not_reached = issues.filter(|issue| issue.kind() == Kind::Unreached);
    not_reached
        .map(|issue| (issue.count.occurred, issue.count.finished))
        .collect()
}
