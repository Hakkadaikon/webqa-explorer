use super::*;

/// The options of one question, each with its probability.
type Table<'a> = &'a [(&'a str, f64)];

fn answers(id: &QuestionId, table: &[(&str, f64)]) -> Answers {
    Answers {
        table: BTreeMap::from([(
            id.clone(),
            table.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect(),
        )]),
        usage: Usage::default(),
    }
}

#[test]
fn best_is_the_most_probable_option() {
    let id = QuestionId::operation();
    let answers = answers(&id, &[("click", 0.2), ("back", 0.7), ("wait", 0.1)]);
    assert_eq!(
        answers.best(&id),
        Some(("back", 0.7)),
        "the option with the highest probability"
    );
}

#[test]
fn best_skips_missing_probabilities() {
    let id = QuestionId::operation();
    let answers = answers(
        &id,
        &[("click", f64::NAN), ("back", 0.2), ("wait", f64::NAN)],
    );
    assert_eq!(
        answers.best(&id),
        Some(("back", 0.2)),
        "NaN is not a probability"
    );
}

#[test]
fn best_is_none_when_every_probability_is_missing() {
    let id = QuestionId::operation();
    let answers = answers(&id, &[("click", f64::NAN), ("back", f64::NAN)]);
    assert_eq!(answers.best(&id), None, "no probability, no best");
}

#[test]
fn best_keeps_the_earlier_option_on_a_tie() {
    let id = QuestionId::operation();
    let answers = answers(&id, &[("wait", 0.1), ("click", 0.45), ("back", 0.45)]);
    assert_eq!(
        answers.best(&id),
        Some(("click", 0.45)),
        "the first of the tied"
    );
}

#[test]
fn best_is_none_without_an_answer() {
    let id = QuestionId::operation();
    assert_eq!(answers(&id, &[]).best(&id), None, "no option, no best");
    assert_eq!(
        answers(&id, &[("click", 1.0)]).best(&QuestionId::target()),
        None,
        "another question has no answer"
    );
}

#[test]
fn every_question_tells_its_id() {
    let id = QuestionId::new("error_shown");
    let prompt = "?".to_owned();
    for question in [
        Question::Noul {
            id: id.clone(),
            prompt: prompt.clone(),
        },
        Question::Choice {
            id: id.clone(),
            prompt: prompt.clone(),
            options: Vec::new(),
        },
        Question::Score {
            id: id.clone(),
            prompt: prompt.clone(),
            levels: Vec::new(),
        },
        Question::Text {
            id: id.clone(),
            prompt,
        },
    ] {
        assert_eq!(question.id(), &id, "the id the question was built with");
    }
    assert_eq!(id.as_str(), "error_shown", "the name it was built from");
}

#[test]
fn text_question_serializes_as_text() {
    let question = Question::Text {
        id: QuestionId::new("expectation"),
        prompt: "p".into(),
    };
    assert_eq!(
        serde_json::to_value(&question).unwrap(),
        serde_json::json!({"kind": "text", "id": "expectation", "prompt": "p"}),
        "the kind is text"
    );
}

#[test]
fn text_answer_is_taken_as_written() {
    let id = QuestionId::new("expectation");
    let nan = f64::NAN;
    let cases: [(Table<'_>, Option<&str>); 7] = [
        (
            &[("押すと注文が確定すると思った", nan)],
            Some("押すと注文が確定すると思った"),
        ),
        (&[("x", 0.3)], Some("x")),
        (&[("  前後に空白  ", nan)], Some("  前後に空白  ")),
        (&[("一行目\n二行目", nan)], Some("一行目\n二行目")),
        (&[("", nan)], Some("")),
        (&[], None),
        (&[("a", nan), ("b", nan)], None),
    ];
    for (table, want) in cases {
        assert_eq!(answers(&id, table).text(&id), want, "{table:?}");
    }
    let other = answers(&QuestionId::operation(), &[("click", 0.9)]);
    assert_eq!(other.text(&id), None, "no answer to the question");
}

#[test]
fn step_asks_the_operation_and_the_element_number() {
    let ops = vec!["click".to_owned(), "back".to_owned()];
    assert_eq!(
        step_questions(&ops, 2),
        vec![
            Question::Choice {
                id: QuestionId::operation(),
                prompt: "次に行う操作の種別を選ぶ".to_owned(),
                options: ops.clone(),
            },
            Question::Choice {
                id: QuestionId::target(),
                prompt: "操作する要素の番号を選ぶ".to_owned(),
                options: vec!["1".to_owned(), "2".to_owned()],
            },
        ],
        "one choice of the operation, one of the element numbers"
    );
}

#[test]
fn step_without_elements_offers_no_element_number() {
    let questions = step_questions(&["back".to_owned()], 0);
    assert_eq!(
        questions.get(1),
        Some(&Question::Choice {
            id: QuestionId::target(),
            prompt: "操作する要素の番号を選ぶ".to_owned(),
            options: Vec::new(),
        }),
        "no element, no number to choose"
    );
}

#[test]
fn judge_error_message_carries_no_value() {
    assert_eq!(
        JudgeError::Exhausted.to_string(),
        "判断モデルの答えが尽きた",
        "a fixed message"
    );
}

#[test]
fn question_ids_are_named() {
    assert_eq!(
        (
            QuestionId::operation().as_str(),
            QuestionId::target().as_str()
        ),
        ("operation", "target"),
        "the names sent to the judge"
    );
}

#[test]
fn joined_answers_hold_both_calls_and_add_the_usage() {
    let used = |input_tokens, output_tokens| Usage {
        input_tokens,
        output_tokens,
    };
    let first = Answers {
        usage: used(10, 2),
        ..answers(&QuestionId::operation(), &[("click", 0.9)])
    };
    let later = Answers {
        usage: used(u64::MAX, 3),
        ..answers(&QuestionId::new("expectation"), &[("進む", f64::NAN)])
    };
    let joined = first.joined(later);
    let ids: Vec<&str> = joined.table.keys().map(QuestionId::as_str).collect();
    assert_eq!(ids, ["expectation", "operation"], "both calls");
    assert_eq!(joined.usage, used(u64::MAX, 5), "added, never overflowing");
}
