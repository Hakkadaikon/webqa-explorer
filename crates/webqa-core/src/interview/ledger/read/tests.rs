//! Tests of reading a ledger back: round trips, forged structure and
//! edited checkboxes.

pub(in crate::interview::ledger) mod broken;

use crate::config::raw::Op;
use crate::interview::clue::Clue;
use crate::interview::issue::tests::sample::{mismatch_sample, sample, unreached_sample};
use crate::interview::issue::{Act, Detail, Halt, Issue, Number, Target, UncheckedPurpose};
use crate::interview::ledger::select::tests::selecting;
use crate::interview::ledger::tests::{
    GOLDEN, expecting, golden, hard_to_see, head, ledger_of, lines, no_trial, persona_trait,
    written,
};
use crate::interview::ledger::{Goal, Head, Ledger, read};
use crate::interview::natural::{PersonaTrait, Section, TRAITS, Trait};

/// `issue` numbered `number`.
fn numbered(number: &str, issue: Issue) -> Issue {
    Issue {
        number: Number::parse(number).unwrap(),
        ..issue
    }
}

/// The hesitation sample numbered `number` with `change` applied.
fn hesitant(number: &str, change: fn(&mut Issue)) -> Issue {
    let mut issue = sample(number, 2, 3);
    change(&mut issue);
    issue
}

/// Issues with the values that have more than one form.
fn varied() -> Vec<Issue> {
    vec![
        hesitant("I-00000001", |it| {
            it.target = Some(Target {
                rect: None,
                nth: 2,
                ..it.target.clone().unwrap()
            });
        }),
        hesitant("I-00000002", |it| it.shot = None),
        hesitant("I-00000003", |it| {
            it.path = vec![Act {
                op: Op::Wait,
                target: None,
            }];
        }),
        hesitant("I-00000004", |it| {
            it.detail = Detail::Hesitant {
                clues: vec![
                    Clue::Hesitant,
                    Clue::LowConfidence,
                    Clue::WentBack,
                    Clue::PingPong,
                ],
                expectation: "押すと注文が確定すると思った".to_owned(),
            };
        }),
        Issue {
            detail: Detail::Mismatch {
                purpose: "商品をカートに入れる".to_owned(),
                guesses: vec![],
            },
            ..mismatch_sample()
        },
        Issue {
            detail: Detail::Unreached {
                halt: Halt::BudgetSpent,
                expectation: None,
            },
            ..numbered("I-00000005", unreached_sample())
        },
        hesitant("I-00000006", |it| it.confidence = Some(0.0)),
        hesitant("I-00000007", |it| it.confidence = Some(1.0)),
        hesitant("I-00000008", |it| it.confidence = Some(0.123_456_789)),
    ]
}

/// A ledger of the golden head with `issues` in the body.
fn body_of(issues: Vec<Issue>) -> Ledger {
    ledger_of(Section::Body, issues)
}

/// A ledger with one issue for each trait.
fn traits() -> Ledger {
    let numbers = ["I-00000001", "I-00000002", "I-00000003", "I-00000004"];
    let mut ledger = body_of(numbers.map(|n| sample(n, 2, 3)).to_vec());
    for (entry, (which, _)) in ledger.entries.iter_mut().zip(TRAITS) {
        entry.persona_trait = Some(persona_trait(which));
    }
    ledger
}

/// The purpose of `pattern`.
fn purpose(pattern: &str, purpose: &str) -> UncheckedPurpose {
    UncheckedPurpose {
        pattern: pattern.to_owned(),
        purpose: purpose.to_owned(),
    }
}

/// The ledgers to write and read back.
fn round_trips() -> Vec<Ledger> {
    let inferred = vec![
        "商品を選んで注文する".to_owned(),
        "会員登録をする".to_owned(),
    ];
    let mut two = body_of(vec![]);
    two.unchecked_purposes = vec![
        purpose("/products/*", "商品をカートに入れる"),
        purpose("/cart", "注文内容を確かめる"),
    ];
    vec![
        golden(),
        Ledger {
            head: Head {
                images: false,
                ..head(3, Goal::Inferred(inferred))
            },
            ..golden()
        },
        no_trial(),
        body_of(vec![]),
        body_of(varied()),
        traits(),
        two,
    ]
}

