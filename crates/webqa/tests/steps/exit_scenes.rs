//! Steps of the Rule of the exit codes: the endings and the issues of a
//! run. The scenarios whose steps are all here are named in `READY`.

use std::time::Duration;

use cucumber::{given, when};
use webqa_core::config::raw::{DoneWhen, Stage};
use webqa_core::machine::ExitReason;
use webqa_core::machine::StuckKind;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::reason::Named;
use crate::scene::{Scene, found};
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "確定の問題を見つけずに終わったら、終了コードは0になる",
    "確定の問題を1件でも見つけて終わったら、終了コードは1になる",
    "参考情報だけを記録して終わったら、終了コードは0になる",
];

/// What a scene of an ending or an issue does to the scene.
type Make = fn(&mut Scene);

/// The origin outside the allowed ones.
const OTHER: &str = "https://other.example.net";

/// The first answer of the judge, changed by `change`.
fn amend(scene: &mut Scene, change: impl FnOnce(Reply) -> Reply) {
    let first = found(scene.replies.pop_front(), "the scene has a first answer");
    scene.replies.push_front(change(first));
}

/// The one stage that needs the text "ようこそ".
fn welcome_stage(scene: &mut Scene) {
    let done_when = DoneWhen {
        url: None,
        element: None,
        text: Some("ようこそ".to_owned()),
    };
    let goal = "ログインする".to_owned();
    scene.stages = vec![Stage {
        name: "ログイン".to_owned(),
        goal,
        done_when,
    }];
}

fn goal_achieved(scene: &mut Scene) {
    welcome_stage(scene);
    let page = found(scene.after.front_mut(), "the page after the click");
    page.text = Some("ようこそ".to_owned());
}

fn reported_done(scene: &mut Scene) {
    welcome_stage(scene);
    scene.replies.push_back(Reply::op("done", 0.9).goal(0.9));
}

fn stuck(scene: &mut Scene) {
    scene.replies.push_back(Reply::op("stuck", 0.9));
}

fn out_of_budget(scene: &mut Scene) {
    scene.limits.insert("max_decisions", 1);
}

fn time_up(scene: &mut Scene) {
    let minute = Duration::from_secs(60);
    scene.max_duration = Some(minute);
    amend(scene, |reply| reply.at(minute));
}

/// How each ending is made, after the base.
const ENDINGS: [(ExitReason, Make); 5] = [
    (ExitReason::GoalAchieved, goal_achieved),
    (ExitReason::ReportedDone, reported_done),
    (ExitReason::Stuck(StuckKind::NoProgress), stuck),
    (ExitReason::BudgetExceeded, out_of_budget),
    (ExitReason::TimeExceeded, time_up),
];

fn error_shown(scene: &mut Scene) {
    amend(scene, |reply| reply.error(0.9));
}

fn unreadable_name(scene: &mut Scene) {
    let icon = || PageSpec::button("icon_save.png");
    scene.start.elements.push(icon());
    let page = found(scene.after.front_mut(), "the page after the click");
    page.elements.push(icon());
}

fn console_error(scene: &mut Scene) {
    let page = found(scene.after.front_mut(), "the page after the click");
    page.console.push("TypeError: x is undefined".to_owned());
}

fn leaves_origin(scene: &mut Scene) {
    scene.start.elements.push(PageSpec::link("外部の記事"));
    scene.replies.pop_front();
    scene
        .replies
        .push_front(Reply::op("click", 0.9).on("外部の記事"));
    let outside = PageSpec {
        origin: Some(OTHER.to_owned()),
        path: String::new(),
        ..PageSpec::default()
    };
    scene
        .after
        .push_front(outside.with(PageSpec::button("記事")));
}

fn hint(scene: &mut Scene) {
    scene.probes = true;
    amend(scene, |reply| reply.confusing(0.9));
}

/// How each issue is made, before the ending.
const ISSUES: [(&str, Make); 5] = [
    ("エラー表示の問題1件", error_shown),
    ("読み上げ名の問題1件", unreadable_name),
    ("コンソールエラーの問題1件", console_error),
    ("許可ドメインの外への遷移の記録1件", leaves_origin),
    ("参考情報1件だけ", hint),
];

/// The scene every ending starts from: a button, a click on it, and the
/// same page after it.
fn base(scene: &mut Scene) {
    scene.show_button("保存");
    scene.replies.push_back(Reply::op("click", 0.9).on("保存"));
    scene.after.push_back(scene.start.clone());
}

/// The scene of `issue` (if any) and then of the ending `named`.
fn ended_with(world: &mut QaWorld, named: Named, issue: &str) {
    let scene = &mut world.scene;
    base(scene);
    for (_, make) in ISSUES.iter().filter(|(name, _)| *name == issue) {
        make(scene);
    }
    let at = ENDINGS.iter().position(|(reason, _)| *reason == named.0);
    let at = found(at, "the ending is one of the table");
    let (_, ending) = found(ENDINGS.get(at), "the ending is in the table");
    ending(scene);
    scene.marks.insert("ending", u64::try_from(at).unwrap_or(0));
}

#[given(expr = "実行が{reason}として終了し、確定の問題は記録されていない")]
fn ended_clean(world: &mut QaWorld, named: Named) {
    ended_with(world, named, "");
}

#[given(expr = "実行が{reason}として終了し、エラー表示の問題1件が記録されている")]
fn ended_with_error(world: &mut QaWorld, named: Named) {
    ended_with(world, named, "エラー表示の問題1件");
}

#[given(expr = "実行が{reason}として終了し、読み上げ名の問題1件が記録されている")]
fn ended_with_name(world: &mut QaWorld, named: Named) {
    ended_with(world, named, "読み上げ名の問題1件");
}

#[given(expr = "実行が{reason}として終了し、コンソールエラーの問題1件が記録されている")]
fn ended_with_console(world: &mut QaWorld, named: Named) {
    ended_with(world, named, "コンソールエラーの問題1件");
}

#[given(expr = "実行が{reason}として終了し、許可ドメインの外への遷移の記録1件が記録されている")]
fn ended_with_leave(world: &mut QaWorld, named: Named) {
    ended_with(world, named, "許可ドメインの外への遷移の記録1件");
}

#[given(expr = "実行がゴール達成として終了し、記録した検出は参考情報1件だけである")]
fn ended_with_hint(world: &mut QaWorld) {
    ended_with(world, Named(ExitReason::GoalAchieved), "参考情報1件だけ");
}

#[when(expr = "ツールが終わる")]
async fn tool_ends(world: &mut QaWorld) {
    world.run().await;
    let at = world.scene.marks.get("ending").copied().unwrap_or(0);
    let ending = ENDINGS.get(usize::try_from(at).unwrap_or(0));
    let (reason, _) = found(ending, "an ending");
    let matched = Named(*reason).matches(world.reason());
    assert!(matched, "the run ended as {:?}", world.reason());
}
