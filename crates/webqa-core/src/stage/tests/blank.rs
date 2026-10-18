//! Tests that empty and blank declarations never hold.

use super::*;

#[test]
fn empty_declarations_never_hold() {
    let rich = [el("heading", "ようこそ"), el("button", ""), el("", "保存")];
    for (path, text) in [("/", ""), ("/home", "本文")] {
        let stages = [stage("ログイン", when(None, None, None))];
        assert_eq!(
            check(&stages, path, &rich, text),
            (false, Goal::Unmet),
            "no declaration at {path}"
        );
    }
    let cases = [
        text_is(""),
        text_is("  "),
        element_is("button", ""),
        element_is("", "保存"),
        when(Some("/home"), None, Some("")),
    ];
    for done_when in cases {
        let shown = format!("{done_when:?}");
        assert!(
            !holds(done_when, "/home", &rich, "本文"),
            "an empty declaration holds: {shown}"
        );
    }
}

#[test]
fn blank_declarations_never_match_blank_values() {
    let stages = [stage("ログイン", element_is("", ""))];
    assert_eq!(
        check(&stages, "/", &[el("", "")], ""),
        (false, Goal::Unmet),
        "an empty element declaration matches an empty element"
    );
    assert!(
        !holds(text_is("  "), "/", &[], "本文  です"),
        "a whitespace text declaration matches whitespace in the page"
    );
    assert!(
        !holds(element_is("button", " "), "/", &[el("button", " ")], ""),
        "a whitespace name matches a whitespace name"
    );
}
