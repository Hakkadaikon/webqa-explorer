//! The viewer of `webqa watch`, on the log of an exploring run.

use std::fs::{self, OpenOptions};
use std::future::ready;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use cucumber::{given, then, when};
use webqa::watch::watch as watching;
use webqa_core::ports::{Clock, Pending};
use webqa_tui::follow::{Follow, draw};

use super::frame::frame_of;
use super::tui;
use crate::interruption::SceneInterrupt;
use crate::scene::found;
use crate::world::InterviewWorld;

/// A run that ends as soon as it waits: the first wait finishes the log.
pub struct Finishing(pub PathBuf);

/// The line that ends a run.
pub const FINISHED: &str =
    r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#;

impl Clock for Finishing {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        let log = OpenOptions::new().append(true).open(&self.0);
        let mut log = found(log.ok(), "the log opens");
        assert!(writeln!(log, "{FINISHED}").is_ok(), "the log ends");
        Box::pin(ready(()))
    }
}

#[given(expr = "探索モードの実行が動いている")]
fn exploring_run(world: &mut InterviewWorld) {
    tui::make_dir(world);
    let started = r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#;
    let decided = r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#;
    let lines = format!("{started}\n{decided}\n");
    assert!(
        fs::write(tui::log(world), lines).is_ok(),
        "the log is written"
    );
}

#[when(expr = "webqa watch でその実行を開く")]
async fn watch(world: &mut InterviewWorld) {
    let mut clock = Finishing(tui::log(world));
    // The first wait is made by the race with the log, which the log wins;
    // the second is the user leaving once the run has finished.
    let mut interrupt = SceneInterrupt {
        fire_on: [2].into(),
        ..SceneInterrupt::default()
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let dir = tui::work_dir(world);
    let code = watching(&dir, &mut clock, &mut interrupt, &mut out, &mut err).await;
    assert_eq!((code, err), (0, Vec::new()), "the viewer ended well");
    tui::keep_screen(world, &out);
}

#[then(expr = "実行中のイベントログを追う画面が、これまでどおり表示される")]
fn viewer_draws_as_before(world: &mut InterviewWorld) {
    let log = fs::read(tui::log(world));
    let mut view = Follow::default();
    assert!(
        view.read(&found(log.ok(), "the log reads")),
        "the log was read"
    );
    let mut drawn = Vec::new();
    assert!(draw(&mut drawn, view.state()).is_ok(), "the viewer draws");
    let expected = String::from_utf8_lossy(&drawn).into_owned();
    let frame = tui::last_frame(world);
    assert_eq!(frame, frame_of(&expected), "the last screen");
    assert!(frame.contains("保存"), "the decision is drawn");
}
