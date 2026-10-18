//! Tests of the clues that the persona hesitated.

use super::*;
use crate::config::raw::Op;

const THRESHOLDS: Thresholds = Thresholds {
    hesitation: 0.6,
    confidence: 0.5,
};

/// A step that shows no hesitation.
const fn calm() -> Signals<'static> {
    Signals {
        hesitation: 0.1,
        confidence: 0.9,
        executed: Some("click"),
    }
}

const STILL: [&str; 1] = ["商品一覧"];

/// The clues of [`calm`] with the hesitation `hesitation`.
fn hesitating(hesitation: f64, thresholds: Thresholds) -> Vec<Clue> {
    let signals = Signals {
        hesitation,
        ..calm()
    };
    clues(signals, &STILL, thresholds)
}

/// The clues of [`calm`] with the confidence `confidence`.
fn doubting(confidence: f64, thresholds: Thresholds) -> Vec<Clue> {
    let signals = Signals {
        confidence,
        ..calm()
    };
    clues(signals, &STILL, thresholds)
}

const fn hesitation(hesitation: f64) -> Thresholds {
    Thresholds {
        hesitation,
        ..THRESHOLDS
    }
}

const fn confidence(confidence: f64) -> Thresholds {
    Thresholds {
        confidence,
        ..THRESHOLDS
    }
}

/// Hesitations and thresholds, and whether they make a clue.
const HESITATIONS: [(f64, Thresholds, bool); 9] = [
    (0.59, THRESHOLDS, false),
    (0.6, THRESHOLDS, true),
    (0.61, THRESHOLDS, true),
    (1.0, THRESHOLDS, true),
    (0.0, THRESHOLDS, false),
    (f64::NAN, THRESHOLDS, false),
    (0.59, hesitation(0.3), true),
    (1.0, hesitation(1.0), true),
    (0.99, hesitation(1.0), false),
];

#[test]
fn hesitation_threshold_boundary() {
    for (value, thresholds, clue) in HESITATIONS {
        let expected = if clue { vec![Clue::Hesitant] } else { vec![] };
        assert_eq!(
            hesitating(value, thresholds),
            expected,
            "{value} {thresholds:?}"
        );
    }
}

/// Confidences and thresholds, and whether they make a clue.
const CONFIDENCES: [(f64, Thresholds, bool); 8] = [
    (0.49, THRESHOLDS, true),
    (0.5, THRESHOLDS, false),
    (0.0, THRESHOLDS, true),
    (1.0, THRESHOLDS, false),
    (f64::NAN, THRESHOLDS, false),
    (0.99, confidence(1.0), true),
    (1.0, confidence(1.0), false),
    (0.49, confidence(0.2), false),
];

#[test]
fn low_confidence_threshold_boundary() {
    for (value, thresholds, clue) in CONFIDENCES {
        let expected = if clue {
            vec![Clue::LowConfidence]
        } else {
            vec![]
        };
        assert_eq!(
            doubting(value, thresholds),
            expected,
            "{value} {thresholds:?}"
        );
    }
}

/// Executed operations, and whether they make a clue.
const EXECUTED: [(Option<&str>, bool); 4] = [
    (Some("back"), true),
    (Some("click"), false),
    (None, false),
    (Some("scroll_up"), false),
];

#[test]
fn executed_back_is_clue() {
    let trail = ["商品一覧", "商品詳細", "商品一覧"];
    for (executed, clue) in EXECUTED {
        let signals = Signals { executed, ..calm() };
        let expected = if clue { vec![Clue::WentBack] } else { vec![] };
        assert_eq!(clues(signals, &trail, THRESHOLDS), expected, "{executed:?}");
    }
    let op = serde_json::from_value::<Op>(serde_json::Value::from(BACK)).unwrap();
    assert_eq!(op, Op::Back);
}

/// Trails, and whether they end moving back and forth.
const TRAILS: [(&[&str], bool); 12] = [
    (&["A", "B", "A", "B"], true),
    (&["A", "B", "A"], false),
    (&[], false),
    (&["A"], false),
    (&["A", "A", "A", "A"], false),
    (&["A", "A", "B", "A", "B"], true),
    (&["A", "B", "B", "A", "A", "B"], true),
    (&["C", "A", "B", "A", "B"], true),
    (&["A", "B", "A", "B", "C"], false),
    (&["A", "B", "A", "B", "A"], true),
    (&["A", "B", "A", "C"], false),
    (&["A", "B", "C", "A", "B", "C"], false),
];

#[test]
fn ping_pong_between_two_screens_is_clue() {
    for (trail, expected) in TRAILS {
        assert_eq!(ping_pong(trail), expected, "{trail:?}");
    }
    let trail = ["商品一覧", "商品詳細", "商品一覧", "商品詳細"];
    assert_eq!(clues(calm(), &trail, THRESHOLDS), vec![Clue::PingPong]);
}

/// `len` screens alternating between A and B.
fn alternating(len: usize) -> Vec<&'static str> {
    ["A", "B"].into_iter().cycle().take(len).collect()
}

#[test]
fn ping_pong_leaves_stuck_counter() {
    let first = clues(calm(), &alternating(4), THRESHOLDS);
    assert_eq!(first, vec![Clue::PingPong]);
    for len in 4..=10 {
        let trail = alternating(len);
        let answer = clues(calm(), &trail, THRESHOLDS);
        assert_eq!(answer, vec![Clue::PingPong], "{len}");
        assert_eq!(answer, first, "{len}");
        assert_eq!(clues(calm(), &trail, THRESHOLDS), answer, "{len} again");
    }
    for len in 1..=3 {
        let trail = alternating(len);
        assert_eq!(clues(calm(), &trail, THRESHOLDS), vec![], "{len}");
        assert_eq!(clues(calm(), &trail, THRESHOLDS), vec![], "{len} again");
    }
}

#[test]
fn clues_come_in_fixed_order() {
    let trail = ["A", "B", "A", "B"];
    let signals = Signals {
        hesitation: 0.9,
        confidence: 0.1,
        executed: Some("back"),
    };
    let all = vec![
        Clue::Hesitant,
        Clue::LowConfidence,
        Clue::WentBack,
        Clue::PingPong,
    ];
    assert_eq!(clues(signals, &trail, THRESHOLDS), all);
    let calmer = Signals {
        hesitation: 0.1,
        ..signals
    };
    let rest = vec![Clue::LowConfidence, Clue::WentBack, Clue::PingPong];
    assert_eq!(clues(calmer, &trail, THRESHOLDS), rest);
}
