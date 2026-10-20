//! The Given steps of the purposes of screens: what is written for the
//! screens, where the trials go, and what the judge answers after them.

use std::iter;

use cucumber::given;
use webqa_core::config::purpose::{ScreenPurpose, UrlPattern};
use webqa_core::interview::ask::DIFFERENT;
use webqa_core::interview::natural::YES;

use super::{
    GUESS, Visit, at, every_trial, journey, mismatch_answer, natural_answer, page, reset,
    same_answers, say, step, unguessed,
};
use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// The purpose written for the screens of `pattern`.
fn screen_of(pattern: &str, purpose: &str) -> ScreenPurpose {
    let pattern = found(UrlPattern::parse(pattern), "the pattern is well formed");
    ScreenPurpose {
        pattern,
        purpose: purpose.to_owned(),
    }
}

/// Writes `purpose` for the screens of `pattern`, and lets every trial
/// reach such a screen where the judge guesses something.
pub(super) fn written(scene: &mut Scene, pattern: &str, purpose: &str) {
    scene.purpose.screens.push(screen_of(pattern, purpose));
    let sample = pattern.replace('*', "1");
    every_trial(scene, &[at(&sample, GUESS)], same_answers());
}

#[given(expr = "画面ごとの目的として、URL の型 {string} に {string} を書いている")]
fn written_for(world: &mut InterviewWorld, pattern: String, purpose: String) {
    written(&mut world.scene, &pattern, &purpose);
    drop((pattern, purpose));
}

#[given(
    expr = "画面ごとの目的として、URL の型 {string} に {string} を書き、試行の回数は {int} 回である"
)]
fn written_for_trials(world: &mut InterviewWorld, pattern: String, purpose: String, trials: u32) {
    world.scene.trials = trials;
    written(&mut world.scene, &pattern, &purpose);
    drop((pattern, purpose));
}

#[given(
    expr = "判断モデルが、URL {string} の画面の目的を、{int}回の試行のうち{int}回 {string}、{int}回 {string} と推測する"
)]
fn guesses_by_trial(
    world: &mut InterviewWorld,
    url: String,
    all: u32,
    first_times: usize,
    first: String,
    second_times: usize,
    second: String,
) {
    let scene = &mut world.scene;
    assert_eq!(scene.trials, all, "the guesses are of every trial");
    let total = usize::try_from(all).unwrap_or(0);
    assert_eq!(first_times + second_times, total, "every trial has a guess");
    reset(scene);
    let guesses = [(first, first_times), (second, second_times)];
    let each = guesses
        .iter()
        .flat_map(|(guess, times)| iter::repeat_n(guess, *times));
    for (trial, guess) in each.enumerate() {
        journey(scene, trial, &[at(&url, guess)]);
    }
    say(scene, same_answers());
    drop(url);
}

#[given(
    expr = "判断モデルが、URL {string} の画面の目的を {string}、URL {string} の画面の目的を {string} と {string} と推測する"
)]
fn guesses_at_two_screens(
    world: &mut InterviewWorld,
    one: String,
    one_guess: String,
    other: String,
    first: String,
    second: String,
) {
    let scene = &mut world.scene;
    assert_eq!(scene.trials, 3, "three trials share the guesses");
    reset(scene);
    journey(scene, 0, &[at(&one, &one_guess), at(&other, &first)]);
    journey(scene, 1, &[at(&one, &one_guess), at(&other, &second)]);
    journey(scene, 2, &[at(&one, &one_guess)]);
    say(scene, same_answers());
    drop((one, one_guess, other, first, second));
}

#[given(expr = "判断モデルが、URL {string} の画面の目的を {string} と推測する")]
fn guesses_at(world: &mut InterviewWorld, url: String, guess: String) {
    // A guess compared with a purpose that was not written would be found
    // different, and would be an issue.
    let compared = vec![mismatch_answer(DIFFERENT), natural_answer(YES)];
    every_trial(&mut world.scene, &[at(&url, &guess)], compared);
    drop((url, guess));
}

/// The visits of a trial that goes through `step - 1` other pages and
/// reaches `url` at its step number `step`.
fn reaching(step: usize, url: &str) -> Vec<Visit> {
    let before = (2..step).map(|n| at(&format!("/steps/{n}"), GUESS));
    let first = iter::once(at("/", GUESS));
    first.chain(before).chain([at(url, GUESS)]).collect()
}

#[given(
    expr = "1回目の試行のステップ {int} で URL {string} に、3回目の試行のステップ {int} で URL {string} に届き、2回目の試行は型に合う画面に届いていない"
)]
fn reached_by_two_of_three(
    world: &mut InterviewWorld,
    first_step: usize,
    first_url: String,
    third_step: usize,
    third_url: String,
) {
    let scene = &mut world.scene;
    assert_eq!(scene.trials, 3, "the second trial is between the two");
    reset(scene);
    journey(scene, 0, &reaching(first_step, &first_url));
    journey(scene, 1, &[at("/", GUESS)]);
    journey(scene, 2, &reaching(third_step, &third_url));
    say(scene, same_answers());
    // The screenshot of an issue comes from the image sent with the question.
    scene.images = true;
    drop((first_url, third_url));
}

#[given(expr = "どの終えた試行も、URL の型 {string} に合う画面に届いていない")]
fn reached_by_none(world: &mut InterviewWorld, pattern: String) {
    let scene = &mut world.scene;
    assert!(
        scene
            .purpose
            .screens
            .iter()
            .any(|screen| screen.pattern.as_str() == pattern),
        "the pattern is written"
    );
    every_trial(scene, &[at("/", GUESS)], same_answers());
    drop(pattern);
}

#[given(expr = "URL {string} の画面に届いたのは、利用者が中断した試行だけである")]
fn reached_by_the_interrupted(world: &mut InterviewWorld, url: String) {
    let scene = &mut world.scene;
    scene.start = page(&url);
    reset(scene);
    // The judge has nothing more to say: the run is interrupted at step 2.
    scene.after = vec![vec![page(&url), page("/steps/2")]];
    say(scene, [step("click", 0.9, Some(GUESS))]);
    drop(url);
}

#[given(
    expr = "URL {string} の画面に届いたステップの応答は、どれも目的の推測の答えが欠けた応答の異常である"
)]
fn answers_without_a_guess(world: &mut InterviewWorld, url: String) {
    every_trial(&mut world.scene, &[unguessed(&url)], same_answers());
    drop(url);
}

#[given(
    expr = r"画面ごとの目的の項目\(URL の型\)が3つあり、どの型に合う画面にも終えた試行が届き、使える推測がある"
)]
fn three_screens_reached(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    let written = [
        ("/products/*", "商品をカートに入れる"),
        ("/orders/*", "注文を確定する"),
        ("/help/*", "問い合わせる"),
    ];
    scene
        .purpose
        .screens
        .extend(written.map(|(pattern, purpose)| screen_of(pattern, purpose)));
    let pages = ["/", "/products/1", "/orders/1", "/help/1"];
    let visits: Vec<Visit> = pages.iter().map(|path| at(path, GUESS)).collect();
    reset(scene);
    journey(scene, 0, &visits);
    journey(scene, 1, &[at("/", GUESS)]);
    journey(scene, 2, &[at("/", GUESS)]);
    // Both the questions that the cap allows find an issue, which would be
    // asked about next if the cap allowed it.
    say(
        scene,
        [mismatch_answer(DIFFERENT), mismatch_answer(DIFFERENT)],
    );
}
