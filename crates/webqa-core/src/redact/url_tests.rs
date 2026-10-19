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
    let email = redactor(Rule::pattern("メールアドレス", EMAIL));
    assert_eq!(
        query(&email, "https://app.example.test/s?m=taro+tag@example.test"),
        Some("m=[[メールアドレス]]".to_owned()),
        "the reading with + kept comes first"
    );
}

#[test]
fn both_readings_of_query_are_masked() {
    let r = Redactor::new(&[
        Rule::literal("氏名", "taro yamada"),
        Rule::literal("m", "x+y@e.test"),
    ])
    .unwrap();
    assert_eq!(
        query(
            &r,
            "https://app.example.test/s?name=taro+yamada&m=x+y@e.test"
        ),
        Some("name=[[氏名]]&m=[[m]]".to_owned())
    );
}

/// Masks `taro@example.test` as `[[mail]]`.
fn mail() -> Redactor {
    redactor(Rule::literal("mail", "taro@example.test"))
}

/// The path and query of the masked screen, as encoded.
fn encoded(redactor: &Redactor, url: &str) -> (String, Option<String>) {
    let s = screen(redactor, url);
    let url = &s.as_inner().url;
    (url.path().to_owned(), url.query().map(str::to_owned))
}

/// The query of a URL, then what masking leaves of it.
const TWICE_ENCODED: [(&str, &str); 3] = [
    (
        "next=https%3A%2F%2Fx.test%2F%3Fq%3Dtaro%2540example.test",
        "next=https%3A%2F%2Fx.test%2F%3Fq%3D[[mail]]",
    ),
    ("m=taro%252540example.test", "m=[[mail]]"),
    ("m=taro%25252540example.test", "m=taro%25252540example.test"),
];

#[test]
fn secret_encoded_twice_is_masked() {
    let r = mail();
    for (query, want) in TWICE_ENCODED {
        let url = format!("https://app.example.test/s?{query}");
        assert_eq!(encoded(&r, &url).1, Some(want.to_owned()), "{query}");
    }
    assert_eq!(
        encoded(&r, "https://app.example.test/a/taro%2540example.test").0,
        "/a/[[mail]]",
        "path"
    );
}

/// A URL, then the path and query masking leaves.
const KEPT: [(&str, &str, Option<&str>); 5] = [
    (
        "/s?q=a%26b&m=taro@example.test",
        "/s",
        Some("q=a%26b&m=[[mail]]"),
    ),
    (
        "/s?q=a%2Bb&m=taro@example.test",
        "/s",
        Some("q=a%2Bb&m=[[mail]]"),
    ),
    ("/a%2Fb/taro@example.test", "/a%2Fb/[[mail]]", None),
    (
        "/s?q=%2561&m=taro%40example.test",
        "/s",
        Some("q=%2561&m=[[mail]]"),
    ),
    ("/%FF/taro@example.test/%ff", "/%FF/[[mail]]/%ff", None),
];

#[test]
fn encoding_outside_secrets_is_kept() {
    let r = mail();
    for (url, path, query) in KEPT {
        let got = encoded(&r, &format!("https://app.example.test{url}"));
        assert_eq!(got, (path.to_owned(), query.map(str::to_owned)), "{url}");
    }
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
    let url = "https://app.example.test/a%20b/%E3%81%82/%e3%81%82/c%2Fd?x=1&y=a+b#top";
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

#[test]
fn url_string_drops_userinfo() {
    let u = Url::parse("https://taro:secret@app.example.test/p?q=1#f").unwrap();
    let want = "https://app.example.test/p?q=1#f";
    assert_eq!(mail().url(&u).into_inner(), want, "with rules");
    assert_eq!(Redactor::default().url(&u).into_inner(), want, "no rules");
    let user = Url::parse("https://taro@app.example.test/p").unwrap();
    assert_eq!(
        mail().url(&user).into_inner(),
        "https://app.example.test/p",
        "user only"
    );
}
