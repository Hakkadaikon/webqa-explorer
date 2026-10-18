//! Tests of stages and `done_when`.

use super::*;
use crate::screen::Rect;

mod blank;

const ORIGIN: &str = "https://app.example.test";

const BOX: Option<Rect> = Some(Rect {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
});

fn at(path: &str) -> url::Url {
    url::Url::parse(&format!("{ORIGIN}{path}")).unwrap()
}

fn el(role: &str, name: &str) -> Element {
    boxed(role, name, BOX)
}

fn boxed(role: &str, name: &str, rect: Option<Rect>) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect,
    }
}

fn wanted(role: &str, name: &str) -> Wanted {
    Wanted {
        role: role.to_owned(),
        name: name.to_owned(),
    }
}

fn url_is(path: &str) -> DoneWhen {
    when(Some(path), None, None)
}

fn text_is(text: &str) -> DoneWhen {
    when(None, None, Some(text))
}

fn element_is(role: &str, name: &str) -> DoneWhen {
    when(None, Some(wanted(role, name)), None)
}

fn when(url: Option<&str>, element: Option<Wanted>, text: Option<&str>) -> DoneWhen {
    DoneWhen {
        url: url.map(str::to_owned),
        element,
        text: text.map(str::to_owned),
    }
}

fn stage(name: &str, done_when: DoneWhen) -> Stage {
    Stage {
        name: name.to_owned(),
        goal: format!("{name}する"),
        done_when,
    }
}

/// "ログイン" then "投稿".
fn two_stages(login: DoneWhen) -> Vec<Stage> {
    vec![
        stage("ログイン", login),
        stage("投稿", text_is("投稿しました")),
    ]
}

/// `met` and `goal` of the first stage of `stages`, for a page.
fn check(stages: &[Stage], path: &str, elements: &[Element], text: &str) -> (bool, Goal) {
    let url = at(path);
    let page = Page {
        url: &url,
        elements,
        text,
    };
    (met(&stages[0].done_when, page), goal(stages, 0, page))
}

/// `met` alone, for a page.
fn holds(done_when: DoneWhen, path: &str, elements: &[Element], text: &str) -> bool {
    check(&[stage("ログイン", done_when)], path, elements, text).0
}

fn goal_at(stages: &[Stage], current: usize, path: &str, text: &str) -> Goal {
    let url = at(path);
    let page = Page {
        url: &url,
        elements: &[],
        text,
    };
    goal(stages, current, page)
}

#[test]
fn later_stage_done_when_does_not_skip() {
    let stages = two_stages(url_is("/home"));
    assert_eq!(
        goal_at(&stages, 0, "/login", "投稿しました"),
        Goal::Unmet,
        "a later stage's done_when must not end the current stage"
    );
    assert_eq!(
        goal_at(&stages, 1, "/login", "投稿しました"),
        Goal::Achieved,
        "the page meets the done_when of the last stage"
    );
}

#[test]
fn done_when_requires_all_declarations() {
    let stages = two_stages(when(
        Some("/home"),
        Some(wanted("heading", "ようこそ")),
        None,
    ));
    let welcome = [el("heading", "ようこそ")];
    let cases: [(&str, &[Element], bool, Goal); 4] = [
        ("/home", &welcome, true, Goal::Advance),
        ("/home", &[], false, Goal::Unmet),
        ("/login", &welcome, false, Goal::Unmet),
        ("/login", &[], false, Goal::Unmet),
    ];
    for (path, elements, ok, want) in cases {
        let got = check(&stages, path, elements, "");
        assert_eq!(got, (ok, want), "path {path}, {} elements", elements.len());
    }
}

#[test]
fn done_when_uses_whole_page() {
    let stages = two_stages(element_is("heading", "ようこそ"));
    let below = Some(Rect {
        x: 0,
        y: 1200,
        width: 100,
        height: 20,
    });
    for rect in [below, None] {
        let elements = [boxed("heading", "ようこそ", rect)];
        assert_eq!(
            check(&stages, "/", &elements, ""),
            (true, Goal::Advance),
            "an element off the viewport counts: {rect:?}"
        );
    }
    let long = format!("{} 利用規約", "長い本文。".repeat(500));
    assert_eq!(
        check(&two_stages(text_is("利用規約")), "/", &[], &long),
        (true, Goal::Advance),
        "text at the end of a long page counts"
    );
}

#[test]
fn url_declaration_compares_path_only() {
    let cases = [
        ("/home?tab=1#top", "/home", true),
        ("/home", "/home?x=1", true),
        ("/home", "/home#x", true),
        ("/検索", "/検索", true),
        ("/home", "/home/", false),
        ("/home/", "/home", false),
        ("/home", "/Home", false),
        ("/home", "home", false),
        ("/", "//app.example.test/", false),
        ("/home", "", false),
    ];
    for (path, declared, want) in cases {
        assert_eq!(
            holds(url_is(declared), path, &[], ""),
            want,
            "page {path}, declared {declared:?}"
        );
    }
}

#[test]
fn element_and_text_need_exact_values() {
    let cases = [
        (vec![el("button", "ようこそ")], false),
        (vec![el("heading", "ようこそ！")], false),
        (
            vec![el("button", "ようこそ"), el("heading", "ようこそ")],
            true,
        ),
    ];
    for (elements, want) in &cases {
        assert_eq!(
            holds(element_is("heading", "ようこそ"), "/", elements, ""),
            *want,
            "{} elements",
            elements.len()
        );
    }
    let texts = [
        ("ようこそ", "ようこそ、太郎さん", true),
        ("ようこそ", "よう こそ", false),
        ("Welcome", "welcome", false),
    ];
    for (declared, text, want) in texts {
        assert_eq!(
            holds(text_is(declared), "/", &[], text),
            want,
            "declared {declared:?}, page text {text:?}"
        );
    }
}

#[test]
fn goal_outside_stages_is_unmet() {
    let stages = two_stages(url_is("/home"));
    assert_eq!(
        goal_at(&stages, 2, "/home", "投稿しました"),
        Goal::Unmet,
        "no stage after the last"
    );
    assert_eq!(
        goal_at(&[], 0, "/home", "投稿しました"),
        Goal::Unmet,
        "no stage at all"
    );
}
