//! Checks of what the Given steps of the later Rules set up: each field of
//! the scene reaches the run.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use webqa_core::interview::ask::{EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS};

use super::ORIGIN;

#[path = "checks_times.rs"]
mod times;
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
    step("click")
}

/// The answers of a step whose operation is `op`, on the first element.
fn step(op: &str) -> Said {
    Said::default()
        .choose("operation", op, 0.9)
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
    let opened = [
        (
            1,
            Opened::Launched {
                started: 0,
                finished: 0,
            },
        ),
        (1, auth),
        (1, Opened::FirstPage),
    ];
    assert_eq!(
        ran(scene).await.openings,
        opened,
        "the state, then the page"
    );
    let plain = ran(on_button("保存")).await.openings;
    let opened = [
        (
            1,
            Opened::Launched {
                started: 0,
                finished: 0,
            },
        ),
        (1, Opened::FirstPage),
    ];
    assert_eq!(plain, opened, "no state");
}

/// The bridge reads the request lines and declares whether it reads
/// images.
async fn check_bridge() {
    for images in [false, true] {
        let mut scene = on_button("保存");
        scene.bridge = Some(images);
        let said = click().write(HARD_TO_SEE, "");
        scene.replies.push_back(Ok(said.build()));
        let ran = ran(scene).await;
        let first = found(ran.bridge.first(), "a request");
        assert_eq!(first.get("image").is_some(), images, "the image");
        assert_eq!(ran.touches.len(), 1, "the answer reached the run");
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

/// The memory files stay as they were, and a second run of a changed
/// scene keeps the first.
async fn check_memory_and_again() {
    let mut scene = on_button("保存");
    scene.memories.insert(ORIGIN.to_owned(), "{}".to_owned());
    let mut world = InterviewWorld {
        scene,
        ..InterviewWorld::default()
    };
    world.run().await;
    world.again();
    world.scene.start = PageSpec::at("/").with(PageSpec::button("次へ"));
    world.run().await;
    let kept = &world.before().store.memories;
    assert_eq!(
        kept.get(ORIGIN).map(String::as_str),
        Some("{}"),
        "the memory"
    );
    let shown = |ran: &Ran, name: &str| {
        ran.judge
            .asked
            .first()
            .is_some_and(|a| a.screen.contains(name))
    };
    assert!(shown(world.before(), "保存"), "the first run");
    assert!(shown(world.ran(), "次へ"), "the second run");
}

/// Checks the fields of the scene.
pub async fn check() {
    check_forbidden().await;
    check_auth().await;
    check_bridge().await;
    check_api_key().await;
    check_memory_and_again().await;
    times::check().await;
}
