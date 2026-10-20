//! Steps of the Rule 台帳の書式: what the ledger the run writes holds, and
//! what it does not.

use cucumber::{given, then};
use webqa_core::interview::issue::{Kind, Number};

use crate::issues::script::{Step, journey};
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::scene::Scene;
use crate::world::InterviewWorld;

#[path = "ledger_content.rs"]
mod content;
#[path = "ledger_head.rs"]
mod head;
#[path = "ledger_items.rs"]
mod items;
#[path = "ledger_numbers.rs"]
mod numbers;
#[path = "ledger_order.rs"]
mod order;
#[path = "ledger_rank.rs"]
mod rank;
#[path = "ledger_shot.rs"]
mod shot;
#[path = "ledger_text.rs"]
mod text;
use text::{blocks, stray};

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "台帳は Markdown の1ファイルで、各課題は直す指示の文を1行目に持つ",
    "各課題は、決まった項目をすべて持つ",
    "課題をソースコードの場所に結び付けず、画面の要素の手がかりまでを書く",
    "3回のうち1回だけ起きた課題も、その回数を添えて台帳に出す",
    "課題は起きた割合の高い順に並べる",
    "起きた割合が同じ課題は、課題の番号の順に並べる",
    "同じ画面の同じ要素の同じ種類の課題は、実行をまたいで同じ番号になる",
    "画面、要素、種類のどれかが違えば、別の番号になる",
    "要素を持たない課題は、画面と種類で番号を決める",
    "スクリーンショットの名前は、課題の番号・試行の番号・ステップの番号から作り、パスの区切りを含めない",
    "台帳を読み戻すと、書いた課題の項目と選択がそのまま戻る",
    "判断モデルの自由な文に台帳の書式が入っていても、台帳の構造を崩さない",
    "判断モデルの自由な文に改行が入っていても、直す指示の文は1行のままにする",
    "台帳に書く文にも、設定で指定した機密情報の伏せ字を通す",
    "台帳に、ペルソナの名前、目的、開始の URL、終えた試行の回数を書く",
    "インタビューの課題だけを見つけて終わったら、終了コードは0になる",
];

/// The start page of the scenes of this Rule, and what it is called in the
/// ledger.
const HOME: &str = "https://shop.example.test/";

/// The button of the scenes of this Rule.
const NEXT: &str = "次へ";

/// The page of these scenes: the home page with `buttons`.
fn home(buttons: &[&str]) -> PageSpec {
    let page = PageSpec::at("/");
    buttons
        .iter()
        .fold(page, |page, button| page.with(PageSpec::button(button)))
}

/// What the ledger says of a hesitation about the button "次へ" on the home
/// page, in a trial where the judge expects `expectation`.
fn hesitation_line(number: Number, expectation: &str) -> String {
    format!(
        "- [ ] {number} {HOME} の画面で、button \"{NEXT}\" について、このペルソナが迷わずに次の操作を選べるようにする(期待: {expectation})"
    )
}

/// The number of the hesitation about the button "次へ" on the home page.
fn hesitation_number() -> Number {
    let target = Some(("button", NEXT, 0));
    number_of("/", &[("button", NEXT)], target, Kind::Hesitant)
}

/// The number of the issue that the trials did not reach the purpose on the
/// home page.
fn unreached_number() -> Number {
    number_of("/", &[("button", NEXT)], None, Kind::Unreached)
}

/// Makes the trials find a hesitation and give up.
fn two_issues(scene: &mut Scene) {
    let steps = [Step::click(1).hesitating(0.9), Step::of("stuck")];
    journey(scene, &[home(&[NEXT])], &steps);
}

/// Makes the trials find one hesitation, and end without a problem.
fn one_issue(scene: &mut Scene) {
    let steps = [Step::click(1).hesitating(0.9), Step::of("done")];
    journey(scene, &[home(&[NEXT])], &steps);
}

#[given(expr = "試行の中で課題が2件見つかっている")]
fn two_found(world: &mut InterviewWorld) {
    two_issues(&mut world.scene);
}

#[given(expr = "試行の中で、判断に迷う課題が1件見つかっている")]
fn one_found(world: &mut InterviewWorld) {
    one_issue(&mut world.scene);
}

#[then(expr = "台帳は Markdown の1つのファイルである")]
async fn one_markdown_file(world: &mut InterviewWorld) {
    world.run().await;
    let text = text::text(world);
    assert_eq!(
        text.lines().next(),
        Some("# インタビューの台帳"),
        "the title"
    );
    assert_eq!(blocks(&text).len(), 2, "both issues are in it");
    assert_eq!(stray(&text), Vec::<String>::new(), "nothing else is in it");
}

#[then(expr = "各課題の1行目は、直す指示の文である")]
async fn first_line_is_instruction(world: &mut InterviewWorld) {
    world.run().await;
    let text = text::text(world);
    let mut firsts: Vec<String> = blocks(&text).into_iter().map(|block| block.first).collect();
    firsts.sort();
    let unreached = format!(
        "- [ ] {} {HOME} の画面で、このペルソナが目的まで進めるようにする(期待: 進む)",
        unreached_number()
    );
    let mut expected = [hesitation_line(hesitation_number(), "進む"), unreached];
    expected.sort();
    assert_eq!(firsts, expected, "the first line of each issue");
}
