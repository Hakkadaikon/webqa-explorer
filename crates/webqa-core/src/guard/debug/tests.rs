//! Tests that page strings stay out of the guards' `Debug`.

use url::Url;

use crate::guard::forbidden::Candidate;
use crate::guard::identity::{Node, Target};
use crate::guard::paywall::PaywallFacts;

const SECRET_URL: &str = "https://app.example.test/u/token123";

fn assert_hidden(debug: &str) {
    for secret in ["山田太郎", "token123"] {
        assert!(!debug.contains(secret), "{secret} in {debug}");
    }
}

#[test]
fn debug_hides_page_strings() {
    let candidate = Candidate {
        role: "link".to_owned(),
        name: "山田太郎の注文".to_owned(),
        href: Some(Url::parse(SECRET_URL).unwrap()),
        form_action: Some(Url::parse(SECRET_URL).unwrap()),
        forbidden_listed: false,
        allowed_listed: true,
    };
    let facts = PaywallFacts {
        url: Url::parse(SECRET_URL).unwrap(),
        headings: vec!["山田太郎".to_owned()],
        dialogs: vec!["山田太郎".to_owned(), "token123".to_owned()],
        buttons: Vec::new(),
        hits: vec![true],
    };
    let nodes = [Node {
        role: "button".to_owned(),
        name: "山田太郎".to_owned(),
        node: "e1".to_owned(),
    }];
    let target = Target::at(&nodes, 0).unwrap();
    let debugs = [
        format!("{candidate:?}"),
        format!("{facts:?}"),
        format!("{nodes:?}"),
        format!("{target:?}"),
    ];
    for debug in &debugs {
        assert_hidden(debug);
    }
    assert_eq!(debugs, SHOWN, "only roles, flags, counts and refs show");
}

const SHOWN: [&str; 4] = [
    concat!(
        r#"Candidate { role: "link", name: "…", href: Some("…"), "#,
        r#"form_action: Some("…"), forbidden_listed: false, allowed_listed: true }"#,
    ),
    "PaywallFacts { headings: 1, dialogs: 2, buttons: 0, hits: 1, .. }",
    r#"[Node { role: "button", name: "…", node: "e1" }]"#,
    r#"Target { role: "button", name: "…", nth: 0, node: "e1" }"#,
];
