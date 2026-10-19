//! Tests of the names of the comparison with the previous run.

use crate::interview::ledger::Compared::{New, Remaining, Resolved, Unverified};
use crate::interview::ledger::read;
use crate::interview::ledger::tests::{golden, written};

#[test]
fn compared_names_round_trip() {
    let mut ledger = golden();
    let all = [Resolved, Unverified, New, Remaining];
    for (entry, compared) in ledger.entries.iter_mut().zip(all) {
        entry.compared = compared;
    }
    let text = written(&ledger);
    assert_eq!(read(&text), Ok(ledger), "read back");
    assert!(
        text.contains("前回との照らし合わせ: 解消した\n"),
        "resolved"
    );
    assert!(
        text.contains("前回との照らし合わせ: 確かめられなかった\n"),
        "unverified"
    );
}
