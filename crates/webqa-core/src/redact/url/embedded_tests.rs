//! Tests that a secret embedded at mixed depths never shows.

use std::fmt::Write as _;

use super::tests::{assert_hidden, parts};
use crate::redact::{Redactor, Rule};

const SECRETS: [&str; 5] = ["taro@example.test", "ab/cd", "x y+z", "秘密の値", "ab=cd@x"];

/// How many times each character is encoded, in turn. The last one leaves
/// the separator inside a secret as it is, with letters around it deeper.
const DEPTHS: [&[usize]; 9] = [
    &[1],
    &[2],
    &[3],
    &[0, 1],
    &[1, 2, 3],
    &[3, 0, 2],
    &[0, 0, 3],
    &[2, 1],
    &[0, 2, 0],
];

/// Where the secret is put: the path, the query or the fragment.
const PLACES: [&str; 3] = ["/p/{}/q", "/s?k={}&z=1", "/s#f={}"];

/// The secret, the depths and the place of each case.
const EMBEDDED: [(usize, usize, usize); 24] = [
    (0, 0, 0),
    (0, 1, 1),
    (0, 2, 2),
    (0, 3, 0),
    (0, 4, 1),
    (0, 5, 2),
    (0, 6, 0),
    (1, 7, 1),
    (1, 1, 0),
    (1, 2, 1),
    (1, 4, 2),
    (1, 6, 0),
    (2, 0, 1),
    (2, 3, 2),
    (2, 5, 0),
    (2, 7, 1),
    (3, 0, 2),
    (3, 2, 0),
    (3, 4, 1),
    (3, 5, 2),
    (3, 6, 1),
    (1, 8, 0),
    (4, 8, 1),
    (4, 1, 2),
];

/// `secret` with each character encoded as many times as `depths` says.
fn embed(secret: &str, depths: &[usize]) -> String {
    secret
        .chars()
        .zip(depths.iter().cycle())
        .map(|(c, &depth)| escaped(c, depth))
        .collect()
}

fn escaped(c: char, depth: usize) -> String {
    let once = c.to_string().bytes().fold(String::new(), |mut s, b| {
        write!(s, "%{b:02X}").unwrap();
        s
    });
    match depth {
        0 => c.to_string(),
        _ => (1..depth).fold(once, |s, _| s.replace('%', "%25")),
    }
}

#[test]
fn embedded_secrets_never_show() {
    let rules: Vec<Rule> = SECRETS
        .iter()
        .enumerate()
        .map(|(i, secret)| Rule::literal(&format!("s{i}"), secret))
        .collect();
    let r = Redactor::new(&rules).unwrap();
    for (secret, depths, place) in EMBEDDED {
        let url = PLACES[place].replacen("{}", &embed(SECRETS[secret], DEPTHS[depths]), 1);
        assert_hidden(&parts(&r, &url), &SECRETS, &url);
    }
}
