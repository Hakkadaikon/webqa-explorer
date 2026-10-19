//! Tests of the heads of the two ledgers.

use super::*;

fn with_head(head: Head) -> Ledger {
    Ledger {
        head,
        ..ledger(3, vec![])
    }
}

#[test]
fn persona_or_purpose_mismatch_rejected() {
    let base = head(3, goal());
    let inferred = |text: &str, start: &str| Head {
        goal: Goal::Inferred(vec![text.to_owned()]),
        start: start.to_owned(),
        ..base.clone()
    };
    let other = "https://other.example.test/";
    let cases = [
        (
            with_head(base.clone()),
            Head {
                persona: "鈴木さん".to_owned(),
                ..base.clone()
            },
            Mismatch::Persona,
        ),
        (
            with_head(base.clone()),
            Head {
                goal: Goal::Written("別の目的".to_owned()),
                ..base.clone()
            },
            Mismatch::Goal,
        ),
        (
            with_head(base.clone()),
            inferred("商品を買う", &base.start),
            Mismatch::Goal,
        ),
        (
            with_head(inferred("a", &base.start)),
            inferred("a", other),
            Mismatch::Start,
        ),
    ];
    for (now, previous, expected) in cases {
        let previous = with_head(previous);
        assert_eq!(
            compared(&now, &previous, 3, 3),
            Err(expected),
            "{expected:?}"
        );
    }
    messages_hide_texts();
}

/// The sentences of the errors, and that no text of a ledger is in them.
fn messages_hide_texts() {
    let texts = [Mismatch::Persona, Mismatch::Goal, Mismatch::Start].map(|it| it.to_string());
    let expected = [
        "前回の台帳のペルソナが今回と違う",
        "前回の台帳の目的が今回と違う",
        "目的を推測させた前回の実行と開始の URL が違う",
    ];
    assert_eq!(texts, expected);
    for text in texts.iter().chain(&[format!("{:?}", Mismatch::Start)]) {
        for secret in ["佐藤さん", "鈴木さん", "注文を確定", "https://", "example"] {
            assert!(!text.contains(secret), "{text} {secret}");
        }
    }
}

#[test]
fn inferred_runs_compare_by_start_url() {
    let inferred = |text: &str| Head {
        goal: Goal::Inferred(vec![text.to_owned()]),
        ..head(3, goal())
    };
    let previous = Ledger {
        head: inferred("商品を選んで注文する"),
        ..ledger(3, vec![body(sample(X, 1, 3))])
    };
    let now = Ledger {
        head: inferred("商品を買う"),
        ..ledger(3, vec![body(sample(X, 2, 3))])
    };
    let expected = Ledger {
        entries: vec![Entry {
            compared: Compared::Remaining,
            ..body(sample(X, 2, 3))
        }],
        ..now.clone()
    };
    assert_eq!(compared(&now, &previous, 3, 3), Ok(expected));
}

#[test]
fn written_runs_ignore_start_url() {
    let previous = with_head(Head {
        start: "https://other.example.test/".to_owned(),
        ..head(3, goal())
    });
    let now = with_head(head(3, goal()));
    assert_eq!(compared(&now, &previous, 3, 3), Ok(now));
}
