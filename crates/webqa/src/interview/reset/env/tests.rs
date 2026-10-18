//! Tests of choosing the variables the reset command gets.

use std::os::unix::ffi::OsStringExt;

use super::*;

/// The variables `run` takes, the values withheld, and the variables passed.
type Case<'a> = (
    &'a [(&'a str, &'a str)],
    &'a [&'a str],
    &'a [(&'a str, &'a str)],
);

const KEY: &str = "value-of-JEV_API_KEY";

const CASES: [Case; 7] = [
    (
        &[("JEV_API_KEY", KEY), ("KEEP", "kept")],
        &[KEY],
        &[("KEEP", "kept")],
    ),
    (
        &[("AUTH", "Bearer value-of-JEV_API_KEY"), ("KEEP", "kept")],
        &[KEY],
        &[("KEEP", "kept")],
    ),
    (&[("B", "2"), ("A", "1")], &[KEY], &[("B", "2"), ("A", "1")]),
    (
        &[("EMPTY", ""), ("KEEP", "kept")],
        &[""],
        &[("EMPTY", ""), ("KEEP", "kept")],
    ),
    (&[("KEEP", "kept")], &[], &[("KEEP", "kept")]),
    (&[], &[KEY], &[]),
    (
        &[("X", "value-of-A"), ("Y", "value-of-B"), ("Z", "z")],
        &["value-of-A", "value-of-B"],
        &[("Z", "z")],
    ),
];

fn vars(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    pairs
        .iter()
        .map(|(name, value)| (OsString::from(name), OsString::from(value)))
        .collect()
}

#[test]
fn withheld_values_never_reach_env() {
    for (given, withheld, expected) in CASES {
        let given = vars(given);
        let seen = passed(&given, withheld);
        assert_eq!(seen, vars(expected), "{given:?} without {withheld:?}");
    }
    let bytes = b"\xffvalue-of-JEV_API_KEY".to_vec();
    let raw = [(OsString::from("RAW"), OsString::from_vec(bytes))];
    assert_eq!(passed(&raw, &[KEY]), vec![], "not UTF-8");
}
