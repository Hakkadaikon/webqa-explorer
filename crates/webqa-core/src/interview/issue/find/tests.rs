//! Tests of finding the hesitation, visibility and unreached issues of a
//! step.

use crate::config::raw::Op;
use crate::interview::answer::Reply;
use crate::interview::clue::Clue;
use crate::interview::issue::tests::fixture::{
    BUDGET_SPENT, CLICK1, GAVE_UP, GOAL, IN1, INFERRED, NO_PROGRESS, OUT, UNVERIFIED, answered,
    cart, confirm, count, expecting, listing, next1, record, reply, run, screen, seen, shipping,
    sure, unsure,
};
use crate::interview::issue::tests::sample::{
    EXPECTED, hesitant_at_confirm, unreached_at_shipping, visibility_once,
};
use crate::interview::issue::{Halt, Issue, Number, Place, Run, ScreenMark, Shot, step_issues};
use crate::interview::outcome::Outcome;
use crate::redact::Redacted;
use crate::screen::Screen;

/// The issues of one finished trial of one step at `page`.
fn one_step(page: &Redacted<Screen>, answers: &Reply, images: bool) -> Vec<Issue> {
    let steps = [answered(page, answers)];
    let trials = [record(GOAL, &steps)];
    step_issues(&Run {
        trials: &trials,
        images,
    })
}

fn at_shipping(listed: &[(usize, &str)], images: bool) -> Vec<Issue> {
    one_step(&shipping(), &listing(Some(listed)), images)
}

/// The issues found when element 1 is listed at a page with the footer,
/// outside the viewport, first.
fn footer_first() -> Vec<Issue> {
    let page = screen(
        "/cart2",
        &[("link", "フッター", OUT), ("button", "次へ", IN1)],
    );
    one_step(&page, &listing(Some(&[(1, "文字が薄い")])), true)
}

#[test]
fn listed_hard_to_see_element_is_visibility_issue() {
    let got = at_shipping(&[(2, "文字が薄い")], true);
    assert_eq!(got, vec![visibility_once()], "the listed element");
    let expected = Issue {
        number: Number::parse("I-125DBC69").unwrap(),
        place: Place::Screen {
            url: "https://shop.example.test/cart2".to_owned(),
            mark: ScreenMark::parse("S-E24D83DC").unwrap(),
        },
        target: Some(next1()),
        ..visibility_once()
    };
    assert_eq!(footer_first(), vec![expected], "numbered in the viewport");
    for n in [3, 0] {
        assert_eq!(at_shipping(&[(n, "薄い")], true), vec![], "no element {n}");
    }
    let got = at_shipping(&[(2, "文字が薄い"), (1, "小さい")], true);
    let names: Vec<String> = got
        .into_iter()
        .filter_map(|it| it.target.map(|t| t.name))
        .collect();
    assert_eq!(names, ["クーポンを使う", "郵便番号"], "in the order listed");
}

#[test]
fn no_listed_element_no_visibility_issue() {
    assert_eq!(at_shipping(&[], true), vec![], "none listed");
    let got = one_step(&shipping(), &listing(None), true);
    assert_eq!(got, vec![], "not asked");
}

/// A trial ending as `outcome` after cart and shipping, with an answer at
/// shipping when `answered`.
fn stalled(outcome: Outcome, answered: bool) -> Vec<Issue> {
    let (cart, shipping, sure) = (cart(), shipping(), sure());
    let stuck = expecting(
        "次に進むボタンが見つからない",
        reply(Op::Stuck, None, Some(0.8)),
    );
    let steps = [
        seen(&cart, Some(&sure), &[], CLICK1),
        seen(&shipping, answered.then_some(&stuck), &[], None),
    ];
    step_issues(&run(&[record(outcome, &steps)]))
}

/// Outcomes with the halt of the unreached issue, if any.
const HALTED: [(Outcome, Option<Halt>); 6] = [
    (GAVE_UP, Some(Halt::GaveUp)),
    (NO_PROGRESS, Some(Halt::NoProgress)),
    (BUDGET_SPENT, None),
    (GOAL, None),
    (INFERRED, None),
    (UNVERIFIED, None),
];

#[test]
fn unreached_trial_records_last_screen_and_reason() {
    for (outcome, halt) in HALTED {
        let expected: Vec<Issue> = halt
            .map(|halt| unreached_at_shipping(halt, Some("次に進むボタンが見つからない")))
            .into_iter()
            .collect();
        assert_eq!(stalled(outcome, true), expected, "{outcome:?}");
    }
    let expected = vec![unreached_at_shipping(Halt::GaveUp, None)];
    assert_eq!(stalled(GAVE_UP, false), expected, "unusable last answers");
    let empty = step_issues(&run(&[record(GAVE_UP, &[])]));
    assert_eq!(empty, vec![], "no step");
}

#[test]
fn no_visibility_issue_without_image_support() {
    let listed = [(2, "文字が薄い")];
    assert_eq!(at_shipping(&listed, false), vec![], "no images");
    assert_eq!(
        at_shipping(&listed, true),
        vec![visibility_once()],
        "images"
    );
}

/// The issues of one finished trial of one step at confirm.
fn hesitating(answers: Option<&Reply>, clues: &[Clue]) -> Vec<Issue> {
    let confirm = confirm();
    let steps = [seen(&confirm, answers, clues, None)];
    step_issues(&run(&[record(GOAL, &steps)]))
}

/// Two trials hesitating at confirm: at steps 2 and 3 of the first, with
/// the expectation `A`, and at step 1 of the second, with `B`.
fn hesitating_twice() -> Vec<Issue> {
    let (cart, confirm) = (cart(), confirm());
    let (a, b) = (expecting("A", unsure()), expecting("B", unsure()));
    let first = [
        answered(&cart, &a),
        seen(&confirm, Some(&a), &[Clue::Hesitant], None),
        seen(&confirm, Some(&a), &[Clue::PingPong], None),
    ];
    let second = [seen(&confirm, Some(&b), &[Clue::WentBack], None)];
    step_issues(&run(&[record(GOAL, &first), record(GOAL, &second)]))
}

/// The hesitation with `clue` at confirm, at the one step of one trial.
fn hesitated(clue: Clue) -> Issue {
    hesitant_at_confirm(
        &[clue],
        EXPECTED,
        count(1, 1, 1),
        Shot { trial: 1, step: 1 },
        vec![],
    )
}

#[test]
fn clues_make_hesitant_issue_on_chosen_element() {
    let unsure = unsure();
    let got = hesitating(Some(&unsure), &[Clue::LowConfidence]);
    assert_eq!(got, vec![hesitated(Clue::LowConfidence)], "on the chosen");
    let back = reply(Op::Back, None, Some(0.3));
    let expected = Issue {
        number: Number::parse("I-5B774D44").unwrap(),
        target: None,
        confidence: Some(0.3),
        ..hesitated(Clue::WentBack)
    };
    let got = hesitating(Some(&back), &[Clue::WentBack]);
    assert_eq!(got, vec![expected], "no element");
    assert_eq!(hesitating(None, &[Clue::Hesitant]), vec![], "unusable");
    assert_eq!(hesitating(Some(&unsure), &[]), vec![], "no clue");
    let first = Shot { trial: 1, step: 2 };
    let expected = hesitant_at_confirm(&[Clue::Hesitant], "A", count(2, 2, 2), first, vec![]);
    assert_eq!(hesitating_twice(), vec![expected], "the first one");
}

mod budget;
mod crossing;
