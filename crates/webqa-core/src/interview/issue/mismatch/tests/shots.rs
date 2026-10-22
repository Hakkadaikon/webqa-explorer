//! Tests of which observation a mismatch issue shows, and of a question
//! no finished trial reached.

use crate::interview::answer::Match;
use crate::interview::issue::mismatch::tests::ruled_on;
use crate::interview::issue::mismatch::{Ruling, ruling};
use crate::interview::issue::tests::fixture::{
    CUT, GOAL, answered, cart, count, guessing, product, products, questions, record, run, sure,
};
use crate::interview::issue::tests::sample::mismatch_of;
use crate::interview::issue::{Issue, Seen, Shot};

const JOIN: [&str; 1] = ["会員登録をする"];

#[test]
fn mismatch_screenshot_is_first_match_of_the_trial() {
    let (cart, sure, join) = (cart(), sure(), guessing("会員登録をする"));
    let [p42, p43] = [42, 43].map(product);
    let steps = [
        answered(&p42, &join),
        answered(&cart, &sure),
        answered(&p43, &join),
    ];
    let got = ruled_on(&[record(GOAL, &steps)], Some(Match::Different));
    let shot = Shot { trial: 1, step: 1 };
    let expected = mismatch_of(&JOIN, count(1, 1, 1), shot, vec![]);
    assert_eq!(got, Ruling::Mismatch(Box::new(expected)), "step 1");
}

#[test]
fn the_mismatch_shot_names_the_step_of_the_log() {
    let (cart, sure, join) = (cart(), sure(), guessing("会員登録をする"));
    let page = product(42);
    let steps = [
        Seen {
            step: 3,
            ..answered(&cart, &sure)
        },
        Seen {
            step: 7,
            ..answered(&page, &join)
        },
    ];
    let got = ruled_on(&[record(GOAL, &steps)], Some(Match::Different));
    let shot = Shot { trial: 1, step: 7 };
    let expected = mismatch_of(&JOIN, count(1, 1, 1), shot, vec![]);
    assert_eq!(got, Ruling::Mismatch(Box::new(expected)));
}

#[test]
fn mismatch_with_no_finished_trial_reaching_has_no_screenshot() {
    let join = guessing("会員登録をする");
    let page = product(42);
    let steps = [answered(&page, &join)];
    let asked = [record(GOAL, &steps)];
    let screens = [products()];
    let question = &questions(&screens, &asked)[0];
    let got = ruling(
        question,
        Some(Match::Different),
        &run(&[record(CUT, &steps)]),
    );
    let expected = Issue {
        shot: None,
        ..mismatch_of(&JOIN, count(0, 0, 0), Shot { trial: 1, step: 1 }, vec![])
    };
    assert_eq!(got, Ruling::Mismatch(Box::new(expected)), "none reached");
}
