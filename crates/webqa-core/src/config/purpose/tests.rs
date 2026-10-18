//! Tests of reading the purpose document.

mod cases;

use url::Url;

use super::*;
use crate::config::interview::tests::{config_with, load_with};
use crate::config::persona::tests::PERSONA;
use crate::config::raw::tests::changed;
use cases::{BAD_PATTERNS, GOOD_PATTERNS, MATCHES, WITHOUT_DECLARATIONS};

/// A valid purpose document.
pub const PURPOSE: &str = "goal: 注文を確定できる画面まで進む
done_when:
  element: { role: button, name: 注文を確定する }
screens:
  - url: /products/*
    purpose: 商品をカートに入れる
";

fn element(role: &str, name: &str) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
    }
}

fn confirm_button() -> DoneWhen {
    DoneWhen {
        url: None,
        element: Some(element("button", "注文を確定する")),
        text: None,
    }
}

/// The error message of reading `text`.
fn rejected(text: &str) -> String {
    parse(text).unwrap_err().to_string()
}

#[test]
fn written_purpose_becomes_single_stage() {
    let purpose = parse(PURPOSE).unwrap();
    let written = Written {
        goal: "注文を確定できる画面まで進む".to_owned(),
        done_when: confirm_button(),
    };
    assert_eq!(purpose.aim, Aim::Written(written), "aim");
    assert_eq!(purpose.screens.len(), 1, "one screen purpose");
    let stage = Stage {
        name: "目的".to_owned(),
        goal: "注文を確定できる画面まで進む".to_owned(),
        done_when: confirm_button(),
    };
    assert_eq!(purpose.stage(), stage, "stage");
    let three = PURPOSE.replacen(
        "done_when:\n",
        "done_when:\n  url: /cart/confirm\n  text: ご注文内容の確認\n",
        1,
    );
    let all = DoneWhen {
        url: Some("/cart/confirm".to_owned()),
        element: Some(element("button", "注文を確定する")),
        text: Some("ご注文内容の確認".to_owned()),
    };
    let done_when = parse(&three).unwrap().stage().done_when;
    assert_eq!(done_when, all, "three declarations");
}

#[test]
fn inferred_purpose_becomes_unmet_stage() {
    let inferred = Purpose::inferred();
    let want = Purpose {
        aim: Aim::Inferred,
        screens: Vec::new(),
    };
    assert_eq!(inferred, want, "inferred purpose");
    let stage = Stage {
        name: "目的".to_owned(),
        goal: INFERRED_GOAL.to_owned(),
        done_when: DoneWhen {
            url: None,
            element: Some(element("", "")),
            text: None,
        },
    };
    assert_eq!(inferred.stage(), stage, "unmet stage");
    let screens = parse("screens:\n  - url: /products/*\n    purpose: 商品を選ぶ\n").unwrap();
    assert_eq!(screens.aim, Aim::Inferred, "screens only");
    assert_eq!(screens.stage(), stage, "screens only stage");
}

#[test]
fn done_when_without_declarations_rejected() {
    for (text, want) in WITHOUT_DECLARATIONS {
        assert_eq!(rejected(text), want, "{text:?}");
    }
    let empty = rejected("screens: []\n");
    assert_eq!(
        empty, "purpose に done_when か screens を書く",
        "no screens"
    );
    let unknown = rejected("goal: 注文する\ndone_when: { button: 注文 }\n");
    assert!(
        unknown.starts_with("purpose を決まった形式で読めない: "),
        "unknown declaration: {unknown}"
    );
}

/// [`PURPOSE`] with screen purposes for `urls`, all with the same text.
fn with_screens(urls: &[&str]) -> String {
    let screens = urls
        .iter()
        .map(|url| format!("  - url: \"{url}\"\n    purpose: 見る\n"))
        .collect::<Vec<_>>()
        .concat();
    let head = PURPOSE.split("screens:").next().unwrap();
    format!("{head}screens:\n{screens}")
}

#[test]
fn malformed_url_pattern_rejected() {
    let first = "purpose.screens の 1 番目の url の型が書式に合わない";
    for url in BAD_PATTERNS {
        assert_eq!(rejected(&with_screens(&[url])), first, "{url:?}");
    }
    let second = rejected(&with_screens(&["/products/*", "/products/["]));
    assert_eq!(
        second, "purpose.screens の 2 番目の url の型が書式に合わない",
        "second"
    );
    assert!(!second.contains("/products/["), "no pattern in {second}");
    accepts_good_patterns();
}

fn accepts_good_patterns() {
    for url in GOOD_PATTERNS {
        let screens = parse(&with_screens(&[url])).unwrap().screens;
        let written: Vec<_> = screens.iter().map(|s| s.pattern.as_str()).collect();
        assert_eq!(written, [url], "{url:?}");
    }
}

#[test]
fn screen_purpose_items_checked() {
    let blank = format!("{}  - url: /b\n    purpose: \" \"\n", with_screens(&["/a"]));
    assert_eq!(
        rejected(&blank),
        "purpose.screens の 2 番目の purpose を書く",
        "blank purpose"
    );
    let duplicate = "purpose.screens の 2 番目の url の型が、前の項目と同じ";
    for pair in [
        ["/products/*", "/products/*"],
        ["/検索", "/%E6%A4%9C%E7%B4%A2"],
    ] {
        assert_eq!(rejected(&with_screens(&pair)), duplicate, "{pair:?}");
    }
    let distinct = parse(&with_screens(&["/products/*", "/products/"])).unwrap();
    assert_eq!(distinct.screens.len(), 2, "trailing slash differs");
}

/// Whether `pattern` matches the path `path` of the test origin.
fn matches(pattern: &str, path: &str) -> bool {
    let url = Url::parse(&format!("https://shop.example.test{path}")).unwrap();
    UrlPattern::parse(pattern).unwrap().matches(&url)
}

/// Whether `pattern` matches each of `paths`.
fn results(pattern: &str, paths: &[&str]) -> Vec<bool> {
    paths.iter().map(|path| matches(pattern, path)).collect()
}

#[test]
fn url_pattern_matches_paths() {
    for (pattern, hits, misses) in MATCHES {
        let want = vec![true; hits.len()];
        assert_eq!(results(pattern, hits), want, "{pattern} hits");
        let want = vec![false; misses.len()];
        assert_eq!(results(pattern, misses), want, "{pattern} misses");
    }
    let search = Url::parse("https://shop.example.test/検索").unwrap();
    assert_eq!(search.path(), "/%E6%A4%9C%E7%B4%A2", "encoded path");
    let pattern = UrlPattern::parse("/検索").unwrap();
    assert_eq!(pattern.as_str(), "/検索", "as written");
}

#[test]
fn forbidden_confirm_button_counts_as_reached() {
    let base = changed("keywords: [\"購入\"]", "keywords: [\"確定\"]");
    let config = config_with(
        &base,
        "interview:\n  persona: persona.yaml\n  purpose: purpose.yaml\n",
    );
    let interview = load_with(&config, PERSONA, Some(PURPOSE)).unwrap();
    assert_eq!(
        interview.purpose.stage().done_when.element,
        Some(element("button", "注文を確定する")),
        "the forbidden button is the goal"
    );
}
