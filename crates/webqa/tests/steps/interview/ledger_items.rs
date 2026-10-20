//! Steps of the Rule 台帳の書式 about the items every issue holds.

use cucumber::{gherkin, then};
use webqa_core::interview::issue::ScreenMark;
use webqa_core::screen::ScreenId;

use super::text::{self, blocks};
use super::{NEXT, hesitation_line, hesitation_number};
use crate::scene::found;
use crate::world::InterviewWorld;

/// The items that the first line of an issue holds.
const FIRST_LINE: [&str; 3] = [
    "選んだかどうかのチェックボックス",
    "実行をまたいで変わらない課題の番号",
    "直す指示の文",
];

/// The item that has the mark of the screen's identity besides the URL.
const SCREEN: &str = "画面の URL と画面の識別";

/// The fields of the items that follow the first line, as the ledger
/// writes the hesitation about the button "次へ" on the home page: the item,
/// the field and its value.
const FIELDS: [(&str, &str, &str); 13] = [
    ("種類", "種類", "判断に迷う"),
    (SCREEN, "画面", "\"https://shop.example.test/\""),
    (
        "対象の要素の役割、名前、位置",
        "要素",
        r#"{"役割":"button","名前":"次へ","順番":0,"位置":{"x":0,"y":0,"幅":100,"高さ":20}}"#,
    ),
    (
        "セレクタの手がかり",
        "セレクタの手がかり",
        r#"getByRole("button", { name: "次へ" }).nth(0)"#,
    ),
    (
        "スクリーンショットの場所",
        "スクリーンショット",
        r#"{"試行":1,"ステップ":1}"#,
    ),
    ("そこに至るまでの操作の列", "操作の列", "[]"),
    (
        "ペルソナのどの性質からそう判断したか",
        "ペルソナの性質",
        r#"{"性質":"ITの慣れ","中身":"スマートフォンの基本操作だけ"}"#,
    ),
    (
        "判断モデルが述べた理由と期待",
        "理由",
        r#"["迷いの度合いが閾値以上"]"#,
    ),
    ("判断モデルが述べた理由と期待", "期待", r#""進む""#),
    (
        "何回の試行のうち何回起きたか",
        "起きた回数",
        "3回の試行のうち3回",
    ),
    ("その画面に届いた試行の数", "画面に届いた試行", "3"),
    ("確信度", "確信度", "0.9"),
    (
        "前回の実行との照らし合わせ",
        "前回との照らし合わせ",
        "新しく出た",
    ),
];

/// The mark of the screen the hesitation is on.
fn mark() -> String {
    let screen = ScreenId {
        path: "/".to_owned(),
        elements: [("button".to_owned(), NEXT.to_owned())].into(),
    };
    ScreenMark::of(&screen).to_string()
}

/// The lines of an issue the ledger has for `item`.
fn lines_of(item: &str) -> Vec<String> {
    let fields = FIELDS.iter().filter(|(name, ..)| *name == item);
    let fixed = fields.map(|(_, field, value)| format!("  - {field}: {value}"));
    let first = FIRST_LINE
        .contains(&item)
        .then(|| hesitation_line(hesitation_number(), "進む"));
    let marked = (item == SCREEN).then(|| format!("  - 画面の識別: {}", mark()));
    fixed.chain(first).chain(marked).collect()
}

#[then(expr = "その課題は次の項目を持つ")]
async fn has_items(world: &mut InterviewWorld, step: &gherkin::Step) {
    world.run().await;
    let all = blocks(&text::text(world));
    assert_eq!(all.len(), 1, "one issue");
    let block = found(all.first(), "the ledger has the issue");
    let fields = block.fields.iter();
    let written: Vec<String> = fields
        .map(|(name, value)| format!("  - {name}: {value}"))
        .collect();
    let table = found(step.table.as_ref(), "the step has a table");
    for row in table.rows.iter().skip(1) {
        let item = found(row.first(), "the row has an item");
        let wanted = lines_of(item);
        assert!(!wanted.is_empty(), "{item} is an item of the ledger");
        let held = wanted
            .iter()
            .filter(|line| **line == block.first || written.contains(line))
            .count();
        assert_eq!(held, wanted.len(), "the ledger has {item}");
    }
}
