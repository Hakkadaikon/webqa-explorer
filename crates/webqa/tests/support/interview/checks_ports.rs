//! Checks of what the Given steps of the later Rules set up: each field of
//! the scene reaches the run.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use webqa::interview::trials::{Ending, Stopped};
use webqa_core::interview::ask::{EXPECTATION, HESITATION, PURPOSE_GUESS};
use webqa_core::ports::Clock;

use super::ORIGIN;
use crate::bridge::RunDir;
use crate::judge::Said;
use crate::launch::Opened;
use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::{InterviewWorld, Ran};

/// A scene of one trial on a page with the button `name`.
fn on_button(name: &str) -> Scene {
    Scene {
        origins: vec![ORIGIN.to_owned()],
        start: PageSpec::at("/").with(PageSpec::button(name)),
        ..Scene::default()
    }
}

/// The answers of a step that clicks the first element.
fn click() -> Said {
    Said::default()
        .choose("operation", "click", 0.9)
        .choose("target", "1", 0.9)
        .choose("error_shown", "true", 0.0)
        .choose("goal_reached", "true", 0.0)
        .choose(HESITATION, "true", 0.0)
        .write(EXPECTATION, "進む")
        .write(PURPOSE_GUESS, "買う")
}

/// The run of `scene`.
async fn ran(scene: Scene) -> Ran {
    let mut world = InterviewWorld {
        scene,
        ..InterviewWorld::default()
    };
    world.run().await;
    let ran = world.ran.take();
    found(ran, "it ran")
}

/// A forbidden keyword stops the click the judge picks. The guard also
/// forbids some words of its own, so the button says none of them.
async fn check_forbidden() {
    let mut scene = on_button("保存");
    scene.replies.push_back(Ok(click().build()));
    let mut forbidding = on_button("保存");
    forbidding.replies.clone_from(&scene.replies);
    forbidding.forbidden = vec!["保存".to_owned()];
    assert_eq!(ran(scene).await.touches.len(), 1, "clicked when allowed");
    assert_eq!(ran(forbidding).await.touches, [], "not when forbidden");
}

/// The storage state goes into the browser before its first page.
async fn check_auth() {
    let mut scene = on_button("保存");
    scene.auth = true;
    let auth = Opened::Auth(PathBuf::from("auth.json"));
    let opened = [(1, auth), (1, Opened::FirstPage)];
    assert_eq!(
        ran(scene).await.openings,
        opened,
        "the state, then the page"
    );
    let plain = ran(on_button("保存")).await.openings;
    assert_eq!(plain, [(1, Opened::FirstPage)], "no state");
}

/// The bridge reads the request lines and declares whether it reads
/// images.
async fn check_bridge() {
    for images in [false, true] {
        let mut scene = on_button("保存");
        scene.bridge = Some(images);
        scene.replies.push_back(Ok(click().build()));
        let ran = ran(scene).await;
        let first = found(ran.bridge.first(), "a request");
        assert_eq!(first.get("image").is_some(), images, "the image");
        assert!(
            ran.judge.asked.is_empty(),
            "the scene's judge was not asked"
        );
    }
}

/// A scene whose judge reads its key from `JEV_API_KEY`, with the reset
/// command that records its environment, run with `env`.
fn keyed(env: &[(&str, OsString)]) -> Scene {
    let mut scene = on_button("保存");
    scene.api_key_env = Some("JEV_API_KEY".to_owned());
    scene.reset_with("record.sh", 5);
    scene.env = env
        .iter()
        .map(|(name, value)| (OsString::from(name), value.clone()))
        .collect();
    scene
}

/// The API key comes from the environment and does not reach the reset
/// command, as the CLI withholds it.
async fn check_api_key() {
    let dir = RunDir::new();
    let out = dir.0.join("reset");
    let env = [
        ("JEV_API_KEY", "the-key".into()),
        ("COPY", "Bearer the-key".into()),
        ("RESET_OUT", out.clone().into_os_string()),
    ];
    assert_eq!(ran(keyed(&env)).await.config_error, None, "the key is read");
    let seen = fs::read_to_string(out.with_extension("env")).unwrap_or_default();
    assert!(
        seen.contains("RESET_OUT") && !seen.contains("the-key"),
        "{seen}"
    );
    let missing = ran(keyed(&[])).await.config_error;
    assert!(missing.is_some(), "no variable");
}

/// A reset script moves the clock; the judge's calls take their times.
async fn check_times() {
    let mut scene = on_button("保存");
    scene.reset_with("clock.sh", 5);
    scene.env = vec![("RESET_ENDS_MS".into(), "120000".into())];
    scene.thinks.push_back(Duration::from_secs(60));
    let ran = ran(scene).await;
    assert_eq!(
        ran.clock.elapsed(),
        Duration::from_secs(180),
        "reset, then a call"
    );
}

/// The memory files stay as they were, and a second run keeps the first.
async fn check_memory_and_again() {
    let mut scene = on_button("保存");
    scene.memories.insert(ORIGIN.to_owned(), "{}".to_owned());
    let mut world = InterviewWorld {
        scene,
        ..InterviewWorld::default()
    };
    world.run().await;
    world.again();
    world.run().await;
    let kept = &world.before().store.memories;
    assert_eq!(
        kept.get(ORIGIN).map(String::as_str),
        Some("{}"),
        "the memory"
    );
    assert_eq!(world.ran().launched, 1, "the second run opened its own");
}

/// A browser that never opens is cut short by the time limit.
async fn check_hang() {
    let mut hanging = on_button("保存");
    hanging.hang_launch = Some(1);
    hanging.max_duration = Some(Duration::from_secs(60));
    let ran = ran(hanging).await;
    let ending = Some(Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    });
    assert_eq!((ran.ending, ran.launched), (ending, 1), "time is up");
}

/// A window of two accesses: the first page and a click fill it, so the
/// next step waits for the window.
fn filling(stall_sleep: bool) -> Scene {
    let mut scene = on_button("保存");
    scene.requests_per_hour = 2;
    scene.stall_sleep = stall_sleep;
    scene.replies.push_back(Ok(click().build()));
    scene
}

/// A sleep moves the clock, or never ends and wakes the interrupt.
async fn check_sleeps() {
    let hour = Duration::from_secs(3600);
    let waited = ran(filling(false)).await;
    assert_eq!(waited.clock.sleeps, [hour], "it waited for the window");
    assert_eq!(waited.clock.elapsed(), hour, "and the clock moved");
    let stalled = ran(filling(true)).await;
    assert_eq!(stalled.clock.sleeps, [hour], "it began to wait");
    assert_eq!(stalled.clock.elapsed(), Duration::ZERO, "the clock stayed");
}

/// Checks the fields of the scene.
pub async fn check() {
    check_forbidden().await;
    check_auth().await;
    check_bridge().await;
    check_api_key().await;
    check_times().await;
    check_memory_and_again().await;
    check_hang().await;
    check_sleeps().await;
}
