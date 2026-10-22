//! Tests of which step of the log the shot of an issue names.

use super::fixture::{CLICK1, GOAL, cart, confirm, run, seen, sure, unsure};
use crate::interview::clue::Clue;
use crate::interview::issue::{Record, Seen, Shot, step_issues};

#[test]
fn the_shot_names_the_step_of_the_log() {
    let (cart, confirm, sure, unsure) = (cart(), confirm(), sure(), unsure());
    let steps = [
        Seen {
            step: 3,
            ..seen(&cart, Some(&sure), &[], CLICK1)
        },
        Seen {
            step: 7,
            ..seen(&confirm, Some(&unsure), &[Clue::Hesitant], None)
        },
    ];
    let got = step_issues(&run(&[Record {
        outcome: GOAL,
        steps: &steps,
    }]));
    let shots: Vec<Option<Shot>> = got.iter().map(|issue| issue.shot).collect();
    assert_eq!(shots, vec![Some(Shot { trial: 1, step: 7 })]);
}
