//! Proposals the guardrails stop: a missing or replaced element, a doubtful
//! "back", and forbidden elements decided by the material.

use crate::explore::Memory;
use crate::guard::forbidden::Candidate;
use crate::guard::identity::Node;
use crate::guard::paywall::PaywallFacts;
use crate::machine::testing::{
    APP, VIEWPORT, answer, ask_top, asking, below, counts, id, inside, look, operated, page, rules,
    secs, seen, showed, told, top, touch, with,
};
use crate::machine::{Facts, Material, Note, Rejection, Rules, Seen, State, Step, start, step};

/// Clicking "保存" waits for it to be looked up again.
fn looking_up_save() -> State {
    let looked = told(&ask_top(rules()), answer("click", Some(0.9), Some("1")));
    assert_eq!(
        looked.effect(),
        Some(&look(id("button", "保存", 0))),
        "looks 保存 up"
    );
    looked.state().clone()
}

/// The look-up found `seen`, which no longer holds the same "保存".
fn aborted(seen: Seen, why: Rejection) -> Step {
    let step = step(looking_up_save(), Facts::Page(Box::new(seen)), secs(0));
    let state = step.state();
    assert_eq!(state.notes().last(), Some(&Note::Rejected(why)), "{why:?}");
    assert_eq!(counts(state), (0, 0), "{why:?} leaves the counts");
    assert_eq!(state.window().used(secs(0)), 0, "{why:?} uses no access");
    let save = id("button", "保存", 0);
    assert!(
        !operated(state, &top(), &save),
        "{why:?} leaves 保存 untouched"
    );
    step
}

#[test]
fn missing_or_replaced_target_aborts() {
    let gone = page(&format!("{APP}/"), &[("link", "ヘルプ")]);
    let step = aborted(seen(&gone), Rejection::Missing);
    assert_eq!(step.effect(), Some(&asking(&rules(), &gone)), "asks again");
    assert_eq!(step.state().last(), &seen(&gone), "takes the page in");
    let mut swapped = seen(&top());
    swapped.material.nodes[0].node = "x9".to_owned();
    aborted(swapped, Rejection::Replaced);
}

/// The note or the effect after `op` with `p` under `rules`.
fn back_with(rules: &Rules, p: Option<f64>) -> Step {
    told(&ask_top(rules.clone()), answer("back", p, None))
}

fn rejected_low(step: &Step) {
    let note = Note::Rejected(Rejection::LowConfidence);
    assert_eq!(
        step.state().notes().last(),
        Some(&note),
        "too unsure to go back"
    );
}

/// A "back" at `p` goes through under `rules`.
fn back_passes(rules: &Rules, p: f64) {
    let step = back_with(rules, Some(p));
    assert_eq!(step.effect(), Some(&touch("back", None, p)), "back at {p}");
}

#[test]
fn low_confidence_back_is_rejected() {
    let rules = rules();
    rejected_low(&back_with(&rules, Some(0.69)));
    rejected_low(&back_with(&rules, None));
    back_passes(&rules, 0.7);
    let lenient = Rules {
        back_confidence: 0.5,
        ..rules.clone()
    };
    back_passes(&lenient, 0.6);
    let unsure = told(&ask_top(rules), answer("click", None, Some("1")));
    let save = id("button", "保存", 0);
    assert_eq!(
        unsure.effect(),
        Some(&look(save.clone())),
        "click is not stopped"
    );
    let clicked = showed(unsure.state(), &top());
    let expected = touch("click", Some(save), 0.0);
    assert_eq!(clicked.effect(), Some(&expected), "missing confidence is 0");
}

/// Starts on `seen` and passes a click on `number`.
fn click_on(seen: Seen, number: &str) -> Step {
    let asked = start(seen, rules(), Memory::default(), secs(0));
    told(asked.state(), answer("click", Some(0.9), Some(number)))
}

#[test]
fn allow_list_wins_over_forbidden() {
    let screen = with(top(), inside("button", "購入する"));
    let mut allowed = seen(&screen);
    allowed.material.candidates[2].allowed_listed = true;
    let buy = id("button", "購入する", 0);
    assert_eq!(click_on(allowed, "3").effect(), Some(&look(buy)), "allowed");
    let mut listed = seen(&top());
    listed.material.candidates[0].forbidden_listed = true;
    let step = click_on(listed, "1");
    let note = Note::Rejected(Rejection::Forbidden);
    assert_eq!(
        step.state().notes().last(),
        Some(&note),
        "listed as forbidden"
    );
}

/// Material shorter than the elements: missing candidates forbid, missing
/// nodes cannot be looked up.
fn short_material_stops() {
    let mut few = seen(&top());
    few.material.candidates.truncate(1);
    let step = click_on(few, "2");
    let note = Note::Rejected(Rejection::Forbidden);
    assert_eq!(step.state().notes().last(), Some(&note), "no candidate");
    let mut unknown = seen(&top());
    unknown.material.nodes.truncate(1);
    let step = click_on(unknown, "2");
    let note = Note::Rejected(Rejection::Missing);
    assert_eq!(step.state().notes().last(), Some(&note), "no node");
    assert_eq!(step.effect(), Some(&asking(&rules(), &top())), "no look-up");
}

#[test]
fn material_plain_mirrors_screen() {
    let screen = with(top(), below("link", "利用規約"));
    let material = Material::plain(&screen, VIEWPORT);
    let facts = PaywallFacts {
        url: screen.url,
        headings: Vec::new(),
        dialogs: Vec::new(),
        buttons: vec!["保存".to_owned()],
        hits: Vec::new(),
    };
    assert_eq!(material.paywall, facts, "only the visible buttons");
    let elements = [("button", "保存"), ("link", "ヘルプ"), ("link", "利用規約")];
    let candidates: Vec<Candidate> = elements.iter().map(|(r, n)| candidate(r, n)).collect();
    assert_eq!(material.candidates, candidates, "every element, unlisted");
    let nodes: Vec<Node> = elements
        .iter()
        .zip(["p0", "p1", "p2"])
        .map(|((r, n), p)| node(r, n, p))
        .collect();
    assert_eq!(material.nodes, nodes, "referenced by position");
    short_material_stops();
}

fn candidate(role: &str, name: &str) -> Candidate {
    Candidate {
        role: role.to_owned(),
        name: name.to_owned(),
        href: None,
        form_action: None,
        forbidden_listed: false,
        allowed_listed: false,
    }
}

fn node(role: &str, name: &str, node: &str) -> Node {
    Node {
        role: role.to_owned(),
        name: name.to_owned(),
        node: node.to_owned(),
    }
}
