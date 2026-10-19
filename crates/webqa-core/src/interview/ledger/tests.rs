//! Materials of the ledger tests, and the tests of the head, the inferred
//! purpose and the order of the entries.

use super::*;
use crate::config::persona::tests::persona;
use crate::interview::clue::Clue;
use crate::interview::issue::Detail;
use crate::interview::issue::tests::fixture::{
    CUT, GAVE_UP, INFERRED, answered, cart, guessing, product, record, run, seen,
};
use crate::interview::issue::tests::sample::{
    mismatch_sample, sample, unreached_sample, visibility_sample,
};
use crate::interview::natural::Trait;
use crate::redact::Redactor;

/// The golden ledger text.
pub const GOLDEN: &str = include_str!("../../../tests/fixtures/interview/ledger.txt");

/// The trait `which` with the text of the test persona.
pub fn persona_trait(which: Trait) -> PersonaTrait {
    PersonaTrait {
        which,
        content: which.content(&persona()),
    }
}

/// The head with `finished` trials and `goal`.
pub fn head(finished: usize, goal: Goal) -> Head {
    Head {
        persona: "佐藤さん".to_owned(),
        goal,
        start: "https://shop.example.test/".to_owned(),
        finished,
        images: true,
    }
}

/// `issue` in `section`, not selected, unchecked and new.
pub const fn entry(issue: Issue, section: Section) -> Entry {
    Entry {
        selected: false,
        section,
        issue,
        persona_trait: None,
        compared: Compared::New,
    }
}

/// The ledger the golden text is written from.
pub fn golden() -> Ledger {
    Ledger {
        head: head(3, Goal::Written("注文を確定できる画面まで進む".to_owned())),
        entries: vec![
            Entry {
                persona_trait: Some(persona_trait(Trait::Wants)),
                ..entry(mismatch_sample(), Section::Body)
            },
            Entry {
                selected: true,
                persona_trait: Some(persona_trait(Trait::ItLiteracy)),
                compared: Compared::Remaining,
                ..entry(sample("I-5334A6B3", 2, 3), Section::Body)
            },
            entry(visibility_sample(), Section::Unchecked),
            Entry {
                persona_trait: Some(persona_trait(Trait::Avoids)),
                ..entry(unreached_sample(), Section::OffPersona)
            },
        ],
        unchecked_purposes: vec![UncheckedPurpose {
            pattern: "/cart".to_owned(),
            purpose: "注文内容を確かめる".to_owned(),
        }],
    }
}

/// The lines of `text`.
pub fn lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

/// `ledger` written without redaction.
pub fn written(ledger: &Ledger) -> String {
    write(ledger, &Redactor::default())
}

/// A ledger of the golden head with `issues` in `section`, in this order.
pub fn ledger_of(section: Section, issues: Vec<Issue>) -> Ledger {
    let entries = issues.into_iter().map(|it| entry(it, section)).collect();
    Ledger {
        entries,
        unchecked_purposes: vec![],
        ..golden()
    }
}

/// The hesitation detail of the samples with `expectation`.
pub fn expecting(expectation: &str) -> Detail {
    Detail::Hesitant {
        clues: vec![Clue::Hesitant, Clue::WentBack],
        expectation: expectation.to_owned(),
    }
}

/// The visibility sample with `reason`.
pub fn hard_to_see(reason: &str) -> Issue {
    let detail = Detail::Visibility {
        reason: reason.to_owned(),
        expectation: "クーポンを入力できると思った".to_owned(),
    };
    Issue {
        detail,
        ..visibility_sample()
    }
}

/// The golden ledger with `head`.
fn golden_with(head: Head) -> Ledger {
    Ledger { head, ..golden() }
}

