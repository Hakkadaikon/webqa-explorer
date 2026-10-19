//! Tests of the memory file: its JSON, reading it back, and the files that
//! cannot be used.

use serde_json::{Value, json};

use super::super::tests::{button, forbid, page, shown};
use super::*;
use crate::explore::graph::{ElementOp, ScreenKey};

const APP: &str = "https://app.example.test";

/// `/` with "保存" and "ヘルプ", `/settings` with "通知" and the forbidden
/// "退会", and the click on "保存" that moved from the first to the second.
fn two_screens() -> Memory {
    let mut memory = Memory::default();
    let top = page("/", vec![shown("button", "保存"), shown("link", "ヘルプ")]);
    let settings = page(
        "/settings",
        vec![shown("button", "通知"), shown("button", "退会")],
    );
    let s1 = memory.visit(&top, &forbid(&[]));
    let s2 = memory.visit(&settings, &forbid(&[1]));
    memory.record(s1, &button("保存"), ElementOp::Click, s2);
    memory
}

fn element(role: &str, name: &str, operated: bool, forbidden: bool) -> Value {
    json!({"role": role, "name": name, "nth": 0, "operated": operated, "forbidden": forbidden})
}

/// The JSON of [`two_screens`].
fn documented() -> Value {
    json!({
        "schema": 1,
        "origin": APP,
        "screens": {
            "s1": {"path": "/", "elements": [
                element("button", "保存", true, false),
                element("link", "ヘルプ", false, false)]},
            "s2": {"path": "/settings", "elements": [
                element("button", "通知", false, false),
                element("button", "退会", false, true)]}
        },
        "edges": [{"from": "s1", "element": {"role": "button", "name": "保存", "nth": 0},
                   "op": "click", "to": "s2"}]
    })
}

#[test]
fn memory_json_matches_documented_shape() {
    assert_eq!(
        two_screens().to_json(APP),
        documented(),
        "the documented shape"
    );
}

/// `memory` written and read back for the same origin.
fn reread(memory: &Memory) -> Memory {
    let text = memory.to_json(APP).to_string();
    Memory::from_json(&text, APP).unwrap()
}

#[test]
fn memory_json_round_trips() {
    let memory = two_screens();
    let back = reread(&memory);
    assert_eq!(back.to_json(APP), memory.to_json(APP), "the same JSON");
    assert_eq!(
        (back.screen_count(), back.edge_count()),
        (2, 1),
        "the counts"
    );
    let (s1, save) = (ScreenKey(0), button("保存"));
    let read = (
        back.is_operated(s1, &save),
        back.finished(s1, &save),
        back.untouched_reachable(s1),
    );
    assert_eq!(
        read,
        (true, false, true),
        "通知 is left on s2, 退会 is forbidden"
    );
    assert_eq!(reread(&Memory::default()).screen_count(), 0, "empty");
    many_screens_read_back();
}

/// Eleven screens, so that `s10` sorts before `s2` as text.
fn many_screens_read_back() {
    let mut many = Memory::default();
    for n in 0..=10 {
        many.visit(
            &page(&format!("/p{n}"), vec![shown("button", "次")]),
            &forbid(&[]),
        );
    }
    assert_eq!(
        reread(&many).to_json(APP),
        many.to_json(APP),
        "s10 after s9"
    );
    assert_eq!(reread(&many).screen_count(), 11, "eleven screens");
}

/// The JSON of [`two_screens`] with `change` applied, as text.
fn changed(change: fn(&mut Value)) -> String {
    let mut value = documented();
    change(&mut value);
    value.to_string()
}

fn renamed(value: &mut Value, from: &str, to: &str) {
    let screens = value["screens"].as_object_mut().unwrap();
    let screen = screens.remove(from).unwrap();
    screens.insert(to.to_owned(), screen);
}

fn next_schema(value: &mut Value) {
    value["schema"] = json!(2);
}

fn other_origin(value: &mut Value) {
    value["origin"] = json!("https://other.example.test");
}

fn skipped_number(value: &mut Value) {
    renamed(value, "s2", "s3");
}

fn not_named_s(value: &mut Value) {
    renamed(value, "s2", "x2");
}

fn edge_to_nowhere(value: &mut Value) {
    value["edges"][0]["to"] = json!("s9");
}

fn edge_of_scrolling(value: &mut Value) {
    value["edges"][0]["op"] = json!("scroll_down");
}

fn without_forbidden(value: &mut Value) {
    let first = &mut value["screens"]["s1"]["elements"][0];
    first.as_object_mut().unwrap().remove("forbidden");
}

fn unusable_files() -> [(String, MemoryFileError); 10] {
    use MemoryFileError::{Origin, Schema, Unreadable};
    let no_schema = json!({"origin": APP, "screens": {}, "edges": []}).to_string();
    [
        (String::new(), Unreadable),
        ("{".to_owned(), Unreadable),
        ("[]".to_owned(), Unreadable),
        (no_schema, Unreadable),
        (changed(next_schema), Schema),
        (changed(other_origin), Origin),
        (changed(skipped_number), Unreadable),
        (changed(not_named_s), Unreadable),
        (changed(edge_to_nowhere), Unreadable),
        (changed(edge_of_scrolling), Unreadable),
    ]
}

#[test]
fn unusable_memory_files_are_rejected() {
    for (text, error) in unusable_files() {
        let read = Memory::from_json(&text, APP).map(|m| m.screen_count());
        assert_eq!(read, Err(error), "{text}");
    }
    let read = Memory::from_json(&changed(without_forbidden), APP).unwrap();
    assert_eq!(
        read.to_json(APP),
        documented(),
        "forbidden is false when left out"
    );
    let messages = [
        MemoryFileError::Unreadable,
        MemoryFileError::Schema,
        MemoryFileError::Origin,
    ]
    .map(|e| e.to_string());
    assert_eq!(
        messages,
        [
            "探索の記憶を読めない",
            "探索の記憶の形式の版が違う",
            "探索の記憶が別の対象のもの"
        ],
        "fixed messages"
    );
}
