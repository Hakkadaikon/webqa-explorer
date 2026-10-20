//! Checks of the base the interview scenarios stand on: the judge, the
//! ledger reader and one whole run of the fakes.

use std::time::Duration;

use webqa::interview::trials::{Ending, Stopped};
use webqa_core::ports::{Browser, Clock};

use crate::fakes::{self, FakeBrowser, FakeClock, RecordingJudge};
use crate::page::PageSpec;
use crate::scene::Scene;
use crate::world::InterviewWorld;

#[path = "checks_judge.rs"]
mod judge_checks;
#[path = "checks_ledger.rs"]
mod ledger_checks;

/// The origin of the scenes of the checks.
const ORIGIN: &str = "https://shop.example.test";

/// One whole run of the fakes: the judge has nothing to say, so it waits,
/// the interrupt ends the trial, and the ledger of no finished trial is
/// written.
async fn check_run() {
    let scene = Scene {
        origins: vec![ORIGIN.to_owned()],
        trials: 2,
        ..Scene::default()
    };
    let mut world = InterviewWorld { scene, ran: None };
    world.run().await;
    let ran = world.ran();
    let ending = Some(Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    });
    assert_eq!(ran.ending, ending, "the interrupt ended the run");
    assert_eq!(
        (ran.code, ran.launched),
        (0, 1),
        "the code and the launches"
    );
    assert_eq!(ran.judge.asked.len(), 1, "the judge was asked once");
    assert_eq!(ran.config_error, None, "the configuration read");
    assert!(ran.touches.is_empty(), "the browser did nothing");
    assert_eq!(
        ran.clock.sleeps,
        Vec::<Duration>::new(),
        "the run did not wait for a window"
    );
    let finished = world.ledger().map(|ledger| ledger.head.finished);
    assert_eq!(finished, Some(0), "no trial finished");
    assert_eq!(
        world.kinds().last().map(String::as_str),
        Some("run_finished"),
        "the log ends with the end of the run"
    );
}

/// The reset command of a scene is the script of the fixtures.
fn check_reset() {
    let mut scene = Scene::default();
    scene.reset_with("echo.sh", 5);
    let reset = scene.reset;
    let program = reset
        .as_ref()
        .map(|reset| reset.program.ends_with("/fixtures/reset/echo.sh"));
    assert_eq!(program, Some(true), "the script of the fixtures");
    assert_eq!(
        reset.map(|reset| reset.timeout_secs.get()),
        Some(5),
        "its timeout"
    );
}

/// The pages of a scene are the pages of `fakes::page`, with more facts.
/// The fakes of the loop's own tests are used here too, so that this test
/// uses all of the shared file.
async fn check_fakes() {
    let shown = PageSpec::button("保存");
    let shown = PageSpec::at("/").with(shown).build(ORIGIN);
    let plain = fakes::page(&shown.screen.url, &[("button", "保存")]);
    assert_eq!(
        (&plain.nodes, &plain.candidates, &plain.text),
        (&shown.nodes, &shown.candidates, &shown.text),
        "the same nodes, candidates and text"
    );
    let mut browser = FakeBrowser::default();
    browser.pages.push_back(plain.clone());
    assert_eq!(
        browser.observe().await.ok(),
        Some(plain),
        "the page is shown"
    );
    assert_eq!(
        RecordingJudge::default().texts.len(),
        0,
        "nothing was asked"
    );
    assert_eq!(FakeClock::default().elapsed(), Duration::ZERO, "no time");
}

/// Checks the base.
pub async fn check() {
    check_reset();
    check_fakes().await;
    judge_checks::check().await;
    ledger_checks::check();
    check_run().await;
}
