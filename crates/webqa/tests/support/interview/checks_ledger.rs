//! Check of the ledger reader: the numbers of the issues read back.

use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{
    Count, Detail, Issue, Kind, Number, Place, ScreenMark, Shot, Target,
};
use webqa_core::interview::ledger::{Compared, Entry, Goal, Head, Ledger, write};
use webqa_core::interview::natural::Section;
use webqa_core::redact::Redactor;
use webqa_core::screen::ScreenId;

use super::ORIGIN;
use crate::ledger::{number_of, read_back};
use crate::scene::{Scene, found};

/// A ledger of one hesitation issue, with the number `number`.
fn sample_ledger(number: Number) -> Ledger {
    let screen = ScreenId {
        path: "/cart".to_owned(),
        elements: [("button".to_owned(), "次へ".to_owned())].into(),
    };
    let issue = Issue {
        number,
        place: Place::Screen {
            url: format!("{ORIGIN}/cart"),
            mark: ScreenMark::of(&screen),
        },
        target: Some(Target {
            role: "button".to_owned(),
            name: "次へ".to_owned(),
            nth: 0,
            rect: None,
        }),
        detail: Detail::Hesitant {
            clues: vec![Clue::Hesitant],
            expectation: "次の画面に進む".to_owned(),
        },
        count: Count {
            occurred: 1,
            finished: 1,
            reached: 1,
        },
        shot: Some(Shot { trial: 1, step: 1 }),
        path: vec![],
        confidence: Some(0.3),
    };
    Ledger {
        head: Head {
            persona: "佐藤さん".to_owned(),
            goal: Goal::Inferred(vec![]),
            start: format!("{ORIGIN}/"),
            finished: 1,
            images: false,
        },
        entries: vec![Entry {
            selected: false,
            section: Section::Unchecked,
            issue,
            persona_trait: None,
            compared: Compared::New,
        }],
        unchecked_purposes: vec![],
    }
}

/// A ledger written and read back keeps the numbers of its issues: `I-` and
/// 8 hex digits.
pub fn check() {
    let spot = ("button", "次へ", 0);
    let number = number_of("/cart", &[("button", "次へ")], Some(spot), Kind::Hesitant);
    let text = write(&sample_ledger(number), &Redactor::default());
    let read = read_back(&text);
    let written = found(read.entries.first(), "one issue")
        .issue
        .number
        .to_string();
    let hex = written.strip_prefix("I-").unwrap_or_default();
    let digits = hex.len() == 8 && hex.chars().all(|c| matches!(c, '0'..='9' | 'A'..='F'));
    assert!(digits, "I- and 8 hex digits: {written}");
    assert_eq!(Number::parse(&written), Some(number), "the number parses");
    check_alias(number, &written);
}

/// An example number of the feature stands for the number of the product.
fn check_alias(number: Number, written: &str) {
    let mut scene = Scene::default();
    scene.aliases.bind("I-0001", number);
    let aliases = &scene.aliases;
    assert_eq!(aliases.written("I-0001"), written, "the example stands");
    assert_eq!(aliases.number("I-0001"), number, "and is read back");
}