/// The head of the golden ledger changed, and its expected line `(line
/// number, text)`.
fn changed_heads() -> [(Head, usize, &'static str); 3] {
    let inferred = vec!["商品を選んで注文する".to_owned()];
    [
        (
            head(3, Goal::Inferred(inferred)),
            5,
            "- 目的: 推測 [\"商品を選んで注文する\"]",
        ),
        (head(3, Goal::Inferred(vec![])), 5, "- 目的: 推測 []"),
        (
            Head {
                finished: 1,
                ..golden().head
            },
            7,
            "- 終えた試行: 1",
        ),
    ]
}

#[test]
fn header_has_persona_purpose_url_count() {
    let text = written(&golden());
    assert_eq!(lines(&text)[2..8], lines(GOLDEN)[2..8], "lines 3-8");
    for (head, line, expected) in changed_heads() {
        let text = written(&golden_with(head));
        assert_eq!(lines(&text).get(line - 1), Some(&expected), "line {line}");
    }
}

/// The purposes inferred over the fixture trials, with the first guesses
/// of the first and second trials.
fn inferred_over(first: &str, second: &str) -> Vec<String> {
    let (product, cart) = (product(42), cart());
    let (first, second) = (guessing(first), guessing(second));
    let (third, cut) = (guessing("会員登録をする"), guessing("別の推測"));
    let steps = [
        [answered(&product, &first)],
        [answered(&product, &second)],
        [answered(&product, &cut)],
    ];
    let gave_up = [seen(&product, None, &[], None), answered(&cart, &third)];
    let trials = [
        record(INFERRED, &steps[0]),
        record(INFERRED, &steps[1]),
        record(GAVE_UP, &gave_up),
        record(CUT, &steps[2]),
    ];
    inferred_goal(&run(&trials))
}

/// Guesses of the first two trials, and the purposes inferred from them.
const GUESSES: [(&str, &str, &[&str]); 3] = [
    (
        "商品を選んで注文する",
        " 商品を選んで注文する ",
        &["商品を選んで注文する"],
    ),
    ("   ", "   ", &[]),
    (
        "商品を選んで注文する",
        "会員登録をする",
        &["商品を選んで注文する", "会員登録をする"],
    ),
];

#[test]
fn inferred_purpose_is_written() {
    for (first, second, expected) in GUESSES {
        assert_eq!(
            inferred_over(first, second),
            expected,
            "{first:?} {second:?}"
        );
    }
    let goal = Goal::Inferred(inferred_over(GUESSES[0].0, GUESSES[0].1));
    let ledger = golden_with(head(3, goal));
    let text = written(&ledger);
    let line = "- 目的: 推測 [\"商品を選んで注文する\"]";
    assert_eq!(lines(&text).get(4), Some(&line), "line 5");
    let back = read(&text).map(|it| it.head.goal);
    assert_eq!(back, Ok(ledger.head.goal), "read back");
}

/// The ledger of a run without finished trials.
pub fn no_trial() -> Ledger {
    Ledger {
        head: head(0, Goal::Inferred(vec![])),
        entries: vec![],
        unchecked_purposes: vec![],
    }
}

/// The ledger text of a run without finished trials.
const NO_TRIAL_TEXT: &str = "# インタビューの台帳

- 書式: 1
- ペルソナ: \"佐藤さん\"
- 目的: 推測 []
- 開始の URL: \"https://shop.example.test/\"
- 終えた試行: 0(終えた試行が無いので、課題と出現の割合を出していない)
- 視認性: 確かめた

## 課題

なし

## 確かめられなかった課題

なし

## ペルソナから外れると判断した課題

なし

";

#[test]
fn zero_finished_trials_has_no_issues() {
    let text = written(&no_trial());
    assert_eq!(text, NO_TRIAL_TEXT, "the whole text");
    assert_eq!(text.find("回の試行のうち"), None, "no rate");
    assert_eq!(read(&text), Ok(no_trial()), "read back");
}

#[test]
fn visibility_skipped_note_when_no_image_support() {
    let ledger = golden_with(Head {
        images: false,
        ..golden().head
    });
    let text = written(&ledger);
    let mut expected = lines(GOLDEN);
    expected[7] = "- 視認性: 確かめなかった(判断モデルが画像を扱えないと申告した)";
    assert_eq!(lines(&text), expected, "line 8 only differs");
    let images = read(&text).map(|it| it.head.images);
    assert_eq!(images, Ok(false), "read back");
}

#[test]
fn arrange_sorts_by_section_then_rank() {
    let mut ledger = golden();
    ledger.entries.reverse();
    ledger.arrange();
    assert_eq!(ledger, golden(), "arranged");
    let number = Number::parse("I-5334A6B3").unwrap();
    assert_eq!(golden().selected(), vec![number], "selected");
    let placed = Placed {
        issue: sample("I-5334A6B3", 1, 3),
        section: Section::OffPersona,
        persona_trait: None,
    };
    let expected = entry(sample("I-5334A6B3", 1, 3), Section::OffPersona);
    assert_eq!(Entry::from(placed), expected, "from placed");
}
