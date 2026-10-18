//! Answers that can be used.

use super::*;

/// What the good answers say, with `op`, `confidence` and `targets` changed.
const fn verdict(op: Op, confidence: Option<f64>, targets: Vec<usize>) -> Verdict {
    Verdict {
        op,
        confidence,
        targets,
        error_shown: Some(0.1),
        goal_reached: Some(0.0),
        usage: Usage {
            input_tokens: 120,
            output_tokens: 3,
        },
    }
}

fn read(rows: &[&str], ask: &Ask<'_>) -> Verdict {
    read_answers(&with(rows), ask).unwrap()
}

#[test]
fn reads_usable_answers() {
    let good = verdict(Op::Click, Some(0.9), vec![1, 2]);
    assert_eq!(read(&[], &ASK8), good, "the good answers");
    assert_eq!(read(&["other x:5.0"], &ASK8), good, "unasked is ignored");
    reads_ranked_options();
    reads_targets_only_when_needed();
    reads_flags();
}

fn reads_ranked_options() {
    let cases = [
        (
            "operation back:0.3 click:0.6",
            Op::Click,
            Some(0.6),
            vec![1, 2],
        ),
        ("operation back:0.5 click:0.5", Op::Back, Some(0.5), vec![]),
        ("operation back:NaN", Op::Back, None, vec![]),
        ("operation click:NaN back:0.2", Op::Back, Some(0.2), vec![]),
        ("target 2:0.6 1:0.8", Op::Click, Some(0.9), vec![1, 2]),
        ("target 1:NaN 2:0.3", Op::Click, Some(0.9), vec![2, 1]),
        ("target 2:0.5 1:0.5", Op::Click, Some(0.9), vec![2, 1]),
    ];
    for (row, op, confidence, targets) in cases {
        let want = verdict(op, confidence, targets);
        assert_eq!(read(&[row], &ASK8), want, "{row}");
    }
}

fn reads_targets_only_when_needed() {
    let last = read(&["target 255:0.9"], &MANY).targets;
    assert_eq!(last, [255], "the last number asked");
    let scroll = "operation scroll_down:0.9";
    let cases = [
        (with(&[scroll]), ASK8),
        (without("target"), ASK8),
        (with(&[scroll, "target 99:0.9"]), ASK8),
        (with(&[scroll, "target 99:0.9"]), NO_ELEMENT),
    ];
    for (mut answers, ask) in cases {
        answers.table.extend([row(scroll)]);
        let got = read_answers(&answers, &ask).map(|v| (v.op, v.targets));
        assert_eq!(got, Ok((Op::ScrollDown, vec![])), "{answers:?}");
    }
}

fn reads_flags() {
    let cases = [
        ("error_shown true:0.3", Some(0.3)),
        ("error_shown true:NaN", None),
        ("error_shown true:1.0", Some(1.0)),
    ];
    for (row, want) in cases {
        assert_eq!(read(&[row], &ASK8).error_shown, want, "{row}");
    }
    let unasked = [without("goal_reached"), with(&["goal_reached maybe:2.0"])];
    for answers in unasked {
        let got = read_answers(&answers, &NO_GOAL).map(|v| v.goal_reached);
        assert_eq!(got, Ok(None), "not asked: {answers:?}");
    }
}
