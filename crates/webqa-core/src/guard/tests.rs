//! Tests of the folding, words and path segments shared by the guards.

use url::Url;

use super::*;

fn owned(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

#[test]
fn words_match_at_ascii_boundaries() {
    let words = Words::new(&[], &owned(&["pay", "購入"]));
    let cases = [
        ("Pay now", true),
        ("PAY", true),
        ("Display", false),
        ("Prepay", false),
        ("payment", false),
        ("今すぐ購入", true),
        ("ｐａｙ", true),
        ("", false),
    ];
    for (text, want) in cases {
        assert_eq!(words.any_in(text), want, "{text}");
    }
}

#[test]
fn empty_keyword_is_ignored() {
    let words = Words::new(&[], &owned(&["", "  ", "　"]));
    assert!(!words.any_in("保存"), "an empty word matches nothing");
    assert!(
        !words.any_in(""),
        "an empty word does not match an empty text"
    );
    let segments = Segments::new(&[], &owned(&["", "-"]));
    let root = Url::parse("https://app.example.test/").unwrap();
    assert!(!segments.any_in(&root), "an empty segment matches nothing");
}

#[test]
fn segments_ignore_case_hyphen_and_query() {
    let segments = Segments::new(&[], &owned(&["signin", "pricing"]));
    let cases = [
        ("https://app.example.test/sign-in", true),
        ("https://app.example.test/Sign_In/", true),
        ("https://app.example.test/en/pricing", true),
        ("https://app.example.test/pricing-history", false),
        ("https://app.example.test/?next=/pricing", false),
        ("https://app.example.test/#/pricing", false),
        ("data:text/html,pricing", false),
    ];
    for (url, want) in cases {
        assert_eq!(segments.any_in(&Url::parse(url).unwrap()), want, "{url}");
    }
}
