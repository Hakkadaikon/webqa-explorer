//! Tests of reading what the page script returns.

use serde_json::error::Category;

use super::*;

fn url(s: &str) -> url::Url {
    url::Url::parse(s).unwrap()
}

fn probe(json: &str) -> Probe {
    Probe::from(serde_json::from_str::<RawProbe>(json).unwrap())
}

#[test]
fn raw_probe_reads_script_result() {
    let link = r#"{"hit":true,"href":"https://app.example.test/settings","formAction":null,"forbidden":false,"allowed":true}"#;
    let expected = Probe {
        hit: true,
        href: Some(url("https://app.example.test/settings")),
        form_action: None,
        forbidden_listed: false,
        allowed_listed: true,
    };
    assert_eq!(probe(link), expected, "a link inside the allowed list");
    let submit = r#"{"hit":false,"href":null,"formAction":"https://app.example.test/checkout","forbidden":true,"allowed":false}"#;
    let expected = Probe {
        hit: false,
        href: None,
        form_action: Some(url("https://app.example.test/checkout")),
        forbidden_listed: true,
        allowed_listed: false,
    };
    assert_eq!(probe(submit), expected, "a forbidden submit button");
    for bad in [
        r#"{"href":null,"formAction":null,"forbidden":false,"allowed":false}"#,
        r#"{"hit":true,"href":null,"formAction":null,"forbidden":false,"allowed":false,"extra":1}"#,
    ] {
        let read = serde_json::from_str::<RawProbe>(bad).map_err(|e| e.classify());
        assert_eq!(read, Err(Category::Data), "{bad}");
    }
}

#[test]
fn unreadable_destination_is_none() {
    let raw = |href: &str, action: &str| RawProbe {
        hit: true,
        href: Some(href.to_owned()),
        form_action: Some(action.to_owned()),
        forbidden: false,
        allowed: false,
    };
    let read = Probe::from(raw("", "not a url"));
    assert_eq!((read.href, read.form_action), (None, None), "neither reads");
    let read = Probe::from(raw("javascript:void(0)", "https://app.example.test/a"));
    let schemes = (
        read.href.map(|u| u.scheme().to_owned()),
        read.form_action.map(|u| u.scheme().to_owned()),
    );
    let expected = (Some("javascript".to_owned()), Some("https".to_owned()));
    assert_eq!(schemes, expected, "any scheme reads");
}

#[test]
fn probe_debug_hides_destinations() {
    let raw = RawProbe {
        hit: true,
        href: Some("https://app.example.test/u/token123".to_owned()),
        form_action: None,
        forbidden: false,
        allowed: true,
    };
    let shown = [
        format!("{raw:?}"),
        format!("{:?}", Probe::from(raw.clone())),
    ];
    for text in &shown {
        assert!(!text.contains("token123"), "{text}");
        assert!(text.contains('…'), "{text}");
    }
    let expected = [
        "RawProbe { hit: true, href: Some(\"…\"), form_action: None, forbidden: false, allowed: true }",
        "Probe { hit: true, href: Some(\"…\"), form_action: None, forbidden_listed: false, allowed_listed: true }",
    ];
    assert_eq!(
        shown, expected,
        "the flags and whether there is a destination"
    );
    let none = RawProbe { href: None, ..raw };
    let shown = [format!("{none:?}"), format!("{:?}", Probe::from(none))];
    for text in &shown {
        assert!(!text.contains('…'), "{text}");
    }
    let arg = ProbeArg {
        forbidden: vec!["#danger".into()],
        allowed: vec![],
    };
    assert_eq!(
        serde_json::to_string(&arg).unwrap(),
        r##"{"forbidden":["#danger"],"allowed":[]}"##,
        "the selectors as the script takes them"
    );
}
