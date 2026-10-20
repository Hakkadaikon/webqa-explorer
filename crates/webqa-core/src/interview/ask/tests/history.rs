//! Tests of the recent operations added to the screen text.

use super::*;
use crate::screen::ElementId;

const HEAD: &str = "直近の操作";

fn past(op: &str, name: Option<&str>, changed: bool) -> Past {
    let target = name.map(|name| ElementId {
        role: "button".to_owned(),
        name: name.to_owned(),
        nth: 0,
    });
    Past {
        op: op.to_owned(),
        target,
        changed,
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
    let got = shown(&[past("click", Some("検索"), true)]);
    let want = format!("{}\n\n{HEAD}\n1. click 「検索」 → 画面が変わった", cart());
    assert_eq!(got, want);
}

#[test]
fn an_operation_without_target_and_without_change() {
    let got = shown(&[past("back", None, false)]);
    let want = format!("{}\n\n{HEAD}\n1. back なし → 画面は変わらなかった", cart());
    assert_eq!(got, want);
}

#[test]
fn five_operations_are_all_kept_oldest_first() {
    let all: Vec<Past> = (1..=5)
        .map(|n| past(&format!("op{n}"), None, true))
        .collect();
    let lines: Vec<String> = (1..=5)
        .map(|n| format!("{n}. op{n} なし → 画面が変わった"))
        .collect();
    let want = format!("{}\n\n{HEAD}\n{}", cart(), lines.join("\n"));
    assert_eq!(shown(&all), want);
}

#[test]
fn the_sixth_operation_drops_the_oldest() {
    let all: Vec<Past> = (1..=6)
        .map(|n| past(&format!("op{n}"), None, true))
        .collect();
    let lines: Vec<String> = (1..=5)
        .map(|n| format!("{n}. op{} なし → 画面が変わった", n + 1))
        .collect();
    let want = format!("{}\n\n{HEAD}\n{}", cart(), lines.join("\n"));
    assert_eq!(shown(&all), want);
}

#[test]
fn the_redactor_masks_a_secret_in_a_name() {
    let rule = Rule::literal("メールアドレス", "test@example.test");
    let redactor = Redactor::new(&[rule]).unwrap();
    let got = told(&redactor, &[past("click", Some("test@example.test"), true)]);
    let want = format!(
        "{}\n\n{HEAD}\n1. click 「[[メールアドレス]]」 → 画面が変わった",
        cart()
    );
    assert_eq!(got, want);
}

#[test]
fn a_control_character_in_a_name_is_escaped() {
    let got = shown(&[past("click", Some("検\n索\"\u{7}"), true)]);
    let want = format!(
        "{}\n\n{HEAD}\n1. click 「検\\n索\\\"\\u{{0007}}」 → 画面が変わった",
        cart()
    );
    assert_eq!(got, want);
}
