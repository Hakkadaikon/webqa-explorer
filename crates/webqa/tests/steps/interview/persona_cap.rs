//! The cap on the questions about naturalness: the issues over it are not
//! checked.

use cucumber::{given, then};
use webqa_core::interview::issue::Number;
use webqa_core::interview::natural::Section;

use super::{BODY, OFF_PERSONA, UNCHECKED, heading_at, naturalness};
use crate::judge::Asked;
use crate::questions::replies::{
    CALM, HESITANT, button, each_trial, entry_of, hesitation_number, ledger_of, natural_yes,
    prompt, reply, step,
};
use crate::world::InterviewWorld;

/// The buttons of the page; each has an issue of hesitation.
const BUTTONS: [&str; 3] = ["保存", "削除", "戻る"];

/// The three issues in the order of the ledger: the lower number first, as
/// they happened in every trial.
fn ranked() -> [(Number, &'static str); 3] {
    let mut ranked = BUTTONS.map(|name| (hesitation_number(&BUTTONS, Some(name)), name));
    ranked.sort_by_key(|&(number, _)| number);
    ranked
}

#[given(expr = "台帳の並びの順に、課題 {string}、{string}、{string} がある")]
fn three_issues(world: &mut InterviewWorld, first: String, second: String, third: String) {
    let scene = &mut world.scene;
    for name in BUTTONS {
        button(scene, name);
    }
    let click = |n: usize| step("click", Some(&n.to_string()), HESITANT);
    each_trial(
        scene,
        &[click(1), click(2), click(3), step("done", None, CALM)],
    );
    for _ in 0..scene.max_natural {
        reply(scene, natural_yes());
    }
    for (example, (number, _)) in [first, second, third].iter().zip(ranked()) {
        scene.aliases.bind(example, number);
    }
}

/// Whether the question of `call` is about the issue at the button `name`.
fn is_about(call: &Asked, name: &str) -> bool {
    let line = format!(r#"要素: button "{name}""#);
    call.questions
        .iter()
        .any(|q| prompt(q).lines().any(|l| l == line))
}

#[then(expr = "自然さの問いは {string} と {string} について1回ずつ、計 {int} 回だけ問われている")]
async fn asked_for_two(world: &mut InterviewWorld, first: String, second: String, times: usize) {
    world.run().await;
    let [(one, _), (two, _), _] = ranked();
    let aliases = &world.scene.aliases;
    assert_eq!(aliases.number(&first), one, "the first of the ledger");
    assert_eq!(aliases.number(&second), two, "the second of the ledger");
    let calls = naturalness(world);
    assert_eq!(calls.len(), times, "the number of questions");
    let about = |call: &&Asked| ranked().map(|(_, name)| is_about(call, name));
    let asked: Vec<[bool; 3]> = calls.iter().map(about).collect();
    let expected = [[true, false, false], [false, true, false]];
    assert_eq!(asked, expected, "one issue for each question, in order");
}

#[then(expr = "{string} は、確かめられなかった区分にある")]
async fn is_unchecked(world: &mut InterviewWorld, example: String) {
    world.run().await;
    let ledger = ledger_of(world);
    let entry = entry_of(&ledger, world.scene.aliases.number(&example));
    assert_eq!(entry.section, Section::Unchecked, "the section");
}

#[then(
    expr = "確かめられなかった区分は、台帳の本体の後、ペルソナから外れると判断した課題の節の前にある"
)]
async fn sections_in_order(world: &mut InterviewWorld) {
    world.run().await;
    let at = [BODY, UNCHECKED, OFF_PERSONA].map(|heading| heading_at(world, heading));
    assert!(at[0] < at[1] && at[1] < at[2], "the headings: {at:?}");
    assert_ne!(at[2], usize::MAX, "the last section is written");
}
