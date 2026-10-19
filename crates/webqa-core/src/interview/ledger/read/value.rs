//! Reading the lines of a ledger, and the values of the fields whose
//! form depends on the kind.

use std::iter::{Enumerate, Peekable};
use std::str;

use serde::de::DeserializeOwned;

use crate::interview::issue::{CLUES, Count, Detail, HALTS, Kind};
use crate::interview::ledger::LedgerError;
use crate::interview::ledger::label::key_of;
use crate::interview::ledger::wire::WireMismatch;

/// The value of a line, and the line's number from 1.
pub(super) type Field<'a> = (&'a str, usize);

/// The lines of a ledger text, numbered from 1.
pub(super) struct Lines<'a> {
    lines: Peekable<Enumerate<str::Lines<'a>>>,
    /// The number of the last line taken.
    last: usize,
}

impl<'a> Lines<'a> {
    /// The lines of `text`.
    pub(super) fn new(text: &'a str) -> Self {
        Self {
            lines: text.lines().enumerate().peekable(),
            last: 0,
        }
    }

    /// The next line, without taking it.
    pub(super) fn peek(&mut self) -> Option<&'a str> {
        self.lines.peek().map(|&(_, line)| line)
    }

    /// Takes the next line with its number. Past the end, the line one
    /// past the last is malformed.
    pub(super) fn next(&mut self) -> Result<Field<'a>, LedgerError> {
        let (i, line) = self.lines.next().at(self.last + 1)?;
        self.last = i + 1;
        Ok((line, self.last))
    }

    /// Takes the next line with its number if `want` accepts it.
    pub(super) fn next_if(&mut self, want: impl Fn(&str) -> bool) -> Option<Field<'a>> {
        let (i, line) = self.lines.next_if(|&(_, line)| want(line))?;
        self.last = i + 1;
        Some((line, self.last))
    }

    /// Takes the next line, which must be `want`.
    pub(super) fn exact(&mut self, want: &str) -> Result<(), LedgerError> {
        let (line, n) = self.next()?;
        (line == want).then_some(()).at(n)
    }

    /// Takes the next line, which must start with `prefix`: the rest.
    pub(super) fn after(&mut self, prefix: &str) -> Result<Field<'a>, LedgerError> {
        let (line, n) = self.next()?;
        line.strip_prefix(prefix).map(|rest| (rest, n)).at(n)
    }

    /// Checks that no line is left: a line left is malformed.
    pub(super) fn end(&mut self) -> Result<(), LedgerError> {
        let left = self.lines.peek().map(|&(i, _)| i + 1);
        left.map_or(Ok(()), |line| Err(malformed(line)))
    }
}

/// Turns a missing value into the error of a line.
pub(super) trait At<T> {
    /// The value, or [`LedgerError::Malformed`] at `line`.
    fn at(self, line: usize) -> Result<T, LedgerError>;
}

impl<T> At<T> for Option<T> {
    fn at(self, line: usize) -> Result<T, LedgerError> {
        self.ok_or(LedgerError::Malformed { line })
    }
}

/// The error for line `line`.
pub(super) const fn malformed(line: usize) -> LedgerError {
    LedgerError::Malformed { line }
}

/// The JSON value of `field`.
pub(super) fn json<T: DeserializeOwned>((text, line): Field<'_>) -> Result<T, LedgerError> {
    serde_json::from_str(text).map_err(|_| malformed(line))
}

/// The key named by `field` in `table`.
pub(super) fn named<T: Copy>(
    table: &[(T, &str)],
    (text, line): Field<'_>,
) -> Result<T, LedgerError> {
    key_of(table, text).at(line)
}

/// `text` as a decimal written without leading zeros or signs.
pub(super) fn decimal(text: &str) -> Option<usize> {
    text.parse::<usize>().ok().filter(|n| n.to_string() == text)
}

/// The detail of `kind` from the reason and the expectation.
pub(super) fn detail(
    kind: Kind,
    reason: Field<'_>,
    expectation: Field<'_>,
) -> Result<Detail, LedgerError> {
    match kind {
        Kind::Hesitant => hesitant(reason, expectation),
        Kind::Visibility => visibility(reason, expectation),
        Kind::Unreached => unreached(reason, expectation),
        Kind::Mismatch => mismatch(reason, expectation),
    }
}

fn hesitant(reason: Field<'_>, expectation: Field<'_>) -> Result<Detail, LedgerError> {
    let names: Vec<String> = json(reason)?;
    let clues = names
        .iter()
        .map(|name| key_of(&CLUES, name))
        .collect::<Option<Vec<_>>>()
        .at(reason.1);
    clues.and_then(|clues| {
        json(expectation).map(|expectation| Detail::Hesitant { clues, expectation })
    })
}

fn visibility(reason: Field<'_>, expectation: Field<'_>) -> Result<Detail, LedgerError> {
    let reason = json(reason);
    reason.and_then(|reason| {
        json(expectation).map(|expectation| Detail::Visibility {
            reason,
            expectation,
        })
    })
}

fn unreached(reason: Field<'_>, expectation: Field<'_>) -> Result<Detail, LedgerError> {
    let name = json::<String>(reason);
    let halt = name.and_then(|name| named(&HALTS, (&name, reason.1)));
    halt.and_then(|halt| {
        json(expectation).map(|expectation| Detail::Unreached { halt, expectation })
    })
}

fn mismatch(reason: Field<'_>, expectation: Field<'_>) -> Result<Detail, LedgerError> {
    let wire = json::<WireMismatch>(reason);
    wire.and_then(|wire| {
        json::<()>(expectation).map(|()| Detail::Mismatch {
            purpose: wire.purpose,
            guesses: wire.guesses,
        })
    })
}

/// How often the issue happened.
pub(super) fn count(occurred: Field<'_>, reached: Field<'_>) -> Result<Count, LedgerError> {
    let (finished, k) = ratio(occurred.0).at(occurred.1)?;
    decimal(reached.0).at(reached.1).map(|all| Count {
        occurred: k,
        finished,
        reached: all,
    })
}

/// N and k of `{N}回の試行のうち{k}回`, with k at most N.
fn ratio(text: &str) -> Option<(usize, usize)> {
    text.strip_suffix('回')?
        .split_once("回の試行のうち")
        .and_then(|(n, k)| decimal(n).zip(decimal(k)))
        .filter(|(n, k)| k <= n)
}
