//! Screens, answers and records for the issue tests.

use url::Url;

use crate::config::purpose::{ScreenPurpose, UrlPattern};
use crate::config::raw::Op;
use crate::interview::answer::{HardToSee, Reply};
use crate::interview::ask::{MismatchQuestion, PURPOSE_MATCH, Trial, Visit, mismatch_questions};
use crate::interview::clue::Clue;
use crate::interview::issue::{Act, Count, Ran, Record, Run, Seen, Target, visits};
use crate::interview::outcome::{Claim, Cut, Ending, Outcome, Stuck};
use crate::judge::{Answers, QuestionId, Usage, Verdict};
use crate::redact::{Redacted, Redactor};
use crate::screen::{Element, Rect, Screen, Viewport};

pub const VIEWPORT: Viewport = Viewport {
    width: 800,
    height: 600,
};
pub const IN1: Rect = Rect {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
};
pub const IN2: Rect = Rect {
    x: 0,
    y: 20,
    width: 10,
    height: 10,
};
pub const OUT: Rect = Rect {
    x: 0,
    y: 900,
    width: 10,
    height: 10,
};

pub const GOAL: Outcome = Outcome::Finished(Ending::Goal);
pub const GAVE_UP: Outcome = Outcome::Finished(Ending::Stuck(Stuck::GaveUp));
pub const NO_PROGRESS: Outcome = Outcome::Finished(Ending::Stuck(Stuck::NoProgress));
pub const CUT: Outcome = Outcome::Cut(Cut::Interrupted);
pub const BUDGET_SPENT: Outcome = Outcome::Finished(Ending::BudgetSpent);
pub const INFERRED: Outcome = Outcome::Finished(Ending::Claimed(Claim::Inferred));
pub const UNVERIFIED: Outcome = Outcome::Finished(Ending::Claimed(Claim::Unverified));

/// A screen at `https://shop.example.test{path}` with `(role, name, rect)`
/// elements and no values.
pub fn screen(path: &str, elements: &[(&str, &str, Rect)]) -> Redacted<Screen> {
    let url = Url::parse(&format!("https://shop.example.test{path}")).unwrap();
    let elements = elements
        .iter()
        .map(|&(role, name, rect)| Element {
            role: role.to_owned(),
            name: name.to_owned(),
            value: None,
            rect: Some(rect),
        })
        .collect();
    Redactor::default().screen(Screen { url, elements })
}

pub fn confirm() -> Redacted<Screen> {
    screen(
        "/confirm",
        &[("button", "次へ", IN1), ("link", "戻る", IN2)],
    )
}

pub fn cart() -> Redacted<Screen> {
    screen(
        "/cart",
        &[
            ("button", "次へ", IN1),
            ("link", "戻る", IN2),
            ("link", "フッター", OUT),
        ],
    )
}

pub fn shipping() -> Redacted<Screen> {
    screen(
        "/shipping",
        &[
            ("textbox", "郵便番号", IN1),
            ("button", "クーポンを使う", IN2),
        ],
    )
}

pub fn product(id: u32) -> Redacted<Screen> {
    screen(
        &format!("/products/{id}"),
        &[("button", "カートに入れる", IN1)],
    )
}

/// Answers choosing `op` on element `target` with `confidence`.
pub fn reply(op: Op, target: Option<usize>, confidence: Option<f64>) -> Reply {
    Reply {
        verdict: Verdict {
            op,
            confidence,
            targets: target.into_iter().collect(),
            error_shown: Some(0.0),
            goal_reached: Some(0.0),
            usage: Usage::default(),
        },
        hesitation: 0.1,
        expectation: "押すと注文が確定すると思った".to_owned(),
        hard_to_see: None,
        purpose_guess: "商品をカートに入れる".to_owned(),
    }
}

/// A click on element 1 with confidence 0.9.
pub fn sure() -> Reply {
    reply(Op::Click, Some(1), Some(0.9))
}

/// A click on element 1 with confidence 0.4.
pub fn unsure() -> Reply {
    reply(Op::Click, Some(1), Some(0.4))
}

