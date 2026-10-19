//! Tests of what is written at the end of a run.

use super::fakes::*;
use webqa_core::config::raw::{DoneWhen, Stage};

use super::*;

#[tokio::test]
async fn records_written_in_order() {
    let mut store = Slow::holding(Hold::Once);
    let ending = on_top(Wait::Step, &mut store).await;
    assert_eq!(exit::code(&ending), 0);
    let tail = &store.order[store.order.len() - 3..];
    assert_eq!(tail, ["write_memory", "write_event", "write_report"]);
    let last = json!({"kind": "run_finished", "reason": "user_aborted", "step": 3});
    assert_eq!(ends(&store.inner).1, Some(last));
}

#[tokio::test]
async fn second_interrupt_while_writing_abandons() {
    let gate = Arc::new(Notify::new());
    let mut store = Slow::holding(Hold::Forever(Arc::clone(&gate)));
    let ending = interrupted(Wait::Question(3), &top(), &gate, &mut store).await;
    assert_eq!(format!("{ending:?}"), "Abandoned");
    assert_eq!(exit::code(&ending), 3);
    assert_eq!(store.inner.report, None);
    let last = ends(&store.inner).1.unwrap();
    assert_ne!(last["kind"], "run_finished");
}

/// The rules of a scenario whose one stage is done on the first page.
fn done_at_start() -> Rules {
    let stage = Stage {
        name: "ログイン".to_owned(),
        goal: "ログインする".to_owned(),
        done_when: DoneWhen {
            url: Some("/".to_owned()),
            element: None,
            text: None,
        },
    };
    Rules {
        mode: Mode::Scenario,
        stages: vec![stage],
        ..rules(100)
    }
}

/// Runs the rules of [`done_at_start`] on `store`, interrupted once.
async fn finished_then_interrupted(store: &mut dyn Store) -> Ending {
    let mut browser = browser_on(&top());
    let mut user = Counting {
        calls: 0,
        fire_on: BTreeSet::from([1]),
    };
    let redactor = Redactor::default();
    let (mut judge, mut clock) = (RecordingJudge::default(), FakeClock::default());
    let mut ports = Ports::new(&mut browser, &mut judge, &mut clock, store, &redactor);
    conduct(&mut ports, &mut user, done_at_start(), &plan()).await
}

#[tokio::test]
async fn interrupt_after_finish_keeps_reason() {
    let mut store = Slow::holding(Hold::Once);
    let ending = finished_then_interrupted(&mut store).await;
    assert_eq!(exit::code(&ending), 0);
    assert_eq!(reason_of(ending), ExitReason::GoalAchieved);
    let last = json!({"kind": "run_finished", "reason": "goal_achieved"});
    assert_eq!(ends(&store.inner).1, Some(last));
    assert_eq!(store.inner.report.unwrap()["reason"], "goal_achieved");
}

#[tokio::test]
async fn ledger_goes_to_the_inner_store() {
    let mut store = Slow::holding(Hold::Once);
    let written = store.write_ledger("# ledger\n").await;
    let kept = (written, store.order, store.inner.ledger);
    let expected = (Ok(()), vec!["write_ledger"], Some("# ledger\n".to_owned()));
    assert_eq!(kept, expected, "named, then passed on");
}
