//! Sample issues, one of each kind, for the questions and the ledger,
//! and the issues the tests expect, built from them.

use super::fixture::{IN1, IN2, clicked, count, next1};
use crate::config::raw::Op;
use crate::interview::clue::Clue;
use crate::interview::issue::{
    Act, Count, Detail, Halt, Issue, Number, Place, ScreenMark, Shot, Target,
};

/// `link "カート"` at [`IN1`].
fn cart_link() -> Target {
    Target {
        role: "link".to_owned(),
        name: "カート".to_owned(),
        nth: 0,
        rect: Some(IN1),
    }
}

fn shipping_place() -> Place {
    Place::Screen {
        url: "https://shop.example.test/shipping".to_owned(),
        mark: ScreenMark::parse("S-F0029AF7").unwrap(),
    }
}

/// A hesitation issue at `button "次へ"` of `/confirm`, numbered `number`,
/// that happened in `occurred` of `finished` trials.
pub fn sample(number: &str, occurred: usize, finished: usize) -> Issue {
    Issue {
        number: Number::parse(number).unwrap(),
        place: Place::Screen {
            url: "https://shop.example.test/confirm".to_owned(),
            mark: ScreenMark::parse("S-B9520D77").unwrap(),
        },
        target: Some(next1()),
        detail: Detail::Hesitant {
            clues: vec![Clue::Hesitant, Clue::WentBack],
            expectation: "押すと注文が確定すると思った".to_owned(),
        },
        count: Count {
            occurred,
            finished,
            reached: finished,
        },
        shot: Some(Shot { trial: 1, step: 2 }),
        path: vec![Act {
            op: Op::Click,
            target: Some(cart_link()),
        }],
        confidence: Some(0.4),
    }
}

/// A visibility issue at `button "クーポンを使う"` of `/shipping`.
pub fn visibility_sample() -> Issue {
    Issue {
        number: Number::parse("I-39E24428").unwrap(),
        place: shipping_place(),
        target: Some(Target {
            role: "button".to_owned(),
            name: "クーポンを使う".to_owned(),
            nth: 0,
            rect: Some(IN2),
        }),
        detail: Detail::Visibility {
            reason: "文字が薄い".to_owned(),
            expectation: "クーポンを入力できると思った".to_owned(),
        },
        count: Count {
            occurred: 1,
            finished: 3,
            reached: 3,
        },
        shot: Some(Shot { trial: 2, step: 1 }),
        path: vec![],
        confidence: None,
    }
}

/// An unreached issue at `/shipping`.
pub fn unreached_sample() -> Issue {
    Issue {
        number: Number::parse("I-47871971").unwrap(),
        place: shipping_place(),
        target: None,
        detail: Detail::Unreached {
            halt: Halt::GaveUp,
            expectation: Some("次に進むボタンが見つからない".to_owned()),
        },
        count: Count {
            occurred: 1,
            finished: 3,
            reached: 2,
        },
        shot: Some(Shot { trial: 3, step: 5 }),
        path: vec![
            Act {
                op: Op::Click,
                target: Some(cart_link()),
            },
            Act {
                op: Op::Back,
                target: None,
            },
        ],
        confidence: None,
    }
}

/// A mismatch issue of `/products/*`.
pub fn mismatch_sample() -> Issue {
    Issue {
        number: Number::parse("I-D0D06221").unwrap(),
        place: Place::Pattern("/products/*".to_owned()),
        target: None,
        detail: Detail::Mismatch {
            purpose: "商品をカートに入れる".to_owned(),
            guesses: vec![
                "会員登録をする".to_owned(),
                "商品をカートに入れる".to_owned(),
            ],
        },
        count: Count {
            occurred: 2,
            finished: 2,
            reached: 2,
        },
        shot: Some(Shot { trial: 1, step: 4 }),
        path: vec![],
        confidence: None,
    }
}

/// What the persona expected in the answers of the fixture.
pub const EXPECTED: &str = "押すと注文が確定すると思った";

/// The hesitation with `clues` and `expectation` at `button "次へ"` of
/// `/confirm`, happening as `count`, first at `shot` after `path`.
pub fn hesitant_at_confirm(
    clues: &[Clue],
    expectation: &str,
    count: Count,
    shot: Shot,
    path: Vec<Act>,
) -> Issue {
    Issue {
        detail: Detail::Hesitant {
            clues: clues.to_vec(),
            expectation: expectation.to_owned(),
        },
        count,
        shot: Some(shot),
        path,
        ..sample("I-5334A6B3", 0, 0)
    }
}

/// The unreached issue of a trial that stopped as `halt` at step 2, at
/// `/shipping`, after a click on `button "次へ"` of `/cart`.
pub fn unreached_at_shipping(halt: Halt, expectation: Option<&str>) -> Issue {
    Issue {
        detail: Detail::Unreached {
            halt,
            expectation: expectation.map(str::to_owned),
        },
        count: count(1, 1, 1),
        shot: Some(Shot { trial: 1, step: 2 }),
        path: clicked(),
        ..unreached_sample()
    }
}

/// The unreached issue of a trial that made no progress at `/cart`.
pub fn unreached_at_cart() -> Issue {
    Issue {
        number: Number::parse("I-0A458E95").unwrap(),
        place: Place::Screen {
            url: "https://shop.example.test/cart".to_owned(),
            mark: ScreenMark::parse("S-E6F64D7B").unwrap(),
        },
        detail: Detail::Unreached {
            halt: Halt::NoProgress,
            expectation: Some(EXPECTED.to_owned()),
        },
        count: count(1, 1, 1),
        shot: Some(Shot { trial: 1, step: 1 }),
        path: vec![],
        ..unreached_sample()
    }
}

/// The visibility sample, happening at the first step of its one trial.
pub fn visibility_once() -> Issue {
    Issue {
        count: count(1, 1, 1),
        shot: Some(Shot { trial: 1, step: 1 }),
        ..visibility_sample()
    }
}

/// The mismatch issue of `/products/*` asked with `guesses`, happening as
/// `count`, first at `shot` after `path`.
pub fn mismatch_of(guesses: &[&str], count: Count, shot: Shot, path: Vec<Act>) -> Issue {
    Issue {
        detail: Detail::Mismatch {
            purpose: "商品をカートに入れる".to_owned(),
            guesses: guesses.iter().map(|&guess| guess.to_owned()).collect(),
        },
        count,
        shot: Some(shot),
        path,
        ..mismatch_sample()
    }
}
