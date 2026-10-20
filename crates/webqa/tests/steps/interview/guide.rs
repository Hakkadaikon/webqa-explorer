//! Steps of this Rule of the feature, and the scenarios that have them.
//! The check of the documents is the executable `webqa check-docs`; the
//! guide is the file the distribution carries.

use std::fs;
use std::path::{Path, PathBuf};

use cucumber::{given, then};
use webqa_core::config::{persona, purpose};

use crate::rerun::course::{stash, stashed};

#[path = "guide_check.rs"]
mod check;
use crate::scene::found;
use crate::world::InterviewWorld;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "形式の検査のサブコマンドは、正しい文書を通す",
    "形式の検査のサブコマンドは、誤りのある文書の誤りの場所を示して失敗する",
    "手引きは Agent Skills の標準の形式で、形式の検査のサブコマンドを呼ぶ手順と書式の例を持つ",
    "手引きは、目的の done_when を確定の操作ができる画面まで来たことで書くよう示す",
    "手引きは、このリポジトリの開発用の手順書とは別の場所に置く",
];

/// The guide, from the root of the repository.
const GUIDE: &str = "share/skills/writing-persona-purpose/SKILL.md";

/// The directory of the skills of this repository's own development.
const DEV_SKILLS: &str = "skills";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The text of the guide, which the Given stashed.
fn guide(world: &mut InterviewWorld) -> String {
    stashed(world, "GUIDE")
}

/// The bodies of the fenced yaml blocks that contain `key`.
fn yaml_blocks<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    let blocks = text.split("```yaml\n").skip(1);
    let bodies = blocks.filter_map(|rest| rest.split("```").next());
    bodies.filter(|block| block.contains(key)).collect()
}

/// The lines of the front matter of the guide.
fn front_matter(text: &str) -> Vec<&str> {
    let front = found(
        text.strip_prefix("---\n"),
        "the guide starts with front matter",
    );
    found(front.split("\n---\n").next(), "the front matter ends")
        .lines()
        .collect()
}

#[given(expr = "配布物の中の、ペルソナと目的を書かせる手引きを読む")]
fn read_guide(world: &mut InterviewWorld) {
    let text = fs::read_to_string(root().join(GUIDE));
    let text = found(text.ok(), "the guide reads");
    assert!(!text.is_empty(), "the guide is not empty");
    stash(world, "GUIDE", &text);
}

#[then(expr = "手引きは Agent Skills の標準の形式の前付けを持つ")]
fn guide_has_front_matter(world: &mut InterviewWorld) {
    let text = guide(world);
    let front = front_matter(&text);
    let field = |key: &str| front.iter().filter(|l| l.starts_with(key)).count();
    let named = front
        .iter()
        .filter(|l| **l == "name: writing-persona-purpose")
        .count();
    assert_eq!(
        (named, field("name:"), field("description:")),
        (1, 1, 1),
        "name and description"
    );
}

#[then(expr = "手引きに、インタビューのメモや分析のデータから項目を埋める手順と、書式の例がある")]
fn guide_fills_and_shows(world: &mut InterviewWorld) {
    let text = guide(world);
    assert!(
        text.contains("インタビューのメモや分析のデータを読み"),
        "the steps start from the notes"
    );
    let keys = ["name", "profile", "it_literacy", "wants", "avoids"];
    let lacking: Vec<&str> = keys
        .into_iter()
        .filter(|k| !text.contains(&format!("- `{k}`:")))
        .collect();
    assert_eq!(lacking, Vec::<&str>::new(), "the fields the steps fill");
    let personas = yaml_blocks(&text, "it_literacy:");
    assert_eq!(personas.len(), 1, "one persona example");
    let parsed = persona::parse(found(personas.first().copied(), "an example"));
    assert_eq!(
        found(parsed.ok(), "the example reads").name,
        "佐藤さん",
        "the example persona"
    );
}

#[then(expr = "手引きに、書いた文書を形式の検査のサブコマンドで確かめる手順がある")]
fn guide_checks(world: &mut InterviewWorld) {
    let text = guide(world);
    let calls = ["webqa check-docs persona ", "webqa check-docs purpose "];
    let counts: Vec<usize> = calls
        .iter()
        .map(|call| text.matches(call).count())
        .collect();
    assert_eq!(counts, [1, 1], "each document has its check");
}

#[then(
    expr = "手引きに、目的の done_when は確定の操作ができる画面まで来たことで書く、という書き方と例がある"
)]
fn guide_ends_at_confirm(world: &mut InterviewWorld) {
    let text = guide(world);
    let needles = [
        "確定の操作ができる画面まで来たこと",
        "確定の操作そのものを実行したこと",
    ];
    let lacking: Vec<&str> = needles.into_iter().filter(|n| !text.contains(n)).collect();
    assert_eq!(lacking, Vec::<&str>::new(), "the way to write it");
    let blocks = yaml_blocks(&text, "goal:");
    let parsed = purpose::parse(found(blocks.first().copied(), "an example"));
    let aim = found(parsed.ok(), "the example reads").aim;
    let url = match aim {
        purpose::Aim::Written(written) => written.done_when.url,
        purpose::Aim::Inferred => None,
    };
    assert_eq!(
        url,
        Some("/order/confirm".to_owned()),
        "the example ends at the confirm screen"
    );
}

#[given(expr = "配布物の中の、ペルソナと目的を書かせる手引きの場所を見る")]
fn see_place(world: &mut InterviewWorld) {
    assert!(
        root().join(GUIDE).is_file(),
        "the guide is in the distribution"
    );
    stash(world, "GUIDE_FILE", GUIDE);
}

#[then(expr = "手引きは、このリポジトリの開発用の手順書の置き場の外にある")]
fn guide_is_outside_dev_skills(world: &mut InterviewWorld) {
    let guide = PathBuf::from(stashed(world, "GUIDE_FILE"));
    let skill = found(
        guide.parent().and_then(Path::file_name),
        "the guide is in a directory",
    );
    let homes = [DEV_SKILLS, ".claude/skills", ".agents/skills"];
    let there: Vec<bool> = homes
        .iter()
        .map(|home| root().join(home).join(skill).exists())
        .collect();
    assert_eq!(there, [false; 3], "a copy among the skills of development");
}
