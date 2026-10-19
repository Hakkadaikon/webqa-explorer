//! The golden test of the ledger text, with a ledger built from the
//! public types only.

use webqa_core::config::raw::Op;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{
    Act, Count, Detail, Halt, Issue, Number, Place, ScreenMark, Shot, Target, UncheckedPurpose,
};
use webqa_core::interview::ledger::{Compared, Entry, Goal, Head, Ledger, read, write};
use webqa_core::interview::natural::{PersonaTrait, Placed, Section, Trait};
use webqa_core::redact::Redactor;
use webqa_core::screen::Rect;

use super::persona;

const LEDGER: &str = include_str!("../fixtures/interview/ledger.txt");

/// The element `role "name"` at `y`, 10 by 10 px.
fn element(role: &str, name: &str, y: i64) -> Target {
    let rect = Rect {
        x: 0,
        y,
        width: 10,
        height: 10,
    };
    Target {
        role: role.to_owned(),
        name: name.to_owned(),
        nth: 0,
        rect: Some(rect),
    }
}

/// The issue numbered `number` about `target` at the shop's `path`
/// marked `mark` (or at the pattern `path` when `mark` is not one), first
/// at `step` of `trial`, happening in `occurred` of `finished` trials
/// and reaching `reached`.
fn issue(
    [number, path, mark]: [&str; 3],
    target: Option<Target>,
    detail: Detail,
    [occurred, finished, reached, trial, step]: [usize; 5],
) -> Option<Issue> {
    Some(Issue {
        number: Number::parse(number)?,
        place: ScreenMark::parse(mark).map_or_else(
            || Place::Pattern(path.to_owned()),
            |mark| Place::Screen {
                url: format!("https://shop.example.test{path}"),
                mark,
            },
        ),
        target,
        detail,
        count: Count {
            occurred,
            finished,
            reached,
        },
        shot: Some(Shot { trial, step }),
        path: vec![],
        confidence: None,
    })
}

/// A click on `link "カート"`.
fn click() -> Act {
    Act {
        op: Op::Click,
        target: Some(element("link", "カート", 0)),
    }
}

/// The issues of the golden body: the mismatch and the hesitation.
fn body() -> Option<[Issue; 2]> {
    let mismatch = Detail::Mismatch {
        purpose: "商品をカートに入れる".to_owned(),
        guesses: vec![
            "会員登録をする".to_owned(),
            "商品をカートに入れる".to_owned(),
        ],
    };
    let hesitant = Detail::Hesitant {
        clues: vec![Clue::Hesitant, Clue::WentBack],
        expectation: "押すと注文が確定すると思った".to_owned(),
    };
    let next = Some(element("button", "次へ", 0));
    let confirm = ["I-5334A6B3", "/confirm", "S-B9520D77"];
    Some([
        issue(
            ["I-D0D06221", "/products/*", ""],
            None,
            mismatch,
            [2, 2, 2, 1, 4],
        )?,
        Issue {
            path: vec![click()],
            confidence: Some(0.4),
            ..issue(confirm, next, hesitant, [2, 3, 3, 1, 2])?
        },
    ])
}

/// The issues at the shipping screen: the visibility issue and the
/// unreached one.
fn shipping() -> Option<[Issue; 2]> {
    let visibility = Detail::Visibility {
        reason: "文字が薄い".to_owned(),
        expectation: "クーポンを入力できると思った".to_owned(),
    };
    let unreached = Detail::Unreached {
        halt: Halt::GaveUp,
        expectation: Some("次に進むボタンが見つからない".to_owned()),
    };
    let coupon = Some(element("button", "クーポンを使う", 20));
    let back = Act {
        op: Op::Back,
        target: None,
    };
    let at = |number| [number, "/shipping", "S-F0029AF7"];
    Some([
        issue(at("I-39E24428"), coupon, visibility, [1, 3, 3, 2, 1])?,
        Issue {
            path: vec![click(), back],
            ..issue(at("I-47871971"), None, unreached, [1, 3, 2, 3, 5])?
        },
    ])
}

/// The golden ledger with `entries`.
fn golden_ledger(entries: Vec<Entry>) -> Ledger {
    let head = Head {
        persona: persona().name,
        goal: Goal::Written("注文を確定できる画面まで進む".to_owned()),
        start: "https://shop.example.test/".to_owned(),
        finished: 3,
        images: true,
    };
    let unchecked_purposes = vec![UncheckedPurpose {
        pattern: "/cart".to_owned(),
        purpose: "注文内容を確かめる".to_owned(),
    }];
    Ledger {
        head,
        entries,
        unchecked_purposes,
    }
}

#[test]
fn ledger_snapshot() {
    let [mismatch, hesitant] = body().unwrap();
    let [visibility, unreached] = shipping().unwrap();
    let entry = |issue, section, which: Option<Trait>| {
        let persona_trait = which.map(|which| PersonaTrait {
            which,
            content: which.content(&persona()),
        });
        Entry::from(Placed {
            issue,
            section,
            persona_trait,
        })
    };
    let ledger = golden_ledger(vec![
        entry(mismatch, Section::Body, Some(Trait::Wants)),
        Entry {
            selected: true,
            compared: Compared::Remaining,
            ..entry(hesitant, Section::Body, Some(Trait::ItLiteracy))
        },
        entry(visibility, Section::Unchecked, None),
        entry(unreached, Section::OffPersona, Some(Trait::Avoids)),
    ]);
    let text = write(&ledger, &Redactor::default());
    assert_eq!(text, LEDGER, "the ledger text");
    assert_eq!(read(LEDGER), Ok(ledger), "read back");
}
