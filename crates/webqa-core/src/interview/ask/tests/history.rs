//! Tests of the recent operations added to the screen text.

use super::*;
use crate::machine::Rejection;
use crate::screen::ElementId;

const HEAD: &str = "直近の操作";

fn past(op: &str, name: Option<&str>, after: After) -> Past {
    let target = name.map(|name| ElementId {
        role: "button".to_owned(),
        name: name.to_owned(),
        nth: 0,
    });
    Past {
        op: op.to_owned(),
        target,
        after,
    }
}

/// The screen text the judge gets, the section passed through `redactor`.
fn told(redactor: &Redactor, past: &[Past]) -> String {
    let with = |section: String| with_section(&cart_text(), &redactor.text(&section));
    let text = recent_section(past).map_or_else(cart_text, with);
    text.as_inner().as_str().to_owned()
}

fn shown(past: &[Past]) -> String {
    told(&Redactor::default(), past)
}

fn cart() -> String {
    cart_text().as_inner().as_str().to_owned()
}

#[test]
fn no_operation_adds_no_section() {
    assert_eq!(shown(&[]), cart());
}

#[test]
fn one_operation_is_one_line_after_the_screen_text() {
    let got = shown(&[past("click", Some("検索"), After::Moved)]);
    let want = format!(
        "{}\n\n{HEAD}\n1. click 「検索」 → 別のページに移った",
        cart()
    );
    assert_eq!(got, want);
}

#[test]
fn an_operation_without_target_and_without_change() {
    let got = shown(&[past("back", None, After::Unchanged)]);
    let want = format!(
        "{}\n\n{HEAD}\n1. back 対象なし → 画面は変わらなかった",
        cart()
    );
    assert_eq!(got, want);
}

#[test]
fn five_operations_are_all_kept_oldest_first() {
    let all: Vec<Past> = (1..=5)
        .map(|n| past(&format!("op{n}"), None, After::Shown))
        .collect();
    let lines: Vec<String> = (1..=5)
        .map(|n| format!("{n}. op{n} 対象なし → 同じページで表示が変わった"))
        .collect();
    let want = format!("{}\n\n{HEAD}\n{}", cart(), lines.join("\n"));
    assert_eq!(shown(&all), want);
}

#[test]
fn the_sixth_operation_drops_the_oldest() {
    let all: Vec<Past> = (1..=6)
        .map(|n| past(&format!("op{n}"), None, After::Shown))
        .collect();
    let lines: Vec<String> = (1..=5)
        .map(|n| format!("{n}. op{} 対象なし → 同じページで表示が変わった", n + 1))
        .collect();
    let want = format!("{}\n\n{HEAD}\n{}", cart(), lines.join("\n"));
    assert_eq!(shown(&all), want);
}

#[test]
fn the_redactor_masks_a_secret_in_a_name() {
    let rule = Rule::literal("メールアドレス", "test@example.test");
    let redactor = Redactor::new(&[rule]).unwrap();
    let got = told(
        &redactor,
        &[past("click", Some("test@example.test"), After::Shown)],
    );
    let want = format!(
        "{}\n\n{HEAD}\n1. click 「[[メールアドレス]]」 → 同じページで表示が変わった",
        cart()
    );
    assert_eq!(got, want);
}

#[test]
fn a_control_character_in_a_name_is_escaped() {
    let got = shown(&[past("click", Some("検索\"\u{7}"), After::Shown)]);
    let want = format!(
        "{}\n\n{HEAD}\n1. click 「検索\\\"\\u{{0007}}」 → 同じページで表示が変わった",
        cart()
    );
    assert_eq!(got, want);
}

/// The section of `past`, without the screen text.
fn section(past: &[Past]) -> Option<String> {
    recent_section(past)
}

#[test]
fn a_repeated_operation_is_counted() {
    let scroll = past("scroll_down", None, After::Shown);
    let got = section(&[
        past("click", Some("検索"), After::Moved),
        scroll.clone(),
        scroll.clone(),
        scroll,
    ]);
    let want = [
        "直近の操作",
        "1. click 「検索」 → 別のページに移った",
        "2. scroll_down 対象なし → 同じページで表示が変わった",
        "3. scroll_down 対象なし → 同じページで表示が変わった",
        "4. scroll_down 対象なし → 同じページで表示が変わった",
        "直前の操作(scroll_down、対象なし)を3回続けて選んでいる",
    ];
    assert_eq!(got, Some(want.join("\n")));
}

#[test]
fn a_single_operation_has_no_count() {
    let clicks = [
        past("click", Some("検索"), After::Unchanged),
        past("click", Some("戻る"), After::Unchanged),
    ];
    let want = [
        "直近の操作",
        "1. click 「検索」 → 画面は変わらなかった",
        "2. click 「戻る」 → 画面は変わらなかった",
    ];
    assert_eq!(section(&clicks), Some(want.join("\n")), "another element");
}

#[test]
fn the_count_runs_beyond_the_five_lines() {
    let all = vec![past("scroll_down", None, After::Unchanged); 7];
    let line = "scroll_down 対象なし → 画面は変わらなかった";
    let lines: Vec<String> = (1..=5).map(|n| format!("{n}. {line}")).collect();
    let want = format!(
        "直近の操作\n{}\n直前の操作(scroll_down、対象なし)を7回続けて選んでいる",
        lines.join("\n")
    );
    assert_eq!(section(&all), Some(want));
}

#[test]
fn a_stopped_operation_is_told_as_stopped() {
    let stopped = |why| past("back", None, After::Stopped(why));
    let all = [
        past("click", Some("検索"), After::Moved),
        stopped(Rejection::Forbidden),
        stopped(Rejection::LowConfidence),
        stopped(Rejection::Missing),
        stopped(Rejection::Replaced),
    ];
    let want = [
        "直近の操作",
        "1. click 「検索」 → 別のページに移った",
        "2. back 対象なし → 止められた(禁止の要素)",
        "3. back 対象なし → 止められた(確信度が低い)",
        "4. back 対象なし → 止められた(要素が見つからない)",
        "5. back 対象なし → 止められた(別の要素に替わった)",
        "直前の操作(back、対象なし)を4回続けて選んでいる",
    ];
    assert_eq!(section(&all), Some(want.join("\n")));
}
