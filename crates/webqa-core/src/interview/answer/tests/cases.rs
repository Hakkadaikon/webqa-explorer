//! Tests of the added answers that are missing, out of range or read.

use super::*;

const MISSING: [(Changes<'_>, Expected); 14] = [
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
    (
        &[(EXPECTATION, None), (HARD_TO_SEE, None)],
        (Unusable::Missing, EXPECTATION),
    ),
    (
        &[(HARD_TO_SEE, None), (PURPOSE_GUESS, None)],
        (Unusable::Missing, HARD_TO_SEE),
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

const OUT_OF_RANGE: [(Changes<'_>, Expected); 13] = [
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
        &[(HESITATION, Some(&[("true", f64::NEG_INFINITY)]))],
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
