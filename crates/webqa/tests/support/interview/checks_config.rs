//! Checks of the stages of starting: the configuration and the ledger
//! before stop the run as the CLI does.

use webqa_core::config::interview::InterviewError;

use super::ORIGIN;
use crate::scene::Scene;
use crate::world::InterviewWorld;

/// A scene of the background.
fn background() -> Scene {
    Scene {
        origins: vec![ORIGIN.to_owned()],
        ..Scene::default()
    }
}

/// The sentence of the configuration error of `scene`, if any, after
/// checking that nothing ran.
async fn rejected(scene: Scene) -> Option<String> {
    let mut world = InterviewWorld {
        scene,
        ..InterviewWorld::default()
    };
    world.run().await;
    let ran = world.ran();
    if ran.config_error.is_some() {
        assert_eq!((ran.code, ran.launched), (2, 0), "stopped before a trial");
        assert_eq!(ran.ending, None, "nothing ran");
    }
    ran.config_error.clone()
}

/// Values out of range are rejected as the CLI rejects them: zero by the
/// YAML, the others by the interview settings.
async fn check_ranges() {
    let scenes = [
        Scene {
            trials: 0,
            ..background()
        },
        Scene {
            trials: 101,
            ..background()
        },
        Scene {
            hesitation: 1.5,
            ..background()
        },
        Scene {
            interview_requests: Some(1),
            ..background()
        },
    ];
    for scene in scenes {
        let error = rejected(scene).await;
        assert!(error.is_some(), "out of range: {error:?}");
    }
    let over = Scene {
        trials: 101,
        ..background()
    };
    let sentence = InterviewError::Trials.to_string();
    assert_eq!(
        rejected(over).await,
        Some(sentence),
        "the sentence of the setting"
    );
    let plain = rejected(background()).await;
    assert_eq!(plain, None, "the background reads");
}

/// A ledger before that does not read, and one of another persona, stop
/// the run; one of this persona does not.
async fn check_previous() {
    let mut first = InterviewWorld {
        scene: background(),
        ..InterviewWorld::default()
    };
    first.run().await;
    let text = first.ran().store.ledger.clone();
    let broken = Scene {
        previous: Some("# 壊れた".to_owned()),
        ..background()
    };
    assert!(
        rejected(broken).await.is_some(),
        "a ledger that does not read"
    );
    let mut other = Scene {
        previous: text.clone(),
        ..background()
    };
    "別の人".clone_into(&mut other.persona.name);
    assert_eq!(
        rejected(other).await.as_deref(),
        Some("前回の台帳のペルソナが今回と違う"),
        "another persona"
    );
    let same = Scene {
        previous: text,
        ..background()
    };
    assert_eq!(rejected(same).await, None, "the same persona and purpose");
}

/// Checks the stages of starting.
pub async fn check() {
    Box::pin(check_ranges()).await;
    check_previous().await;
}
