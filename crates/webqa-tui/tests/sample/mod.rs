//! The ledger the tests of the ledger screen and file are built from.

use webqa_core::config::raw::Op;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{Act, Count, Detail, Issue, Number, Place, ScreenMark, Target};
use webqa_core::interview::ledger::{Compared, Entry, Goal, Head, Ledger};
use webqa_core::interview::natural::{PersonaTrait, Section, Trait};

/// The number `I-0000000{n}`.
pub fn number(n: u8) -> Option<Number> {
    Number::parse(&format!("I-0000000{n}"))
}

/// An element.
pub fn target(role: &str, name: &str) -> Target {
    Target {
        role: role.to_owned(),
        name: name.to_owned(),
        nth: 0,
        rect: None,
    }
}

/// An issue of the cart screen with no path, shot or confidence.
pub fn issue(n: u8, element: Target, detail: Detail) -> Option<Issue> {
    Some(Issue {
        number: number(n)?,
        place: Place::Screen {
            url: "https://shop.example.test/cart".to_owned(),
            mark: ScreenMark::parse("S-00000001")?,
        },
        target: Some(element),
        detail,
        count: Count {
            occurred: 1,
            finished: 2,
            reached: 2,
        },
        shot: None,
        path: Vec::new(),
        confidence: None,
    })
}

/// An unselected, new entry in the body.
pub const fn entry(issue: Issue, persona_trait: Option<PersonaTrait>) -> Entry {
    Entry {
        selected: false,
        section: Section::Body,
        issue,
        persona_trait,
        compared: Compared::New,
    }
}

fn hesitant(clues: Vec<Clue>, expectation: &str) -> Detail {
    Detail::Hesitant {
        clues,
        expectation: expectation.to_owned(),
    }
}

/// Issue 1: hesitant, reached by clicking the link.
fn first() -> Option<Entry> {
    let clues = vec![Clue::Hesitant, Clue::WentBack];
    let mut issue = issue(1, target("link", "注文"), hesitant(clues, "注文が確定する"))?;
    issue.path = vec![Act {
        op: Op::Click,
        target: Some(target("link", "注文")),
    }];
    Some(entry(issue, None))
}

/// Issue 2: hesitant.
fn second() -> Option<Entry> {
    let detail = hesitant(vec![Clue::LowConfidence], "商品が消える");
    Some(entry(issue(2, target("button", "削除"), detail)?, None))
}

/// Issue 3: hard to see, from a persona who is used to phones.
fn third() -> Option<Entry> {
    let detail = Detail::Visibility {
        reason: "文字が小さい".to_owned(),
        expectation: "確定できる".to_owned(),
    };
    let persona_trait = PersonaTrait {
        which: Trait::ItLiteracy,
        content: "スマホは慣れている".to_owned(),
    };
    Some(entry(
        issue(3, target("button", "確定"), detail)?,
        Some(persona_trait),
    ))
}

/// Three issues: two hesitant (1, 2) and one hard to see (3).
pub fn sample() -> Option<Ledger> {
    Some(Ledger {
        head: Head {
            persona: "佐藤さん".to_owned(),
            goal: Goal::Written("注文を確定する".to_owned()),
            start: "https://shop.example.test/".to_owned(),
            finished: 2,
            images: true,
        },
        entries: vec![first()?, second()?, third()?],
        unchecked_purposes: Vec::new(),
    })
}
