//! The recent operations the judge is told in the screen text.

use cucumber::{given, then};

use super::replies::{CALM, SAVE, button, each_trial, step};
use crate::world::InterviewWorld;

const HEAD: &str = "\n\n直近の操作\n";

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
    expr = "{int}ステップ目の画面の文の直近の操作は、{string} から始まる5行で、scroll_down は含まれない"
)]
async fn five_lines(world: &mut InterviewWorld, nth: usize, first: String) {
    world.run().await;
    let lines = lines_at(world, 2 * (nth - 1));
    assert_eq!(lines.first(), Some(&first), "the oldest left");
    assert_eq!(lines.len(), 5, "five lines");
    assert!(
        !lines.iter().any(|line| line.contains("scroll_down")),
        "dropped"
    );
}
