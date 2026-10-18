//! Contract tests of the port when Playwright fails: the launch stops with
//! an error, and a closed page fails to observe and to touch.

use std::sync::{Arc, Mutex, PoisonError};

use playwright_rs::{BrowserContext, Error};
use webqa_browser::{LaunchError, PlaywrightBrowser};
use webqa_core::ports::{Browser, BrowserError};

use super::port::{at, kinds, launch, touch};
use super::serve::answer;
use super::session::Res;

#[tokio::test]
async fn unreadable_storage_state_fails_launch() {
    let mut options = at("/start").unwrap();
    options.storage_state = Some(env!("CARGO_TARGET_TMPDIR").into());
    let launched = PlaywrightBrowser::launch(options).await;
    assert_eq!(launched.err(), Some(LaunchError::Other), "not a file");
}

#[tokio::test]
async fn failed_preparation_fails_launch() {
    let launched = PlaywrightBrowser::launch_with(at("/start").unwrap(), |_| async {
        Err(Error::InvalidArgument("prepare".to_owned()))
    });
    assert_eq!(launched.await.err(), Some(LaunchError::Other), "prepare");
}

#[tokio::test]
async fn closed_context_fails_launch() {
    let launched = PlaywrightBrowser::launch_with(at("/start").unwrap(), |context| async move {
        context.close().await
    });
    assert_eq!(launched.await.err(), Some(LaunchError::Other), "no page");
}

/// The context the port opened its page in, kept by the test.
type Kept = Arc<Mutex<Option<BrowserContext>>>;

/// Launches on `/start`, keeping the context.
async fn launch_keeping(kept: &Kept) -> Res<PlaywrightBrowser> {
    let keep = Arc::clone(kept);
    let launched = PlaywrightBrowser::launch_with(at("/start")?, move |context| async move {
        let routed = context.route("**/*", |route| answer(route, 0)).await;
        *keep.lock().unwrap_or_else(PoisonError::into_inner) = Some(context);
        routed
    });
    Ok(launched.await?)
}

#[tokio::test]
async fn closed_page_fails_observe_and_touch() {
    let kept = Kept::default();
    let mut browser = launch_keeping(&kept).await.unwrap();
    let context = kept.lock().unwrap().take().unwrap();
    context.close().await.unwrap();
    assert_eq!(browser.observe().await.err(), Some(BrowserError), "observe");
    let touched = touch(&mut browser, "back", None).await;
    assert_eq!(touched, Err(BrowserError), "touch");
}

#[tokio::test]
async fn element_failing_probe_is_left_out() {
    let mut options = at("/start").unwrap();
    options.selectors.forbidden = vec!["[[[".to_owned()];
    let mut browser = launch(options, 0).await.unwrap();
    let o = browser.observe().await.unwrap();
    assert_eq!(kinds(&o), [], "every probe failed");
    assert_eq!((o.nodes.len(), o.candidates.len()), (0, 0), "no materials");
}
