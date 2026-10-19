//! Tests of what the run records for each operation, and of the run
//! writing nothing once a write failed.

use std::time::Duration;

use webqa_core::config::raw::Op;
use webqa_core::machine::Cause;
use webqa_core::screen::ElementId;

use crate::record::Act;

use super::super::fakes;
use super::super::*;
use super::{Fakes, limited, script, url};

fn act(op: &str, target: Option<ElementId>, p: f64, cause: Cause) -> Act {
    Act {
        step: 1,
        op: op.to_owned(),
        target,
        p,
        cause,
        act: Duration::ZERO,
        settle: Duration::ZERO,
    }
}

#[tokio::test]
async fn act_keeps_the_judge_probability_and_the_target() {
    let mut fakes = Fakes::new();
    fakes.judge.script = script(&[("click", 0.7, "2")]);
    let _ = fakes.explore().await;
    let link = ElementId {
        role: "link".to_owned(),
        name: "[[メールアドレス]] に送る".to_owned(),
        nth: 0,
    };
    let expected = [act("click", Some(link), 0.7, Cause::Judge)];
    assert_eq!(fakes.record.acts, expected, "the second element, redacted");
}

#[tokio::test]
async fn input_on_every_input_field_is_not_skipped() {
    for role in ["textbox", "searchbox", "combobox", "spinbutton"] {
        let mut fakes = Fakes::new();
        fakes.judge.script = script(&[("input", 0.9, "1")]);
        fakes.stay_on(&fakes::page(&url("/"), &[(role, "名前")]));
        fakes.rules = limited(&[Op::Click, Op::Input], 1, 10);
        let _ = fakes.explore().await;
        let kinds = ["asked", "decided", "executed"];
        assert_eq!(fakes.kinds(), kinds, "typed into a {role}, nothing skipped");
    }
}

#[tokio::test]
async fn act_without_a_target_keeps_the_probability() {
    let mut fakes = Fakes::new();
    fakes.judge.script = script(&[("scroll_down", 0.6, "1")]);
    fakes.rules = limited(&[Op::Click, Op::ScrollDown], 1, 10);
    let _ = fakes.explore().await;
    let expected = [act("scroll_down", None, 0.6, Cause::Judge)];
    assert_eq!(fakes.record.acts, expected, "no element");
}

#[tokio::test]
async fn nothing_is_written_after_a_failed_write() {
    for limit in 0..3 {
        let mut fakes = Fakes::new();
        fakes.store.limit = Some(limit);
        let mut ports = fakes.ports();
        let run = explore(
            &mut ports,
            &mut super::Never,
            super::rules(),
            Memory::default(),
        )
        .await;
        assert_eq!(run.map(|o| o.reason), Err(StoreError.into()), "{limit}");
        let again = ports.log(Event::Waited { until_ms: 0 }).await;
        assert_eq!(again, Err(StoreError.into()), "refused after {limit}");
        drop(ports);
        assert_eq!(fakes.store.writes, limit + 1, "stopped at {limit}");
    }
}

#[tokio::test]
async fn nothing_is_written_after_a_failed_screenshot() {
    let mut fakes = Fakes::new();
    fakes.stay_on(&fakes::page(&url("/"), &[("button", "icon_save.png")]));
    fakes.store.limit = Some(0);
    let mut ports = fakes.ports();
    let run = explore(
        &mut ports,
        &mut super::Never,
        super::rules(),
        Memory::default(),
    )
    .await;
    assert_eq!(
        run.map(|o| o.reason),
        Err(StoreError.into()),
        "the shot fails"
    );
    let again = ports.log(Event::Waited { until_ms: 0 }).await;
    assert_eq!(again, Err(StoreError.into()), "refused after the shot");
    drop(ports);
    assert_eq!(fakes.store.writes, 1, "only the screenshot was written");
}
