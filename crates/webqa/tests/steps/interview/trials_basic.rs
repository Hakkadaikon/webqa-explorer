//! Steps of the trials on what each trial starts with: its browser, its
//! memory and its budget of decisions.

use std::collections::BTreeSet;
use std::path::PathBuf;

use cucumber::{given, then, when};
use webqa_core::explore::{ElementOp, Memory};
use webqa_core::judge::{Question, QuestionId};
use webqa_core::redact::Redactor;
use webqa_core::screen::element_ids;

use super::{End, ending_of, fill, first, found, has, kind_of, openings, within};
use crate::judge::Asked;
use crate::launch::Opened;
use crate::page::PageSpec;
use crate::world::InterviewWorld;

/// The storage state the scenes name.
const AUTH: &str = "auth.json";

#[then(expr = "試行は {int} 回行われている")]
async fn trials_ran(world: &mut InterviewWorld, n: usize) {
    world.run().await;
    let ended = |trial: usize| has(world, "trial_finished", trial);
    let all: Vec<bool> = (1..=n).map(ended).collect();
    assert_eq!(all, vec![true; n], "each trial ran to its end");
    assert_eq!(world.ran().launched, n, "a browser for each");
}

#[then(expr = "台帳は1つのファイルで、{int}回の試行の結果がまとめられている")]
async fn one_ledger(world: &mut InterviewWorld, n: usize) {
    world.run().await;
    let finished = found(world.ledger(), "one ledger").head.finished;
    assert_eq!(finished, n, "the trials of the ledger");
}

#[given(expr = "設定に認証状態が指定されている")]
const fn auth_named(world: &mut InterviewWorld) {
    world.scene.auth = true;
}

#[given(expr = "1回目の試行でカートに商品を入れ、ブラウザの保存領域に値が残っている")]
fn cart_filled(world: &mut InterviewWorld) {
    first(&mut world.scene, 1, End::Goal);
}

#[when(expr = "2回目の試行が始まる")]
async fn second_starts(world: &mut InterviewWorld) {
    fill(&mut world.scene);
    world.run().await;
}

#[then(expr = "ブラウザは新しく開かれ、1回目の保存領域の値は残っていない")]
async fn browser_is_fresh(world: &mut InterviewWorld) {
    world.run().await;
    let touched = |n: usize| {
        let own = world.ran().touches.iter().filter(|(trial, _)| *trial == n);
        own.count()
    };
    assert_eq!((touched(1), touched(2)), (1, 1), "each browser did its own");
    let auth = Opened::Auth(PathBuf::from(AUTH));
    let launched = Opened::Launched {
        started: 0,
        finished: 0,
    };
    let fresh = [launched, auth, Opened::FirstPage];
    assert_eq!(openings(world, 2), fresh, "a new browser, then its page");
}

#[then(expr = "指定した認証状態が入れ直されている")]
async fn auth_put_again(world: &mut InterviewWorld) {
    world.run().await;
    let state = Opened::Auth(PathBuf::from(AUTH));
    let put = |n: usize| openings(world, n).contains(&state);
    assert_eq!((put(1), put(2)), (true, true), "the state of each trial");
}

#[given(expr = "対象のオリジンの探索の記憶のファイルに、操作済みの要素 {string} が記録されている")]
fn remembered(world: &mut InterviewWorld, name: String) {
    let scene = &mut world.scene;
    scene.start = PageSpec::at("/").with(PageSpec::button(&name));
    let origin = scene.origin();
    let memory = memory_of(&origin, &name);
    scene.memories.insert(origin, memory);
    fill(scene);
    drop(name);
}

/// The memory file of a run that pressed the button `name` of the top page.
fn memory_of(origin: &str, name: &str) -> String {
    let shown = PageSpec::at("/").with(PageSpec::button(name));
    let screen = Redactor::default().screen(shown.build(origin).screen);
    let mut memory = Memory::default();
    let key = memory.visit(&screen, &BTreeSet::new());
    let id = found(element_ids(&screen).into_iter().next(), "the button");
    memory.record(key, &id, ElementOp::Click, key);
    memory.to_json(origin).to_string()
}

#[then(expr = "各試行は、操作済みの要素が無い空の記憶から始まっている")]
async fn memory_starts_empty(world: &mut InterviewWorld) {
    world.run().await;
    let asked = &world.ran().judge.asked;
    let all: Vec<Vec<String>> = asked.iter().map(offered).collect();
    let button = vec!["1".to_owned()];
    let expected = vec![button; asked.len()];
    assert_eq!(all, expected, "every call offers the button");
}

/// The options of the question about the target at a call of the judge.
fn offered(call: &Asked) -> Vec<String> {
    let target = QuestionId::target();
    let options = |question: &Question| {
        if let Question::Choice { id, options, .. } = question {
            (*id == target).then(|| options.clone())
        } else {
            None
        }
    };
    call.questions
        .iter()
        .filter_map(options)
        .flatten()
        .collect()
}

#[then(expr = "探索の記憶のファイルは実行の前と同じ中身のままである")]
async fn memory_untouched(world: &mut InterviewWorld) {
    world.run().await;
    let kept = &world.ran().store.memories;
    assert_eq!(kept, &world.scene.memories, "the memory files");
}

#[given(expr = "1回目の試行が判断予算切れで終わる")]
fn budget_spent(world: &mut InterviewWorld) {
    let calls = usize::try_from(world.scene.max_decisions).unwrap_or(usize::MAX);
    first(&mut world.scene, calls, End::Cut);
}

#[then(expr = "2回目の試行は判断モデルに問い合わせる")]
async fn second_asks(world: &mut InterviewWorld) {
    world.run().await;
    let ended = ending_of(world, 1);
    assert_eq!(ended.as_deref(), Some("budget_spent"), "the first ran out");
    let inside = within(world, 2);
    let asked = inside.iter().filter(|line| kind_of(line) == "asked");
    assert_eq!(asked.count(), 1, "the second trial asked the judge");
}
