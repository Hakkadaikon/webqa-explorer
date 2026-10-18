//! Answers that arrived but cannot be used.

use super::*;
use crate::judge::ask::tests::ALL_OPS;

/// The problem written as `"<kind> <question>"`, the kind being `missing`,
/// `option` or `range`. Panics on any other kind.
fn unusable(problem: &str) -> Unusable {
    let (kind, id) = problem.split_once(' ').unwrap();
    let id = QuestionId::new(id);
    [
        ("missing", Unusable::Missing(id.clone())),
        ("option", Unusable::NotAnOption(id.clone())),
        ("range", Unusable::OutOfRange(id)),
    ]
    .into_iter()
    .find(|(k, _)| *k == kind)
    .map(|(_, u)| u)
    .expect("the kind is missing, option or range")
}

fn assert_rejected(answers: &Answers, ask: &Ask<'_>, problem: &str) {
    let got = read_answers(answers, ask);
    assert_eq!(got, Err(unusable(problem)), "{answers:?}");
}

#[test]
fn rejects_unusable_answers() {
    let empty = Answers {
        table: BTreeMap::new(),
        ..with(&[])
    };
    assert_rejected(&empty, &ASK8, "missing operation");
    for id in ["operation", "target", "error_shown", "goal_reached"] {
        assert_rejected(&without(id), &ASK8, &format!("missing {id}"));
    }
    let mut drag = without("error_shown");
    drag.table.extend([row("operation drag:0.9")]);
    assert_rejected(&drag, &ASK8, "option operation");
    rejects_rows_of_the_step();
    rejects_rows_of_other_steps();
}

fn rejects_rows_of_the_step() {
    let cases = [
        (["operation"].as_slice(), "missing operation"),
        (&["operation drag:0.9"], "option operation"),
        (&["operation click:0.9 drag:0.1"], "option operation"),
        (&["operation click:0.9 click:0.1"], "option operation"),
        (&["operation click:1.5"], "range operation"),
        (&["operation click:-0.1"], "range operation"),
        (&["operation click:inf"], "range operation"),
        (&["target 99:0.9"], "option target"),
        (&["target 0:0.9"], "option target"),
        (&["target 01:0.9"], "option target"),
        (&["target +1:0.9"], "option target"),
        (&["target 保存:0.9"], "option target"),
        (&["target 1:0.8 3:0.1"], "option target"),
        (&["target 1:0.8 1:0.6"], "option target"),
        (&["target 1:0.8 2:-0.2"], "range target"),
        (&["target"], "missing target"),
        (&["operation input:0.9", "target 3:0.9"], "option target"),
    ];
    for (rows, problem) in cases {
        assert_rejected(&with(rows), &ASK8, problem);
    }
    rejects_flags();
}

fn rejects_flags() {
    let cases = [
        ("error_shown", "missing error_shown"),
        ("error_shown false:0.9", "option error_shown"),
        ("error_shown true:0.3 false:0.7", "option error_shown"),
        ("error_shown yes:0.3", "option error_shown"),
        ("error_shown true:0.1 true:0.2", "option error_shown"),
        ("error_shown true:1.01", "range error_shown"),
        ("goal_reached true:-1.0", "range goal_reached"),
    ];
    for (row, problem) in cases {
        assert_rejected(&with(&[row]), &ASK8, problem);
    }
}

fn rejects_rows_of_other_steps() {
    let ops: Vec<Op> = ALL_OPS.into_iter().filter(|&op| op != Op::Done).collect();
    let no_done = Ask { ops: &ops, ..ASK8 };
    let cases = [
        ("operation done:0.9", no_done, "option operation"),
        ("target 1:0.9", NO_ELEMENT, "option target"),
        ("target", NO_ELEMENT, "missing target"),
        ("target 256:0.9", MANY, "option target"),
    ];
    for (row, ask, problem) in cases {
        assert_rejected(&with(&[row]), &ask, problem);
    }
}
