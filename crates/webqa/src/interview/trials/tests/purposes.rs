//! Tests of the questions about the screen purposes after the trials.

use webqa_core::config::purpose::{self, Purpose};
use webqa_core::interview::ask::{DIFFERENT, PURPOSE_MATCH, SAME};
use webqa_core::interview::issue::{Kind, Place, Record, Run, Seen, Shot, UncheckedPurpose};
use webqa_core::interview::natural::Section;
use webqa_core::interview::outcome::{self, Outcome};
use webqa_core::redact::Redacted;
use webqa_core::screen::{Screen, screen_text};

use super::super::after::shot_text;
use super::findings::{ONCE, entries};
use super::*;

/// The written purpose with screen purposes for `/` and `/items`.
fn written() -> Purpose {
    purpose::parse(
        "goal: 経費を申請する
done_when:
  url: /done
screens:
  - url: /
    purpose: 経費の一覧を見る
  - url: /items
    purpose: 経費を選ぶ
",
    )
    .unwrap()
}

/// A browser that shows the top page, then the page of the items after the
/// click.
fn through_items() -> FakeBrowser {
    let url = Url::parse("https://app.example.test/items").unwrap();
    let items = fakes::page(&url, &[("button", "保存")]);
    let pages = [top(), top()].into_iter().chain(iter::repeat_n(items, 20));
    FakeBrowser {
        pages: pages.collect(),
        ..FakeBrowser::default()
    }
}

/// A world of one trial towards the written purpose, in which the judge
/// answers `matched` to each question about the screen purposes.
pub(super) fn purposeful(matched: &[&str]) -> World {
    let mut world = World::new(1);
    world.plan.interview.purpose = written();
    let script = hesitating(&["click", "done"], 0.0);
    world.judge.script = matched.iter().fold(script, |script, answer| {
        script.then(QuestionId::new(PURPOSE_MATCH), [(*answer, 1.0)])
    });
    world.launch.browsers = VecDeque::from([through_items()]);
    world
}

#[tokio::test]
async fn different_guess_is_a_mismatch_issue() {
    let mut world = purposeful(&[DIFFERENT, SAME]);
    world.run().await;
    let ledger = world.ledger().unwrap();
    let places: Vec<_> = ledger
        .entries
        .iter()
        .map(|e| e.issue.place.clone())
        .collect();
    assert_eq!(
        (entries(&world), places, ledger.unchecked_purposes),
        (
            vec![(Kind::Mismatch, Section::Unchecked, ONCE)],
            vec![Place::Pattern("/".to_owned())],
            Vec::new()
        ),
        "the screen purpose the judge understood otherwise"
    );
}

#[tokio::test]
async fn unusable_answer_leaves_the_purpose_unchecked() {
    let mut world = purposeful(&[]);
    world.run().await;
    let unchecked = |pattern: &str, purpose: &str| UncheckedPurpose {
        pattern: pattern.to_owned(),
        purpose: purpose.to_owned(),
    };
    assert_eq!(
        (entries(&world), world.ledger().unwrap().unchecked_purposes),
        (
            Vec::new(),
            vec![
                unchecked("/", "経費の一覧を見る"),
                unchecked("/items", "経費を選ぶ")
            ]
        ),
        "both purposes could not be checked"
    );
}

#[tokio::test]
async fn mismatch_questions_leave_trial_budget() {
    let mut world = purposeful(&[SAME, SAME]);
    world.run().await;
    let report = world.store.report.unwrap();
    let calls = report["calls"].as_array().unwrap();
    let steps: Vec<_> = calls.iter().filter_map(|c| c["step"].as_u64()).collect();
    assert_eq!(
        steps,
        [1, 1, 2, 0, 0],
        "the two calls of the click, the done, then the two questions about the screens"
    );
}

#[tokio::test]
async fn questions_over_the_cap_are_unchecked() {
    let mut world = purposeful(&[SAME, SAME]);
    world.plan.interview.max_natural_questions = nz(1);
    world.run().await;
    let report = world.store.report.clone().unwrap();
    let asked = report["calls"].as_array().map(Vec::len);
    assert_eq!(
        (
            asked,
            world.ledger().unwrap().unchecked_purposes.len(),
            entries(&world)
        ),
        (Some(4), 1, Vec::new()),
        "three calls for two decisions, one question within the cap; the other purpose is unchecked"
    );
}

/// The text of the page of `screen`, as the judge reads it.
fn read(screen: &Redacted<Screen>) -> String {
    let text = screen_text(screen, top().viewport);
    text.as_inner().as_str().to_owned()
}

/// The text `shot_text` finds for `step` of the one trial made of `seen`.
fn texted(seen: &[Seen<'_>], step: u64) -> Option<String> {
    let outcome = Outcome::Finished(outcome::Ending::Goal);
    let record = Record {
        outcome,
        steps: seen,
    };
    let run = Run {
        trials: &[record],
        images: false,
    };
    let text = shot_text(&run, Shot { trial: 1, step })?;
    Some(text.as_inner().as_str().to_owned())
}

#[test]
fn the_text_after_the_trials_is_the_page_of_the_log_step() {
    let url = Url::parse(ORIGIN).unwrap();
    let screens = ["保存", "送信"].map(|name| {
        let page = fakes::page(&url, &[("button", name)]);
        Redactor::default().screen(page.screen)
    });
    // The core's own recovery was step 2, so the second question is step 3.
    let seen = [(1, 0), (3, 1)].map(|(step, i)| Seen {
        step,
        screen: &screens[i],
        viewport: top().viewport,
        reply: None,
        clues: &[],
        ran: None,
    });
    let got = [1, 2, 3].map(|step| texted(&seen, step));
    assert_eq!(
        got,
        [Some(read(&screens[0])), None, Some(read(&screens[1]))],
        "the log's step picks the page; step 2 asked no question"
    );
}
