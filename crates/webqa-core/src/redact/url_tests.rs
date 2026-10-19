//! Tests that secrets in a URL are matched on its decoded form.

use percent_encoding::percent_decode_str;
use url::Url;

use super::{Redacted, Redactor, Rule};
use crate::screen::{Screen, screen_id};

const EMAIL: &str = r"[\w.+-]+@[\w-]+\.[\w.]+";

fn redactor(rule: Rule) -> Redactor {
    Redactor::new(&[rule]).unwrap()
}

/// The screen of `url` with no elements, through `redactor`.
fn screen(redactor: &Redactor, url: &str) -> Redacted<Screen> {
    redactor.screen(Screen {
        url: Url::parse(url).unwrap(),
        elements: vec![],
    })
}

fn decoded(s: &str) -> String {
    percent_decode_str(s).decode_utf8_lossy().into_owned()
}

/// The path of the masked screen, decoded.
fn path(redactor: &Redactor, url: &str) -> String {
    decoded(screen(redactor, url).as_inner().url.path())
}

/// The query of the masked screen, decoded.
fn query(redactor: &Redactor, url: &str) -> Option<String> {
    screen(redactor, url).as_inner().url.query().map(decoded)
}

/// The fragment of the masked screen, decoded.
fn fragment(redactor: &Redactor, url: &str) -> Option<String> {
    screen(redactor, url).as_inner().url.fragment().map(decoded)
}

#[test]
fn encoded_secret_in_path_is_masked() {
    let r = redactor(Rule::literal("氏名", "山田 太郎"));
    let s = screen(
        &r,
        "https://app.example.test/users/%E5%B1%B1%E7%94%B0%20%E5%A4%AA%E9%83%8E/edit",
    );
    let url = &s.as_inner().url;
    assert_eq!(decoded(url.path()), "/users/[[氏名]]/edit", "path");
    assert!(!url.as_str().contains("%E5%B1%B1"), "{url}");
    assert_eq!(
        decoded(&screen_id(&s).path),
        "/users/[[氏名]]/edit",
        "screen id"
    );
}

#[test]
fn secret_with_slash_is_masked() {
    let r = redactor(Rule::literal("鍵", "ab/cd"));
    assert_eq!(
        path(&r, "https://app.example.test/k/ab/cd/x"),
        "/k/[[鍵]]/x"
    );
}

#[test]
fn encoded_query_is_masked() {
    let r = redactor(Rule::pattern("メールアドレス", EMAIL));
    assert_eq!(
        query(&r, "https://app.example.test/s?q=taro%40example.test"),
        Some("q=[[メールアドレス]]".to_owned())
    );
}

#[test]
fn plus_as_space_in_query_is_masked() {
    let r = redactor(Rule::literal("氏名", "taro yamada"));
    let want = Some("name=[[氏名]]".to_owned());
    for url in [
        "https://app.example.test/s?name=taro+yamada",
        "https://app.example.test/s?name=taro%20yamada",
    ] {
        assert_eq!(query(&r, url), want, "{url}");
    }
}

#[test]
fn plus_in_secret_is_kept() {
    let r = redactor(Rule::literal("メール", "taro+tag@example.test"));
    assert_eq!(
        query(&r, "https://app.example.test/s?m=taro+tag@example.test"),
        Some("m=[[メール]]".to_owned())
    );
}

#[test]
fn fragment_is_masked() {
    let r = redactor(Rule::literal("トークン", "abc123"));
    assert_eq!(
        fragment(&r, "https://app.example.test/cb#access_token=abc123"),
        Some("access_token=[[トークン]]".to_owned()),
        "fragment"
    );
    assert_eq!(fragment(&r, "https://app.example.test/cb"), None, "none");
}

#[test]
fn unmatched_url_is_unchanged() {
    let r = redactor(Rule::pattern("メールアドレス", EMAIL));
    let url = "https://app.example.test/a%20b/%E3%81%82?x=1&y=a+b#top";
    assert_eq!(screen(&r, url).as_inner().url.as_str(), url);
}

#[test]
fn url_string_is_redacted() {
    let r = redactor(Rule::literal("トークン", "abc123"));
    let u = Url::parse("https://app.example.test/p/abc123?t=abc123#abc123").unwrap();
    let got = r.url(&u).into_inner();
    assert!(!got.contains("abc123"), "{got}");
    let want =
        Url::parse("https://app.example.test/p/[[トークン]]?t=[[トークン]]#[[トークン]]").unwrap();
    assert_eq!(got, want.as_str(), "masked");
    assert_eq!(
        Redactor::default().url(&u).into_inner(),
        u.as_str(),
        "no rules"
    );
}
