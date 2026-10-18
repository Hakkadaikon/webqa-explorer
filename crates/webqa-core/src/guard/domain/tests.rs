//! Tests of the allowed origin guard.

use url::Url;

use super::*;

fn origins(list: &[&str]) -> Vec<AllowedOrigin> {
    list.iter()
        .map(|s| AllowedOrigin::parse(s).unwrap())
        .collect()
}

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

fn assert_allowed(allowed: &[AllowedOrigin], cases: &[(&str, bool)]) {
    for (s, want) in cases {
        assert_eq!(is_allowed(&url(s), allowed), *want, "{s}");
    }
}

#[test]
fn allowed_origin_rejects_more_than_origin() {
    let bad = [
        "",
        "not an origin",
        "https://a.*.test",
        "https://*",
        "https://*.",
        "https://app.example.test/p?q",
        "https://app.example.test/path",
        "https://app.example.test?q",
        "https://app.example.test#f",
        "https://user@app.example.test",
        "https://user:pw@app.example.test",
        "javascript:alert(1)",
        "data:text/html,hello",
    ];
    for s in bad {
        assert_eq!(AllowedOrigin::parse(s), Err(InvalidOrigin), "{s}");
    }
    let good = [
        "https://app.example.test",
        "https://app.example.test/",
        "https://*.app.example.test",
        "http://localhost:8080",
    ];
    for s in good {
        assert_eq!(AllowedOrigin::parse(s).map(drop), Ok(()), "{s}");
    }
    assert_eq!(
        InvalidOrigin.to_string(),
        "許可ドメインはスキーム、ホスト、ポートだけのオリジンで書く",
        "the message has no value in it"
    );
}

#[test]
fn origin_must_match_exactly() {
    let exact = origins(&["https://app.example.test"]);
    assert_allowed(
        &exact,
        &[
            ("https://app.example.test/settings", true),
            ("https://app.example.test:443/", true),
            ("https://APP.example.test/", true),
            ("https://app.example.test.evil.test/", false),
            ("https://app.examp1e.test/", false),
            ("http://app.example.test/", false),
            ("http://app.example.test:443/", false),
            ("https://app.example.test:8443/", false),
            ("javascript:alert(1)", false),
            ("data:text/html,hello", false),
            ("https://sub.app.example.test/", false),
        ],
    );
    let both = origins(&["https://app.example.test", "https://*.app.example.test"]);
    assert_allowed(
        &both,
        &[
            ("https://sub.app.example.test/", true),
            ("https://a.b.app.example.test/", true),
            ("https://app.example.test/", true),
            ("http://sub.app.example.test/", false),
            ("https://sub.app.example.test:8443/", false),
            ("https://subapp.example.test/", false),
        ],
    );
    let wildcard = origins(&["https://*.app.example.test"]);
    assert_allowed(&wildcard, &[("https://app.example.test/", false)]);
    assert_allowed(&[], &[("https://app.example.test/", false)]);
}

#[test]
fn judges_final_url() {
    let allowed = origins(&["https://app.example.test"]);
    let cases = [
        (
            "https://app.example.test/go",
            "https://other.example.net/",
            true,
        ),
        (
            "https://other.example.net/r",
            "https://app.example.test/home",
            false,
        ),
        (
            "https://app.example.test/home",
            "https://app.example.test/home",
            false,
        ),
    ];
    for (requested, landed, want) in cases {
        let nav = Navigation {
            requested: url(requested),
            landed: url(landed),
        };
        assert_eq!(left_allowed(&nav, &allowed), want, "{requested} {landed}");
    }
}

#[test]
fn empty_list_allows_only_start_origin() {
    let allowed = allowed_or_start(Vec::new(), &url("https://app.example.test/login?next=/"));
    assert_allowed(
        &allowed,
        &[
            ("https://app.example.test/home", true),
            ("https://app.example.test:443/", true),
            ("https://other.example.net/", false),
            ("https://sub.app.example.test/", false),
            ("http://app.example.test/", false),
            ("https://app.example.test:8443/", false),
        ],
    );
    let listed = allowed_or_start(
        origins(&["https://app.example.test"]),
        &url("https://other.example.net/"),
    );
    assert_allowed(
        &listed,
        &[
            ("https://app.example.test/", true),
            ("https://other.example.net/", false),
        ],
    );
    let no_host = allowed_or_start(Vec::new(), &url("data:text/html,hello"));
    assert_eq!(no_host, Vec::new(), "a start without a host allows nothing");
    let no_port = allowed_or_start(Vec::new(), &url("foo://app.example.test/"));
    assert_eq!(
        no_port,
        Vec::new(),
        "a start without a known port allows nothing"
    );
}
