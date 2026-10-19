//! Tests of the issue numbers and the screen marks. The expected numbers
//! were computed apart from this code, by the steps the numbers follow.

use super::*;
use crate::interview::issue::tests::fixture::{IN1, IN2, OUT, cart, confirm, screen, shipping};
use crate::redact::{Redacted, Redactor};
use crate::screen::{Screen, element_ids, screen_id};

fn number(text: &str) -> Number {
    Number::parse(text).unwrap()
}

/// The number of the `nth` element of `screen` (in page order) and `kind`.
fn element_number(screen: &Redacted<Screen>, nth: usize, kind: Kind) -> Number {
    let id = screen_id(screen);
    let element = &element_ids(screen)[nth];
    Number::of(Key {
        spot: Spot::Screen {
            screen: &id,
            element: Some(element),
        },
        kind,
    })
}

/// The number of `screen` without an element and `kind`.
fn screen_number(screen: &Redacted<Screen>, kind: Kind) -> Number {
    Number::of(Key {
        spot: Spot::Screen {
            screen: &screen_id(screen),
            element: None,
        },
        kind,
    })
}

fn pattern_number(pattern: &str) -> Number {
    Number::of(Key {
        spot: Spot::Pattern(pattern),
        kind: Kind::Mismatch,
    })
}

/// `confirm()` with a value in its first element.
fn confirm_with_value() -> Redacted<Screen> {
    let mut page = confirm().into_inner();
    page.elements[0].value = Some("入力".to_owned());
    Redactor::default().screen(page)
}

/// Screens rebuilt from `confirm()` that keep its identity.
fn rebuilt() -> [Redacted<Screen>; 4] {
    [
        confirm(),
        screen(
            "/confirm?from=mail",
            &[("button", "次へ", IN1), ("link", "戻る", IN2)],
        ),
        screen(
            "/confirm",
            &[("button", "次へ", OUT), ("link", "戻る", IN1)],
        ),
        confirm_with_value(),
    ]
}

#[test]
fn same_screen_element_kind_same_number() {
    for (i, page) in rebuilt().iter().enumerate() {
        let got = element_number(page, 0, Kind::Hesitant).to_string();
        assert_eq!(got, "I-5334A6B3", "case {i}");
    }
    let two = screen(
        "/confirm",
        &[("button", "次へ 2", IN1), ("link", "戻る", IN2)],
    );
    let three = screen(
        "/confirm",
        &[("button", "次へ 3", IN1), ("link", "戻る", IN2)],
    );
    assert_eq!(
        element_number(&two, 0, Kind::Hesitant),
        element_number(&three, 0, Kind::Hesitant),
        "only the digits differ"
    );
    let page = confirm();
    assert_eq!(
        element_number(&page, 0, Kind::Hesitant),
        element_number(&page, 0, Kind::Hesitant),
        "called twice"
    );
    let mark = ScreenMark::of(&screen_id(&confirm())).to_string();
    assert_eq!(mark, "S-B9520D77", "the mark of confirm");
}

/// Makes a screen.
type Page = fn() -> Redacted<Screen>;

/// A screen, an element `(role, name, nth)`, a kind and the number.
type ElementRow = (
    Page,
    (&'static str, &'static str, usize),
    Kind,
    &'static str,
);

const DIFFERENT: [ElementRow; 4] = [
    (cart, ("button", "次へ", 0), Kind::Hesitant, "I-83731A80"),
    (confirm, ("link", "戻る", 0), Kind::Hesitant, "I-AB800023"),
    (
        confirm,
        ("button", "次へ", 0),
        Kind::Visibility,
        "I-28ED8923",
    ),
    (confirm, ("button", "次へ", 1), Kind::Hesitant, "I-80E361A4"),
];

fn row_number((page, (role, name, nth), kind, _): ElementRow) -> Number {
    let element = ElementId {
        role: role.to_owned(),
        name: name.to_owned(),
        nth,
    };
    Number::of(Key {
        spot: Spot::Screen {
            screen: &screen_id(&page()),
            element: Some(&element),
        },
        kind,
    })
}

#[test]
fn different_key_different_number() {
    let got: Vec<String> = DIFFERENT.map(row_number).map(|n| n.to_string()).into();
    let expected: Vec<String> = DIFFERENT.map(|row| row.3.to_owned()).into();
    assert_eq!(got, expected, "the numbers of the table");
    let mut all = expected;
    all.push("I-5334A6B3".to_owned());
    all.sort();
    all.dedup();
    assert_eq!(all.len(), 5, "all differ: {all:?}");
}

const ELEMENTLESS: [(Page, Kind, &str); 4] = [
    (shipping, Kind::Unreached, "I-47871971"),
    (shipping, Kind::Hesitant, "I-08D111E9"),
    (confirm, Kind::Unreached, "I-5341E412"),
    (cart, Kind::Unreached, "I-0A458E95"),
];

#[test]
fn elementless_number_from_screen_and_kind() {
    for (page, kind, expected) in ELEMENTLESS {
        let got = screen_number(&page(), kind).to_string();
        assert_eq!(got, expected, "{kind:?}");
    }
    let queried = screen(
        "/shipping?step=2",
        &[
            ("textbox", "郵便番号", IN1),
            ("button", "クーポンを使う", IN2),
        ],
    );
    let got = screen_number(&queried, Kind::Unreached);
    assert_eq!(got, number("I-47871971"), "the query is not used");
}

#[test]
fn mismatch_number_from_url_pattern_and_kind() {
    let got = ["/products/*", "/shipping", "/cart"].map(|p| pattern_number(p).to_string());
    assert_eq!(
        got,
        ["I-D0D06221", "I-3DDEA212", "I-6E3B87F7"],
        "the numbers of the patterns"
    );
}

/// Texts that are not numbers.
const NOT_NUMBERS: [&str; 8] = [
    "I-5334a6b3",
    "I-5334A6B",
    "I-5334A6B30",
    "5334A6B3",
    "I-5334A6BG",
    "I-+334A6B3",
    "",
    "S-5334A6B3",
];

#[test]
fn number_text_round_trips() {
    let text = Number::parse("I-5334A6B3").map(|n| n.to_string());
    assert_eq!(text, Some("I-5334A6B3".to_owned()), "a number");
    let text = ScreenMark::parse("S-B9520D77").map(|m| m.to_string());
    assert_eq!(text, Some("S-B9520D77".to_owned()), "a mark");
    for text in NOT_NUMBERS {
        assert_eq!(Number::parse(text), None, "{text}");
    }
    assert_eq!(ScreenMark::parse("I-5334A6B3"), None, "a number is no mark");
    let ordered = ["I-00000001", "I-0000000A", "I-00000010"].map(number);
    assert_eq!(
        (ordered[0] < ordered[1], ordered[1] < ordered[2]),
        (true, true),
        "{ordered:?}"
    );
}
