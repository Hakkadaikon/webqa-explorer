//! Tests of the questions of a step, and the question shared with the tests
//! of reading the answers.

use serde_json::json;

use super::*;

/// All eight operation kinds.
pub const ALL_OPS: [Op; 8] = [
    Op::Click,
    Op::Input,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Wait,
    Op::Done,
    Op::Stuck,
];

/// A step with all eight kinds, two elements and a goal.
pub const ASK8: Ask<'static> = Ask {
    ops: &ALL_OPS,
    elements: 2,
    goal: Some("ログインを済ませる"),
};

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|&n| n.to_owned()).collect()
}

fn choice(id: QuestionId, prompt: &str, options: &[&str]) -> Question {
    Question::Choice {
        id,
        prompt: prompt.to_owned(),
        options: strings(options),
    }
}

fn noul(id: QuestionId, prompt: &str) -> Question {
    Question::Noul {
        id,
        prompt: prompt.to_owned(),
    }
}

/// The options of the `n`th question, read from its serde form.
fn options_of(ask: &Ask<'_>, n: usize) -> Vec<String> {
    let questions = serde_json::to_value(questions(ask)).unwrap();
    serde_json::from_value(questions[n]["options"].clone()).unwrap()
}

fn op_options(ops: &[Op]) -> Vec<String> {
    options_of(&Ask { ops, ..ASK8 }, 0)
}

fn target_options(elements: usize) -> Vec<String> {
    options_of(&Ask { elements, ..ASK8 }, 1)
}

const NAMES: [&str; 8] = [
    "click",
    "input",
    "scroll_down",
    "scroll_up",
    "back",
    "wait",
    "done",
    "stuck",
];

fn error_question() -> Question {
    noul(QuestionId::error_shown(), "画面にエラーの表示があるか")
}

#[test]
fn step_asks_op_target_error_goal() {
    let three = vec![
        choice(QuestionId::operation(), "次に行う操作の種別を選ぶ", &NAMES),
        choice(
            QuestionId::target(),
            "操作する要素の番号を選ぶ",
            &["1", "2"],
        ),
        error_question(),
    ];
    let goal = noul(
        QuestionId::goal_reached(),
        "次のゴールを達成したか: ログインを済ませる",
    );
    let four = [three.clone(), vec![goal]].concat();
    assert_eq!(questions(&ASK8), four, "four questions in order");
    let no_goal = Ask { goal: None, ..ASK8 };
    assert_eq!(questions(&no_goal), three, "no goal, no goal question");
    let back_click = op_options(&[Op::Back, Op::Click]);
    assert_eq!(back_click, ["back", "click"], "the order of ops kept");
    let empty = questions(&Ask {
        goal: Some(""),
        ..ASK8
    });
    let empty_goal = noul(QuestionId::goal_reached(), "次のゴールを達成したか: ");
    assert_eq!(empty.get(3), Some(&empty_goal), "an empty goal");
    assert_eq!(MAX_OPTIONS, 255, "JEV's limit");
}

#[test]
fn no_element_asks_no_target() {
    let none = questions(&Ask {
        elements: 0,
        ..ASK8
    });
    let target = QuestionId::target();
    assert!(
        none.iter().all(|q| *q.id() != target),
        "no element, no question"
    );
}

#[test]
fn questions_send_no_duplicate_or_excess_options() {
    let twice = op_options(&[Op::Click, Op::Back, Op::Click]);
    assert_eq!(twice, ["click", "back"], "the later click left out");
    assert_eq!(op_options(&[Op::Back, Op::Back]), ["back"], "back once");
    let all: Vec<String> = (1..=255).map(|n: usize| n.to_string()).collect();
    for elements in [255, 256, 300] {
        assert_eq!(target_options(elements), all, "{elements} elements");
    }
    for question in questions(&ASK8) {
        if let Question::Choice { options, .. } = question {
            let mut unique = options.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), options.len(), "no duplicate: {options:?}");
        }
    }
}

#[test]
fn op_names_follow_serde() {
    let names: Vec<String> = ALL_OPS.into_iter().map(op_name).collect();
    assert_eq!(names, NAMES, "serde's names");
    let serialized = [
        noul(QuestionId::error_shown(), "p"),
        choice(QuestionId::operation(), "p", &["click"]),
        Question::Score {
            id: QuestionId::new("s"),
            prompt: "p".to_owned(),
            levels: strings(&["低い", "高い"]),
        },
    ]
    .map(|q| serde_json::to_value(q).unwrap());
    let expected = [
        json!({"kind": "noul", "id": "error_shown", "prompt": "p"}),
        json!({"kind": "choice", "id": "operation", "prompt": "p", "options": ["click"]}),
        json!({"kind": "score", "id": "s", "prompt": "p", "levels": ["低い", "高い"]}),
    ];
    assert_eq!(serialized, expected, "the shape sent to the bridge");
}
