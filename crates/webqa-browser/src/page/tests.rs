//! Tests of putting what the browser read into one observation.

use url::Url;
use webqa_core::guard::forbidden::Candidate;
use webqa_core::guard::identity::Node;
use webqa_core::guard::paywall::PaywallFacts;
use webqa_core::screen::Viewport;

use super::*;
use crate::snapshot::parse;

/// The sample page of the snapshot tests.
const SAMPLE: &str = r#"- generic [active] [ref=e1] [box=8,8,784,300]:
  - heading "見出し 1" [level=1] [ref=e2] [box=8,21,784,37]
  - textbox "メール" [ref=e3] [box=8,99,185,21]: a@example.test
  - 'checkbox "同意" [checked] [ref=e4] [box=299,101,13,13]'
  - link "設定" [ref=e5] [cursor=pointer] [box=8,80,38,17]:
    - /url: /settings
  - button [ref=e6] [box=84,141,73,22]:
    - img "保存" [ref=e7] [box=92,144,57,16]
  - button "閉じる" [ref=e8] [box=117,33,24,21]: ×
  - button "無効" [disabled] [ref=e9] [box=450,32,43,24]
  - text: 地の文
  - dialog [ref=e10] [box=244,181,311,126]:
    - 'button "閉じる" [ref=e11] [box=263,267,84,21]'
  - iframe [ref=e12] [box=102,2231,304,154]:
    - button "枠の中" [ref=f1e2] [box=8,8,85,21]"#;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

fn home() -> Url {
    Url::parse("https://app.example.test/home").unwrap()
}

fn settings() -> Url {
    Url::parse("https://app.example.test/settings").unwrap()
}

fn checkout() -> Url {
    Url::parse("https://app.example.test/checkout").unwrap()
}

/// What the page script found for the element of `node`: each material
/// stands on its own element.
fn probe_of(node: &str) -> Probe {
    Probe {
        hit: node != "e8",
        href: (node == "e5").then(settings),
        form_action: (node == "e6").then(checkout),
        forbidden_listed: node == "e3",
        allowed_listed: node == "e11",
    }
}

/// The sample, each entry with its probe, leaving out the entry of `skip`.
fn observed(skip: &str) -> Observation {
    let snapshot = parse(SAMPLE).unwrap();
    let probed = snapshot
        .entries
        .into_iter()
        .filter(|e| e.node != skip)
        .map(|e| {
            let probe = probe_of(&e.node);
            (e, probe)
        })
        .collect();
    let console = vec!["TypeError: x".to_owned()];
    assemble(home(), VIEW, probed, snapshot.headings, console)
}

fn node(role: &str, name: &str, node: &str) -> Node {
    Node {
        role: role.to_owned(),
        name: name.to_owned(),
        node: node.to_owned(),
    }
}

/// A candidate of `role` and `name` with nothing found.
fn plain(role: &str, name: &str) -> Candidate {
    Candidate {
        role: role.to_owned(),
        name: name.to_owned(),
        href: None,
        form_action: None,
        forbidden_listed: false,
        allowed_listed: false,
    }
}

fn names(o: &Observation) -> Vec<&str> {
    o.screen.elements.iter().map(|e| e.name.as_str()).collect()
}

fn refs(o: &Observation) -> Vec<&str> {
    o.nodes.iter().map(|n| n.node.as_str()).collect()
}

fn actions(o: &Observation) -> Vec<Option<&str>> {
    let actions = o.candidates.iter().map(|c| c.form_action.as_ref());
    actions.map(|u| u.map(Url::as_str)).collect()
}

#[test]
fn observation_lines_up_materials() {
    let o = observed("");
    assert_eq!(
        names(&o),
        ["メール", "同意", "設定", "", "閉じる", "閉じる"],
        "the elements"
    );
    assert_eq!(refs(&o), ["e3", "e4", "e5", "e6", "e8", "e11"], "the refs");
    let checkout = Some("https://app.example.test/checkout");
    assert_eq!(
        actions(&o),
        [None, None, None, checkout, None, None],
        "the form of e6 only"
    );
    let paywall = PaywallFacts {
        url: home(),
        headings: vec!["見出し 1".to_owned()],
        dialogs: vec!["閉じる".to_owned()],
        buttons: vec!["閉じる".to_owned()],
        hits: vec![true, true, true, true, false],
    };
    assert_eq!(o.paywall, paywall, "the paywall's facts");
    assert_eq!(o.console, ["TypeError: x"], "the console");
    assert_eq!(
        (o.screen.url.clone(), o.viewport),
        (home(), VIEW),
        "the page and its viewport"
    );
}

#[test]
fn element_without_probe_leaves_every_list() {
    let o = observed("e4");
    assert_eq!(
        names(&o),
        ["メール", "設定", "", "閉じる", "閉じる"],
        "no 同意"
    );
    assert_eq!(refs(&o), ["e3", "e5", "e6", "e8", "e11"], "no e4");
    let candidates: Vec<&str> = o.candidates.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        candidates,
        ["メール", "設定", "", "閉じる", "閉じる"],
        "no candidate for 同意"
    );
    assert_eq!(
        o.paywall.hits,
        [true, true, true, false],
        "four main operations"
    );
}

#[test]
fn candidates_and_nodes_carry_each_element() {
    let o = observed("");
    let nodes = vec![
        node("textbox", "メール", "e3"),
        node("checkbox", "同意", "e4"),
        node("link", "設定", "e5"),
        node("button", "", "e6"),
        node("button", "閉じる", "e8"),
        node("button", "閉じる", "e11"),
    ];
    assert_eq!(o.nodes, nodes, "role, name and ref of each element");
    let candidates = vec![
        Candidate {
            forbidden_listed: true,
            ..plain("textbox", "メール")
        },
        plain("checkbox", "同意"),
        Candidate {
            href: Some(settings()),
            ..plain("link", "設定")
        },
        Candidate {
            form_action: Some(checkout()),
            ..plain("button", "")
        },
        plain("button", "閉じる"),
        Candidate {
            allowed_listed: true,
            ..plain("button", "閉じる")
        },
    ];
    assert_eq!(o.candidates, candidates, "each probe on its own element");
}
