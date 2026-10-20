//! The Given steps of what the judge answers after the trials: to the
//! mismatch question, and to the naturalness question about its issue.

use cucumber::given;
use webqa_core::interview::ask::{DIFFERENT, SAME};
use webqa_core::interview::natural::{NO, YES};
use webqa_core::judge::Answers;

use super::drift::written;
use super::{answer_after, mismatch_answer, natural_answer};
use crate::world::InterviewWorld;

/// The mismatch question gets `answers`, in place of what the scene had
/// ready for it.
fn asked_gets(world: &mut InterviewWorld, answers: Vec<Answers>) {
    answer_after(&mut world.scene, answers);
}

#[given(
    expr = "すべての試行を終えたあとのずれの問いで、判断モデルが、これらの推測は書いた目的と同じことを指さないと答える"
)]
#[given(expr = "ずれの問いで、判断モデルが、推測は書いた目的と同じことを指さないと答える")]
fn says_different(world: &mut InterviewWorld) {
    let answers = vec![mismatch_answer(DIFFERENT), natural_answer(YES)];
    asked_gets(world, answers);
}

#[given(
    expr = "すべての試行を終えたあとのずれの問いで、判断モデルが、これらの推測は書いた目的と同じことを指すと答える"
)]
fn says_same(world: &mut InterviewWorld) {
    asked_gets(world, vec![mismatch_answer(SAME)]);
}

#[given(expr = "すべての試行を終えたあとのずれの問いへの判断モデルの答えが 欠けた答え である")]
fn answer_is_missing(world: &mut InterviewWorld) {
    asked_gets(world, vec![Answers::default()]);
}

#[given(
    expr = "すべての試行を終えたあとのずれの問いへの判断モデルの答えが 「同じ」「同じでない」のどちらでもない答え である"
)]
fn answer_is_neither(world: &mut InterviewWorld) {
    asked_gets(world, vec![mismatch_answer("どちらでもない")]);
}

#[given(
    expr = "URL の型 {string} のずれの問いで、判断モデルが同じことを指さないと答え、目的のずれの課題が出る"
)]
fn mismatch_issue_comes_out(world: &mut InterviewWorld, pattern: String) {
    written(&mut world.scene, &pattern, "この画面で目的を果たす");
    // The naturalness question about that issue is answered too: not natural.
    let answers = vec![mismatch_answer(DIFFERENT), natural_answer(NO)];
    asked_gets(world, answers);
    drop(pattern);
}
