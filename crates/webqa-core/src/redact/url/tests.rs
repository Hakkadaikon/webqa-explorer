//! Tests that no secret shows in a masked URL, however deep it is encoded.

use percent_encoding::percent_decode_str;
use url::Url;

use crate::redact::{Redactor, Rule};
use crate::screen::Screen;

/// The path, query and fragment of `url` after masking, as encoded.
pub type Parts = (String, Option<String>, Option<String>);

pub fn parts(redactor: &Redactor, url: &str) -> Parts {
    let s = redactor.screen(Screen {
        url: Url::parse(&format!("https://app.example.test{url}")).unwrap(),
        elements: vec![],
    });
    let u = &s.as_inner().url;
    (
        u.path().to_owned(),
        u.query().map(str::to_owned),
        u.fragment().map(str::to_owned),
    )
}

/// `encoded` itself and decoded one to three times, with `+` read as itself
/// or as a space.
fn readings(encoded: &str) -> Vec<String> {
    [false, true]
        .into_iter()
        .flat_map(|plus| {
            (0..3).fold(vec![encoded.to_owned()], |mut layers, _| {
                layers.push(decoded(layers.last().unwrap(), plus));
                layers
            })
        })
        .collect()
}

/// `encoded` decoded once, written independently of the code under test.
fn decoded(encoded: &str, plus: bool) -> String {
    let read = if plus {
        encoded.replace('+', " ")
    } else {
        encoded.to_owned()
    };
    percent_decode_str(&read).decode_utf8_lossy().into_owned()
}

/// Fails when a secret shows in any part, in any reading, at any depth.
pub fn assert_hidden(got: &Parts, secrets: &[&str], what: &str) {
    let all = [Some(got.0.clone()), got.1.clone(), got.2.clone()];
    for layer in all.iter().flatten().flat_map(|part| readings(part)) {
        for secret in secrets {
            assert!(!layer.contains(secret), "{what}: {secret} in {layer}");
        }
    }
}

/// A URL with its masked parts and the secrets that must not show.
type Case = (
    &'static str,
    (&'static str, Option<&'static str>, Option<&'static str>),
);

fn assert_cases(redactor: &Redactor, cases: &[Case], secrets: &[&str]) {
    for &(url, (path, query, fragment)) in cases {
        let got = parts(redactor, url);
        let want = (
            path.to_owned(),
            query.map(str::to_owned),
            fragment.map(str::to_owned),
        );
        assert_eq!(got, want, "{url}");
        assert_hidden(&got, secrets, url);
    }
}

const PASSWORDS: [Case; 4] = [
    ("/s?password=hun%2574er2", ("/s", Some("[[pw]]"), None)),
    ("/s?password=hun%252574er2", ("/s", Some("[[pw]]"), None)),
    ("/s?password%3Dhun%2574er2", ("/s", Some("[[pw]]"), None)),
    ("/k/x#password=hun%2574er2", ("/k/x", None, Some("[[pw]]"))),
];

#[test]
fn match_cut_short_by_an_escape_is_masked_whole() {
    let r = Redactor::new(&[Rule::pattern("pw", r"password=\w+")]).unwrap();
    assert_cases(&r, &PASSWORDS, &["hunter2", "ter2"]);
}

const MIXED_DEPTHS: [Case; 3] = [
    (
        "/k/a%252574b,taro%2540example.test",
        ("/k/[[A]]%2C[[mail]]", None, None),
    ),
    (
        "/k#taro%2540example.test,a%252574b",
        ("/k", None, Some("[[mail]]%2C[[A]]")),
    ),
    (
        "/s?v=a%252574b,taro%2540example.test",
        ("/s", Some("v=[[A]]%2C[[mail]]"), None),
    ),
];

#[test]
fn secrets_at_different_depths_are_all_masked() {
    let r = Redactor::new(&[
        Rule::literal("mail", "taro@example.test"),
        Rule::literal("A", "atb"),
    ])
    .unwrap();
    assert_cases(&r, &MIXED_DEPTHS, &["taro@example.test", "atb"]);
}

#[test]
fn secret_across_pieces_is_masked() {
    let key = Redactor::new(&[Rule::literal("key", "a/b/c")]).unwrap();
    assert_cases(
        &key,
        &[("/k/a%252Fb/c", ("/k/[[key]]", None, None))],
        &["a/b/c"],
    );
    let val = Redactor::new(&[Rule::literal("val", "ab=cd@x")]).unwrap();
    assert_cases(
        &val,
        &[("/s?x=ab=cd%2540x", ("/s", Some("x=[[val]]"), None))],
        &["ab=cd@x"],
    );
}

#[test]
fn spans_are_written_only_when_something_matched() {
    let r = Redactor::new(&[Rule::literal("mail", "taro@example.test")]).unwrap();
    assert_eq!(r.mask_spans("q=a%26b", false), None, "nothing matched");
    assert_eq!(
        r.mask_spans("m=taro%40example.test&q=a%26b", false),
        Some("m=[[mail]]&q=a%26b".to_owned()),
        "a match"
    );
}

#[test]
fn masking_that_never_settles_empties_the_part() {
    let r = Redactor::new(&[Rule::pattern("x", r"\[\[")]).unwrap();
    assert_eq!(
        parts(&r, "/a[[b?q=1"),
        ("/".to_owned(), Some("q=1".to_owned()), None)
    );
}
