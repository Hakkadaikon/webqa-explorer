//! Tests of issues of one kind on one element at different screens, and
//! of the path to an issue.

use crate::interview::clue::Clue;
use crate::interview::issue::tests::fixture::{
    CLICK1, GOAL, cart, clicked, confirm, count, record, run, seen, sure, unsure,
};
use crate::interview::issue::{Place, ScreenMark, step_issues};

fn place(path: &str, mark: &str) -> Place {
    Place::Screen {
        url: format!("https://shop.example.test{path}"),
        mark: ScreenMark::parse(mark).unwrap(),
    }
}

#[test]
fn same_element_and_kind_at_two_screens_are_two_issues() {
    let (cart, confirm, unsure) = (cart(), confirm(), unsure());
    let steps = [
        seen(&cart, Some(&unsure), &[Clue::Hesitant], None),
        seen(&confirm, Some(&unsure), &[Clue::Hesitant], None),
    ];
    let got = step_issues(&run(&[record(GOAL, &steps)]));
    let places: Vec<_> = got.iter().map(|it| it.place.clone()).collect();
    assert_eq!(
        places,
        [
            place("/cart", "S-E6F64D7B"),
            place("/confirm", "S-B9520D77")
        ],
        "one per screen"
    );
    let counts: Vec<_> = got.iter().map(|it| it.count).collect();
    assert_eq!(counts, [count(1, 1, 1), count(1, 1, 1)], "each once");
    assert_eq!(got[0].target, got[1].target, "the same element");
    assert_ne!(got[0].number, got[1].number, "numbered apart");
}

#[test]
fn path_leaves_out_what_ran_at_the_issue_step() {
    let (cart, confirm, sure, unsure) = (cart(), confirm(), sure(), unsure());
    let steps = [
        seen(&cart, Some(&sure), &[], CLICK1),
        seen(&confirm, Some(&unsure), &[Clue::Hesitant], CLICK1),
    ];
    let got = step_issues(&run(&[record(GOAL, &steps)]));
    let paths: Vec<_> = got.into_iter().map(|it| it.path).collect();
    assert_eq!(paths, [clicked()], "only the click before the step");
}
