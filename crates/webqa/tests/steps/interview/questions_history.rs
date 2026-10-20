//! The recent operations the judge is told in the screen text.

use cucumber::{given, then};

use super::replies::{CALM, SAVE, button, each_trial, step};
use crate::world::InterviewWorld;

pub(super) const HEAD: &str = "\n\n直近の操作\n";

#[given(expr = "判断モデルが、scroll_down を選んだあとに click を5回選び、そのあと done を選ぶ")]
fn scrolls_then_clicks(world: &mut InterviewWorld) {
    button(&mut world.scene, SAVE);
    let click = step("click", Some("1"), CALM);
    let mut steps = vec![step("scroll_down", None, CALM)];
    steps.extend(vec![click; 5]);
    steps.push(step("done", None, CALM));
    each_trial(&mut world.scene, &steps);
}

/// The lines of the recent operations in the screen text of the first
/// call of the `n`th step of the first trial; none when it has no section.
/// A step that carries out an operation makes two calls, the last one is done.
fn lines_at(world: &InterviewWorld, call: usize) -> Vec<String> {
    let asked = world.ran().judge.asked.get(call);
    let screen = &asked.map(|asked| asked.screen.clone()).unwrap_or_default();
    let section = screen.split_once(HEAD).map(|(_, section)| section);
    section
        .iter()
        .flat_map(|s| s.lines())
        .map(str::to_owned)
        .collect()
}

#[then(expr = "{int}ステップ目の画面の文に、直近の操作の節は付いていない")]
async fn no_section(world: &mut InterviewWorld, nth: usize) {
    world.run().await;
    let call = 2 * (nth - 1);
    assert_eq!(lines_at(world, call), Vec::<String>::new(), "step {nth}");
}

#[then(expr = "2回目の試行の1ステップ目の画面の文にも、直近の操作の節は付いていない")]
async fn no_section_in_the_next_trial(world: &mut InterviewWorld) {
    world.run().await;
    // The first trial made 6 steps of two calls and a done of one.
    assert_eq!(lines_at(world, 13), Vec::<String>::new(), "the next trial");
}

#[then(expr = "{int}ステップ目の画面の文の最後に、直近の操作として {string} の1行がある")]
async fn one_line(world: &mut InterviewWorld, nth: usize, line: String) {
    world.run().await;
    let call = 2 * (nth - 1);
    assert_eq!(lines_at(world, call), [line], "step {nth}");
}

#[then(
    expr = "{int}ステップ目の画面の文の直近の操作は、1から3行目と5行目が click 「保存」、4行目が中核の reload で、どれも画面は変わらなかった5行である"
)]
async fn five_lines(world: &mut InterviewWorld, nth: usize) {
    world.run().await;
    let click = "click 「保存」 → 画面は変わらなかった";
    let reload = "reload なし → 画面は変わらなかった";
    let ops = [click, click, click, reload, click];
    let want: Vec<String> = ops
        .iter()
        .zip(1..)
        .map(|(op, n)| format!("{n}. {op}"))
        .collect();
    assert_eq!(lines_at(world, 2 * (nth - 1)), want, "the oldest left");
}
