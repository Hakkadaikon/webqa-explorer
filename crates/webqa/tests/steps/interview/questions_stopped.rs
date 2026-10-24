//! The recent operations the judge is told: another page, a stopped
//! proposal, and how many times in a row.

use cucumber::gherkin::Step;
use cucumber::{given, then};
use webqa_core::judge::Question;

use super::history::HEAD;
use super::replies::{CALM, each_trial, natural_yes, reply, step};
use crate::judge::{Asked, Said};
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

#[given(
    expr = "判断モデルが、{string} を押して別のページに移ったあと、確信度 {float} の back を2回返し、そのあと done を返す"
)]
fn moves_then_goes_back(world: &mut InterviewWorld, name: String, p: f64) {
    let scene = &mut world.scene;
    let at = |path: &str| PageSpec::at(path).with(PageSpec::button(&name));
    scene.start = at("/");
    // Each trial: the page seen when the button is looked up again, then
    // the next page.
    let trials = usize::try_from(scene.trials).unwrap_or(0);
    scene.after = vec![vec![at("/"), at("/next")]; trials];
    let back = Said::default()
        .choose("operation", "back", p)
        .choose("error_shown", "true", 0.0)
        .choose("goal_reached", "true", 0.0)
        .choose("hesitation", "true", CALM)
        .write("hard_to_see", "")
        .write("purpose_guess", "買い物をする")
        .build();
    let steps = [
        step("click", Some("1"), CALM),
        back.clone(),
        back,
        step("done", None, CALM),
    ];
    each_trial(scene, &steps);
    // The unsure backs make one hesitation issue, found natural.
    reply(scene, natural_yes());
    drop(name);
}

/// The first call of the `nth` step, from 1: the one that asks for the
/// operation.
fn step_call(world: &InterviewWorld, nth: usize) -> &Asked {
    let asks_operation = |asked: &&Asked| {
        let ids = asked.questions.iter().map(Question::id);
        ids.into_iter().any(|id| id.as_str() == "operation")
    };
    let calls = world.ran().judge.asked.iter().filter(asks_operation);
    found(calls.skip(nth - 1).take(1), "the step was asked")
}

#[then(expr = "{int}ステップ目の画面の文の直近の操作は、次の行である")]
async fn section_lines(world: &mut InterviewWorld, nth: usize, step: &Step) {
    world.run().await;
    let table = found(step.table.as_ref(), "the step has a table");
    let want: Vec<String> = table.rows.iter().skip(1).flatten().cloned().collect();
    let screen = &step_call(world, nth).screen;
    let section = screen.split_once(HEAD).map(|(_, section)| section);
    let lines: Vec<String> = section
        .iter()
        .flat_map(|s| s.lines())
        .map(str::to_owned)
        .collect();
    assert_eq!(lines, want, "step {nth}");
}
