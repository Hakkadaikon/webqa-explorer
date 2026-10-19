//! Tests of the issues read from the judge's answers.

use std::collections::BTreeMap;

use super::*;
use crate::issue::IssueKind;

type Options = &'static [(&'static str, f64)];

/// The answers to "error shown" and "confusing display", each left out
/// when `None`.
fn answers(error: Option<Options>, confusing: Option<Options>) -> Answers {
    let ids = [
        QuestionId::error_shown(),
        QuestionId::new("confusing_display"),
    ];
    let table: BTreeMap<QuestionId, Vec<(String, f64)>> = ids
        .into_iter()
        .zip([error, confusing])
        .filter_map(|(id, options)| options.map(|o| (id, owned(o))))
        .collect();
    Answers {
        table,
        ..Answers::default()
    }
}

fn owned(options: Options) -> Vec<(String, f64)> {
    options.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect()
}

fn error(p: f64) -> Issue {
    Issue {
        kind: IssueKind::ErrorDisplay,
        evidence: None,
        label: None,
        p: Some(p),
    }
}

fn confusing(p: f64) -> Issue {
    Issue {
        kind: IssueKind::Semantic,
        evidence: None,
        label: Some("わかりにくい表示".to_owned()),
        p: Some(p),
    }
}

/// The answers of each row, and the issues they show.
fn rows() -> [(Answers, Vec<Issue>); 7] {
    [
        (
            answers(Some(&[("true", 0.9)]), Some(&[("true", 0.1)])),
            vec![error(0.9)],
        ),
        (answers(Some(&[("true", 0.8)]), None), vec![error(0.8)]),
        (
            answers(Some(&[("true", 0.79)]), Some(&[("true", 0.8)])),
            vec![confusing(0.8)],
        ),
        (
            answers(Some(&[("false", 0.9), ("true", 0.1)]), None),
            Vec::new(),
        ),
        (answers(Some(&[("true", f64::NAN)]), None), Vec::new()),
        (answers(None, None), Vec::new()),
        (
            answers(Some(&[("true", 0.95)]), Some(&[("true", 0.85)])),
            vec![error(0.95), confusing(0.85)],
        ),
    ]
}

#[test]
fn answers_give_error_and_semantic_issues() {
    let probes = probes();
    for (n, (answers, issues)) in rows().into_iter().enumerate() {
        let found = answered_issues(&answers, 0.8, &probes);
        assert_eq!(found, issues, "row {n}");
    }
    let semantic_only = answers(Some(&[("true", 0.79)]), Some(&[("true", 0.8)]));
    let none = answered_issues(&semantic_only, 0.8, &[]);
    assert_eq!(none, Vec::<Issue>::new(), "nothing is asked without probes");
}

#[test]
fn probe_questions_ask_yes_or_no() {
    let asked = Question::Noul {
        id: QuestionId::new("confusing_display"),
        prompt: "画面にわかりにくい表示があるか".to_owned(),
    };
    assert_eq!(
        probe_questions(&probes()),
        [asked],
        "one yes-or-no question"
    );
    assert_eq!(probe_questions(&[]), Vec::<Question>::new(), "none");
}
