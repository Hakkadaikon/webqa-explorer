//! Tests of the low confidence guard for "back".

use super::*;

#[test]
fn back_confidence_boundary() {
    let cases: [(f64, &[f64], bool); 8] = [
        (0.7, &[0.69], true),
        (0.7, &[0.7], false),
        (0.7, &[0.71], false),
        (0.5, &[0.6], false),
        (0.5, &[0.49], true),
        (1.0, &[1.0], false),
        (0.7, &[0.1, 0.69, 0.2], true),
        (0.7, &[0.1, 0.7], false),
    ];
    for (threshold, probabilities, want) in cases {
        assert_eq!(
            is_low(confidence(probabilities), threshold),
            want,
            "{threshold} {probabilities:?}"
        );
    }
    assert_eq!(confidence(&[0.1, 0.7]), Some(0.7), "the maximum");
}

#[test]
fn back_without_probability_is_low() {
    assert_eq!(confidence(&[]), None, "no probability");
    assert_eq!(confidence(&[f64::NAN]), None, "NaN only");
    assert_eq!(confidence(&[f64::NAN, 0.8]), Some(0.8), "NaN is skipped");
    assert_eq!(confidence(&[f64::INFINITY]), None, "infinity only");
    assert!(is_low(None, 0.7), "no confidence is low");
    assert!(is_low(None, 0.01), "no confidence is low at any threshold");
}