/// `answers` with the expectation `text`.
pub fn expecting(text: &str, answers: Reply) -> Reply {
    Reply {
        expectation: text.to_owned(),
        ..answers
    }
}

/// Answers guessing the purpose `guess`.
pub fn guessing(guess: &str) -> Reply {
    Reply {
        purpose_guess: guess.to_owned(),
        ..sure()
    }
}

/// Answers at shipping expecting a coupon field, listing `listed` as
/// `(number, reason)`.
pub fn listing(listed: Option<&[(usize, &str)]>) -> Reply {
    let hard_to_see = listed.map(|listed| {
        listed
            .iter()
            .map(|&(element, reason)| HardToSee {
                element,
                reason: reason.to_owned(),
            })
            .collect()
    });
    let answers = Reply {
        hard_to_see,
        ..reply(Op::Click, Some(2), Some(0.9))
    };
    expecting("クーポンを入力できると思った", answers)
}

/// Answers with `table` for the mismatch question, or none.
pub fn purpose_match(table: Option<&[(&str, f64)]>) -> Answers {
    let rows = table.map(|table| {
        let options = table.iter().map(|&(o, p)| (o.to_owned(), p)).collect();
        (QuestionId::new(PURPOSE_MATCH), options)
    });
    Answers {
        table: rows.into_iter().collect(),
        usage: Usage::default(),
    }
}

/// The mismatch questions of `screens` over `records`.
pub fn questions<'s>(
    screens: &'s [ScreenPurpose],
    records: &[Record<'_>],
) -> Vec<MismatchQuestion<'s>> {
    let visits: Vec<Vec<Visit<'_>>> = records.iter().map(visits).collect();
    let trials: Vec<Trial<'_>> = records
        .iter()
        .zip(&visits)
        .map(|(record, visits)| Trial {
            outcome: record.outcome,
            visits,
        })
        .collect();
    mismatch_questions(screens, &trials)
}

/// The screen purpose of `/products/*`.
pub fn products() -> ScreenPurpose {
    ScreenPurpose {
        pattern: UrlPattern::parse("/products/*").unwrap(),
        purpose: "商品をカートに入れる".to_owned(),
    }
}

/// The screen purpose of `/cart`.
pub fn cart_purpose() -> ScreenPurpose {
    ScreenPurpose {
        pattern: UrlPattern::parse("/cart").unwrap(),
        purpose: "注文内容を確かめる".to_owned(),
    }
}

/// A click on element 1.
pub const CLICK1: Option<Ran> = Some(Ran {
    op: Op::Click,
    target: Some(1),
});

/// A step on `screen` in [`VIEWPORT`].
pub const fn seen<'a>(
    screen: &'a Redacted<Screen>,
    reply: Option<&'a Reply>,
    clues: &'a [Clue],
    ran: Option<Ran>,
) -> Seen<'a> {
    Seen {
        screen,
        viewport: VIEWPORT,
        reply,
        clues,
        ran,
    }
}

/// A trial ending as `outcome` after `steps`.
pub const fn record<'a>(outcome: Outcome, steps: &'a [Seen<'a>]) -> Record<'a> {
    Record { outcome, steps }
}

/// A step on `screen` with `answers`, no clue, and nothing run.
pub const fn answered<'a>(screen: &'a Redacted<Screen>, answers: &'a Reply) -> Seen<'a> {
    seen(screen, Some(answers), &[], None)
}

/// The run of `trials`, with a judge that reads images.
pub const fn run<'a>(trials: &'a [Record<'a>]) -> Run<'a> {
    Run {
        trials,
        images: true,
    }
}

/// `button "次へ"` at [`IN1`].
pub fn next1() -> Target {
    Target {
        role: "button".to_owned(),
        name: "次へ".to_owned(),
        nth: 0,
        rect: Some(IN1),
    }
}

/// The path of a click on [`next1`].
pub fn clicked() -> Vec<Act> {
    vec![Act {
        op: Op::Click,
        target: Some(next1()),
    }]
}

/// How often an issue happened.
pub const fn count(occurred: usize, finished: usize, reached: usize) -> Count {
    Count {
        occurred,
        finished,
        reached,
    }
}
