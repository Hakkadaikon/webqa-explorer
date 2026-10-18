//! Tests of the added answers that are missing, out of range or read.

use super::*;

const MISSING: [(Changes<'_>, Expected); 12] = [
    (&[(HESITATION, None)], (Unusable::Missing, HESITATION)),
    (&[(HESITATION, Some(&[]))], (Unusable::Missing, HESITATION)),
    (
        &[(HESITATION, Some(&[("true", NAN)]))],
        (Unusable::Missing, HESITATION),
    ),
    (&[(EXPECTATION, None)], (Unusable::Missing, EXPECTATION)),
    (
        &[(EXPECTATION, Some(&[("", NAN)]))],
        (Unusable::Missing, EXPECTATION),
    ),
    (
        &[(EXPECTATION, Some(&[("  \n", NAN)]))],
        (Unusable::Missing, EXPECTATION),
    ),
    (
        &[(EXPECTATION, Some(&[("a", NAN), ("b", NAN)]))],
        (Unusable::Missing, EXPECTATION),
    ),
    (&[(HARD_TO_SEE, None)], (Unusable::Missing, HARD_TO_SEE)),
    (&[(PURPOSE_GUESS, None)], (Unusable::Missing, PURPOSE_GUESS)),
    (
        &[(PURPOSE_GUESS, Some(&[(" ", NAN)]))],
        (Unusable::Missing, PURPOSE_GUESS),
    ),
    (&[("operation", None)], (Unusable::Missing, "operation")),
    (
        &[(HESITATION, None), (PURPOSE_GUESS, None)],
        (Unusable::Missing, HESITATION),
    ),
];

#[test]
fn missing_added_answer_is_anomaly() {
    let shot = shot();
    let step = step(Some(&shot));
    rejects(&step, &MISSING);
    let reply = read_step(&full(true), &step).map(|r| r.verdict);
    assert_eq!(reply, Ok(verdict()), "every answer");
}

const OUT_OF_RANGE: [(Changes<'_>, Expected); 12] = [
    (
        &[(HESITATION, Some(&[("true", 1.2)]))],
        (Unusable::OutOfRange, HESITATION),
    ),
    (
        &[(HESITATION, Some(&[("true", -0.1)]))],
        (Unusable::OutOfRange, HESITATION),
    ),
    (
        &[(HESITATION, Some(&[("true", f64::INFINITY)]))],
        (Unusable::OutOfRange, HESITATION),
    ),
    (
        &[(HESITATION, Some(&[("false", 0.3)]))],
        (Unusable::NotAnOption, HESITATION),
    ),
    (
        &[(HESITATION, Some(&[("true", 0.3), ("true", 0.4)]))],
        (Unusable::NotAnOption, HESITATION),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("99: 文字が薄い", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("3: 文字が薄い", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("0: 文字が薄い", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("02: 文字が薄い", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("文字が薄い", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("2:  ", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
    (
        &[(HARD_TO_SEE, Some(&[("2: 薄い\n2: 小さい", NAN)]))],
        (Unusable::NotAnOption, HARD_TO_SEE),
    ),
];

const NO_ELEMENTS: [(Changes<'_>, Expected); 1] = [(
    &[(HARD_TO_SEE, Some(&[("1: 薄い", NAN)]))],
    (Unusable::NotAnOption, "target"),
)];

#[test]
fn added_answer_out_of_range_is_anomaly() {
    let shot = shot();
    let step = step(Some(&shot));
    rejects(&step, &OUT_OF_RANGE);
    rejects(
        &Step {
            elements: 0,
            ..step
        },
        &NO_ELEMENTS,
    );
}

#[test]
fn hesitation_range_edges_accepted() {
    for p in [0.0, 1.0, 0.5] {
        let answers = changed(full(true), &[(HESITATION, Some(&[("true", p)]))]);
        let shot = shot();
        let got = read_step(&answers, &step(Some(&shot))).map(|r| r.hesitation);
        assert_eq!(got, Ok(p), "{p}");
    }
}

/// Answers to `hard_to_see` with the elements read from them.
const SIGHTS: [(&str, &[(usize, &str)]); 5] = [
    ("", &[]),
    ("\n  \n", &[]),
    ("1: 小さい\n2: 薄い", &[(1, "小さい"), (2, "薄い")]),
    (
        " 2 :  文字が薄い  \r\n\n1:小さい",
        &[(2, "文字が薄い"), (1, "小さい")],
    ),
    ("2: 色: 薄い", &[(2, "色: 薄い")]),
];

fn sights(listed: &[(usize, &str)]) -> Vec<HardToSee> {
    let listed = listed.iter().map(|&(element, reason)| HardToSee {
        element,
        reason: reason.to_owned(),
    });
    listed.collect()
}

#[test]
fn hard_to_see_required_with_image_support() {
    let shot = shot();
    let step = step(Some(&shot));
    let missing = read_step(&full(false), &step);
    assert_eq!(
        missing,
        Err(Unusable::Missing(QuestionId::new(HARD_TO_SEE))),
        "asked, not answered"
    );
    let got = read_step(&full(true), &step).map(|r| r.hard_to_see);
    assert_eq!(
        got,
        Ok(Some(sights(&[(2, "文字が薄い")]))),
        "the one listed"
    );
    for (text, listed) in SIGHTS {
        let answers = changed(full(true), &[(HARD_TO_SEE, Some(&[(text, NAN)]))]);
        let got = read_step(&answers, &step).map(|r| r.hard_to_see);
        assert_eq!(got, Ok(Some(sights(listed))), "{text:?}");
    }
}
