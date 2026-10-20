//! Steps of the Rule 台帳の書式 about the order of the issues.

use cucumber::{given, then};
use webqa_core::interview::issue::{Issue, Kind, Number};
use webqa_core::interview::natural::Section;

use super::home;
use crate::issues::script::{Step, entries, trials};
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

/// The buttons of the scene of the order: the issues of the three kinds.
const ORDER: [&str; 3] = ["商品を見る", "カートに入れる", "購入に進む"];

/// The buttons of the scene of the tie.
const TIED: [&str; 2] = ["注文する", "お気に入りに入れる"];

#[given(
    expr = "3回のうち1回起きた課題 A と、3回のうち3回起きた課題 B と、3回のうち2回起きた課題 C がある"
)]
fn a_b_c(world: &mut InterviewWorld) {
    let hesitant = [
        [true, true, true],
        [false, true, true],
        [false, true, false],
    ];
    let run = |row: &[bool; 3]| {
        let clicks = row
            .iter()
            .zip(1..)
            .map(|(&hesitant, n)| Step::click(n).hesitating(if hesitant { 0.9 } else { 0.0 }));
        let steps: Vec<Step> = clicks.chain([Step::of("done")]).collect();
        (vec![home(&ORDER)], steps)
    };
    let runs: Vec<_> = hesitant.iter().map(run).collect();
    trials(&mut world.scene, &runs);
}

/// The name of the button of the issue and how many trials it happened in,
/// for each issue of the body of the ledger, in the order it has them.
fn body_of(world: &InterviewWorld) -> Vec<(Option<String>, usize)> {
    let body = entries(world)
        .into_iter()
        .filter(|entry| entry.section == Section::Body);
    let issues = body.map(|entry| entry.issue);
    let named = |issue: Issue| (issue.target.map(|target| target.name), issue.count.occurred);
    issues.map(named).collect()
}

/// The button of the scene of the order that is the `n`-th.
fn button(n: usize) -> Option<String> {
    ORDER.get(n).map(|name| (*name).to_owned())
}

#[then(expr = "台帳の本体の課題は B、C、A の順に並んでいる")]
async fn b_c_a(world: &mut InterviewWorld) {
    world.run().await;
    let expected = [(button(1), 3), (button(2), 2), (button(0), 1)];
    assert_eq!(body_of(world), expected, "B, C, A");
}

/// The number of the hesitation about the button `n` (from 1) of the tie,
/// with `n`. The two numbers are ranked, the smaller first.
fn tied_numbers() -> [(Number, usize); 2] {
    let elements = TIED.map(|name| ("button", name));
    let number = |n: usize| {
        let name = found(TIED.get(n - 1), "a button");
        let target = Some(("button", *name, 0));
        (number_of("/", &elements, target, Kind::Hesitant), n)
    };
    let mut ranked = [number(1), number(2)];
    ranked.sort();
    ranked
}

/// The runs of the tie: the issue with the larger number is found first,
/// so that the order of the numbers is not the order the trials found the
/// issues in; the third trial finds neither.
fn tied_runs(large: usize, small: usize) -> Vec<(Vec<PageSpec>, Vec<Step>)> {
    let run = |degree: f64| {
        let clicks = [large, small].map(|n| Step::click(n).hesitating(degree));
        let steps = [clicks.to_vec(), vec![Step::of("done")]].concat();
        (vec![home(&TIED)], steps)
    };
    [0.9, 0.9, 0.0].map(run).to_vec()
}

#[given(expr = "課題の番号が {string} と {string} の課題が、どちらも3回のうち2回起きている")]
fn tied(world: &mut InterviewWorld, first: String, second: String) {
    let [small, large] = tied_numbers();
    let mut ids = [first, second];
    ids.sort();
    let [low, high] = ids;
    world.scene.aliases.bind(&low, small.0);
    world.scene.aliases.bind(&high, large.0);
    trials(&mut world.scene, &tied_runs(large.1, small.1));
}

#[then(expr = "{string} の課題が {string} の課題より前にある")]
async fn earlier(world: &mut InterviewWorld, first: String, second: String) {
    world.run().await;
    let order: Vec<Number> = entries(world).iter().map(|e| e.issue.number).collect();
    let at = |id: &str| {
        order
            .iter()
            .position(|n| *n == world.scene.aliases.number(id))
    };
    assert_eq!((at(&first), at(&second)), (Some(0), Some(1)), "the order");
}