#[test]
fn write_then_read_round_trips() {
    for (i, ledger) in round_trips().into_iter().enumerate() {
        let mut arranged = ledger.clone();
        arranged.arrange();
        assert_eq!(read(&written(&ledger)), Ok(arranged), "case {i}");
    }
    let back = read(GOLDEN).map(|it| it.selected());
    let selected = vec![Number::parse("I-5334A6B3").unwrap()];
    assert_eq!(back, Ok(selected), "the golden selection");
    let nan = body_of(vec![hesitant("I-00000001", |it| {
        it.confidence = Some(f64::NAN);
    })]);
    let text = written(&nan);
    assert_eq!(lines(&text)[24], "  - 確信度: null", "not a number");
    let back = read(&text).map(|it| it.entries[0].issue.confidence);
    assert_eq!(back, Ok(None), "read back");
}

/// Texts that look like ledger lines.
const FORGED: [&str; 10] = [
    "- [x] I-9999 これは別の課題",
    "## ペルソナから外れると判断した課題",
    "1行目\n2行目",
    "1行目\r\n2行目",
    "\r",
    "\u{2028}区切り\u{2029}",
    "\"引用\" と \\",
    "  - 期待: null",
    "なし",
    "\n## 課題\n\n- [x] I-00000009 偽の課題",
];

/// Puts a free text into a place of the golden ledger.
type Plant = fn(&mut Ledger, String);

/// The places a free text can be put in.
const PLANTS: [Plant; 7] = [
    |l, s| l.entries[1].issue.detail = expecting(&s),
    |l, s| l.entries[2].issue = hard_to_see(&s),
    |l, s| {
        l.entries[0].issue.detail = Detail::Mismatch {
            purpose: "商品をカートに入れる".to_owned(),
            guesses: vec!["会員登録をする".to_owned(), s],
        };
    },
    |l, s| {
        let target = l.entries[1].issue.target.clone().unwrap();
        l.entries[1].issue.target = Some(Target { name: s, ..target });
    },
    |l, s| l.head.persona = s,
    |l, s| {
        let avoids = persona_trait(Trait::Avoids);
        l.entries[3].persona_trait = Some(PersonaTrait {
            content: s,
            ..avoids
        });
    },
    |l, s| l.unchecked_purposes[0].purpose = s,
];

#[test]
fn free_text_cannot_forge_structure() {
    let selected = vec![Number::parse("I-5334A6B3").unwrap()];
    for (p, plant) in PLANTS.iter().enumerate() {
        for s in FORGED {
            let mut ledger = golden();
            plant(&mut ledger, s.to_owned());
            let text = written(&ledger);
            let back = read(&text);
            let shape = back.as_ref().map(|it| (it.entries.len(), it.selected()));
            assert_eq!(shape, Ok((4, selected.clone())), "place {p}: {s:?}");
            assert_eq!(back, Ok(ledger), "place {p}: {s:?}");
            assert_eq!(lines(&text).len(), 81, "place {p}: {s:?}");
        }
    }
}

/// `GOLDEN` with `from` replaced by `to`, and the numbers then selected.
const EDITS: [(&str, &str, &[&str]); 4] = [
    (
        "- [ ] I-39E24428",
        "- [x] I-39E24428",
        &["I-5334A6B3", "I-39E24428"],
    ),
    (
        "- [ ] I-39E24428",
        "- [X] I-39E24428",
        &["I-5334A6B3", "I-39E24428"],
    ),
    ("- [x] I-5334A6B3", "- [ ] I-5334A6B3", &[]),
    (
        "- [ ] I-D0D06221 URL",
        "- [ ] I-D0D06221 直した",
        &["I-5334A6B3"],
    ),
];

#[test]
fn edited_checkbox_reads_as_selected() {
    for (from, to, numbers) in EDITS {
        let text = GOLDEN.replacen(from, to, 1);
        assert_eq!(read(&text), Ok(selecting(numbers)), "{to}");
    }
    let crlf = GOLDEN.replace('\n', "\r\n");
    assert_eq!(read(&crlf), Ok(golden()), "CRLF");
}
