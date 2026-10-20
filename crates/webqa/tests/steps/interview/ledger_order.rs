//! Steps of the Rule 台帳の書式 about the fields an issue has, how many
//! trials it happened in, and the order of the issues.

use cucumber::{given, then};
use webqa_core::interview::issue::{Count, Number, ScreenMark};
use webqa_core::interview::natural::Section;
use webqa_core::screen::ScreenId;

use super::text::{self, blocks};
use super::{NEXT, hesitation_number, home, unreached_number};
use crate::issues::script::{Step, entries, trials};
use crate::scene::found;
use crate::world::InterviewWorld;

/// The names of the fields of an issue, in the order the ledger has them:
/// none of them is about a source file or a line of it.
const FIELDS: [&str; 14] = [
    "種類",
    "画面",
    "画面の識別",
    "要素",
    "セレクタの手がかり",
    "スクリーンショット",
    "操作の列",
    "ペルソナの性質",
    "理由",
    "期待",
    "起きた回数",
    "画面に届いた試行",
    "確信度",
    "前回との照らし合わせ",
];

#[then(expr = "各課題に、画面の要素の手がかりが書かれている")]
async fn has_clues(world: &mut InterviewWorld) {
    world.run().await;
    let screen = ScreenId {
        path: "/".to_owned(),
        elements: [("button".to_owned(), NEXT.to_owned())].into(),
    };
    let mark = ScreenMark::of(&screen).to_string();
    let selector = r#"getByRole("button", { name: "次へ" }).nth(0)"#;
    let clues = |number: Number| {
        let block = blocks(&text::text(world));
        let of = block
            .into_iter()
            .find(|b| b.first.contains(&number.to_string()));
        let of = found(of, "the issue is in the ledger");
        (
            of.field("画面の識別").map(str::to_owned),
            of.field("セレクタの手がかり").map(str::to_owned),
        )
    };
    let hesitant = (Some(mark.clone()), Some(selector.to_owned()));
    assert_eq!(
        clues(hesitation_number()),
        hesitant,
        "the hesitation's clues"
    );
    let unreached = (Some(mark), Some("なし".to_owned()));
    assert_eq!(
        clues(unreached_number()),
        unreached,
        "the clues without an element"
    );
}

#[then(expr = "各課題に、ソースファイルの名前や行番号の項目は無い")]
async fn has_no_source(world: &mut InterviewWorld) {
    world.run().await;
    let all = blocks(&text::text(world));
    assert_eq!(all.len(), 2, "both issues");
    for block in &all {
        assert_eq!(block.names(), FIELDS, "only the fixed fields");
    }
}

/// Makes the three trials find the hesitation about the button "次へ" in
/// those of `hesitant`, which count from 1.
fn hesitant_in(world: &mut InterviewWorld, hesitant: &[usize]) {
    let run = |trial: usize| {
        let degree = if hesitant.contains(&trial) { 0.9 } else { 0.0 };
        let steps = vec![Step::click(1).hesitating(degree), Step::of("done")];
        (vec![home(&[NEXT])], steps)
    };
    let runs: Vec<_> = (1..=3).map(run).collect();
    trials(&mut world.scene, &runs);
}

#[given(expr = "判断に迷う課題が、3回の試行のうち1回だけで起きている")]
fn once_in_three(world: &mut InterviewWorld) {
    hesitant_in(world, &[1]);
}

#[then(expr = "その課題は台帳の本体にあり、3回のうち1回起きたと書かれている")]
async fn once_in_body(world: &mut InterviewWorld) {
    world.run().await;
    let found: Vec<(Section, Count)> = entries(world)
        .iter()
        .map(|entry| (entry.section, entry.issue.count))
        .collect();
    let count = Count {
        occurred: 1,
        finished: 3,
        reached: 3,
    };
    assert_eq!(found, [(Section::Body, count)], "the issue, in the body");
    let text = text::text(world);
    let all = blocks(&text);
    let counts: Vec<Option<&str>> = all.iter().map(|b| b.field("起きた回数")).collect();
    assert_eq!(counts, [Some("3回の試行のうち1回")], "what the ledger says");
}
