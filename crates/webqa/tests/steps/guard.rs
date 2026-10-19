//! Steps of the rule on the guards: going back and the paywall.

use cucumber::{given, then, when};
use serde_json::json;

use crate::judge::Reply;
use crate::page::PageSpec;
use crate::world::QaWorld;

/// The scenarios that have their steps.
pub const READY: &[&str] = &[
    "確信度が欠けた応答の「戻る」は低確信度として扱い、実行しない",
    "許可リストに一致する要素は、禁止のキーワードを含んでも操作する",
    "実行の直前に対象要素が無くなっていたら操作を中止する",
    "名前が同じでも役割や位置が変わった要素は操作しない",
    "同じ名前のボタンの位置が入れ替わったら操作を中止する",
    "位置で一意に決まる同名のボタンは操作する",
    "通常のページを課金画面と取り違えない",
    "課金画面から離脱しても消えないまま3回続いたら、行き詰まりで終了する",
    "確信度の低い「戻る」は実行しない",
    "購入・削除・退会・サインインに当たる要素は操作しない",
    "課金画面が出たら判断モデルに問い合わせず、既定の離脱操作をする",
];

#[given(expr = "「戻る」の確信度の閾値は {float} に設定されている")]
const fn back_threshold(world: &mut QaWorld, threshold: f64) {
    world.scene.back_confidence = threshold;
}

#[given(expr = "判断モデルが操作 {string} を確信度 {float} で返す")]
fn replies_with(world: &mut QaWorld, op: String, p: f64) {
    world.scene.replies.push_back(Reply::op(op, p));
}

#[given(expr = "判断モデルが操作 {string} を確信度なしで返す")]
fn replies_without_p(world: &mut QaWorld, op: String) {
    world.scene.replies.push_back(Reply::op_without_p(op));
}

#[then(expr = "ブラウザで「戻る」を実行している")]
async fn went_back(world: &mut QaWorld) {
    world.run().await;
    let touched = world.touched();
    assert!(touched.contains(&("back".to_owned(), None)), "{touched:?}");
}

#[then(expr = "ブラウザで「戻る」を実行していない")]
async fn did_not_go_back(world: &mut QaWorld) {
    world.run().await;
    let touched = world.touched();
    assert!(touched.iter().all(|(op, _)| op != "back"), "{touched:?}");
}

/// The page the paywall check reads, and the ordinary page after leaving.
fn paywalled(world: &mut QaWorld, page: PageSpec) {
    let ordinary = PageSpec::at("/").with(PageSpec::button("保存"));
    world.scene.start = page;
    world.scene.after.push_back(ordinary);
}

/// An ordinary page at `path` with a button "保存".
fn ordinary(path: &str) -> PageSpec {
    PageSpec::at(path).with(PageSpec::button("保存"))
}

#[given(expr = "対象ページに {string} のモーダルが表示されているが、主要な操作は塞がれていない")]
fn modal_not_blocking(world: &mut QaWorld, title: String) {
    let page = PageSpec {
        dialog: vec![title],
        ..ordinary("/")
    };
    paywalled(world, page);
}

#[given(
    expr = "対象ページに モーダルの無い通常の URL のページに、見出し {string} とボタン {string} だけがある"
)]
fn heading_and_button(world: &mut QaWorld, heading: String, button: String) {
    let page = PageSpec {
        headings: vec![(heading, false)],
        ..PageSpec::at("/").with(PageSpec::button(&button))
    };
    drop(button);
    paywalled(world, page);
}

#[given(expr = "対象ページに 遷移した URL が {string} で、課金を促す語は表示されていない")]
fn paywall_url(world: &mut QaWorld, path: String) {
    paywalled(world, ordinary(&path));
    drop(path);
}

#[given(expr = "対象ページに 課金を促す語の無いモーダルが表示され、主要な操作がすべて塞がれている")]
fn blocked_modal(world: &mut QaWorld) {
    let page = PageSpec {
        dialog: vec!["お知らせ".to_owned()],
        blocked: true,
        ..ordinary("/")
    };
    paywalled(world, page);
}

#[then(expr = "既定の離脱操作を実行している")]
async fn left_the_paywall(world: &mut QaWorld) {
    world.run().await;
    let touched = world.touched();
    assert!(touched.contains(&("back".to_owned(), None)), "{touched:?}");
    let left = world.lines().into_iter().any(|line| {
        line.get("kind") == Some(&json!("recovered"))
            && line.get("cause") == Some(&json!("paywall"))
    });
    assert!(left, "the core left the paywall");
}

#[then(expr = "時間窓の操作回数は {int} 増えている")]
async fn window_grew(world: &mut QaWorld, used: u32) {
    world.run().await;
    assert_eq!(world.window_used(), used, "the window's count");
}

#[given(expr = "対象ページに {string} という見出しがあるが、主要な操作は塞がれていない")]
fn heading_only(world: &mut QaWorld, heading: String) {
    world.scene.start = PageSpec {
        headings: vec![(heading, false)],
        ..ordinary("/")
    };
}

#[given(expr = "対象ページの課金画面は、既定の離脱操作をしても表示されたままである")]
fn paywall_stays(world: &mut QaWorld) {
    world.scene.start = PageSpec::at("/pricing").with(PageSpec::button("戻る"));
}

#[when(expr = "既定の離脱操作をするステップが {int} 回続く")]
async fn leaves_in_a_row(world: &mut QaWorld, times: usize) {
    world.run().await;
    let touched = world.touched();
    let backs = touched.iter().filter(|(op, _)| op == "back");
    assert_eq!(backs.count(), times, "the core left each time");
}
