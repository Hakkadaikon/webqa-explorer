//! Contract tests of the wait for a page to rest after a touch.

use std::future::Future;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Duration;

use tokio::runtime::Builder;

use super::port::{at, id, kinds, launch, seen, touch};

/// How long a test may wait on the page, once the browser is open.
const LIMIT: Duration = Duration::from_secs(10);

/// Runs `body` on a thread of its own and fails if it is not done within
/// `LIMIT` of the moment it calls `ready`, so that a wait with no end in the
/// page ends in a failure instead of hanging. Opening the browser takes as
/// long as the machine is busy, so it is not counted. The tests read no
/// clock, so the thread that waits is the timer; a body that hangs is left
/// behind to end with the process.
fn bounded<F: Future<Output = ()>>(body: impl FnOnce(Sender<()>) -> F + Send + 'static) {
    let (ready, started) = channel();
    let (done, wait) = channel();
    thread::spawn(move || {
        let ran = Builder::new_current_thread()
            .enable_all()
            .build()
            .map(|runtime| runtime.block_on(body(ready)));
        done.send(ran.is_ok()).ok();
    });
    assert_eq!(started.recv(), Ok(()), "the browser did not open");
    assert_eq!(
        wait.recv_timeout(LIMIT),
        Ok(true),
        "the page was waited for without end"
    );
}

/// A page that rewrites its content again and again after the click is not
/// read before it is quiet: each rewrite, the second at 600 ms, restarts the
/// wait, so the observation shows the last content.
#[tokio::test]
async fn waits_for_the_page_to_stop_changing_after_a_touch() {
    let mut browser = launch(at("/rewriting").unwrap(), 0).await.unwrap();
    seen(&mut browser).await.unwrap();
    touch(&mut browser, "click", Some(id("button", "検索")))
        .await
        .unwrap();
    let o = seen(&mut browser).await.unwrap();
    assert_eq!(
        kinds(&o),
        [("button", "検索"), ("button", "結果")],
        "the rewritten content"
    );
}

/// A page that never stops changing is waited for up to a limit, and then
/// read as it is: the touch is no failure. The page rewrites every 100 ms and
/// the limit is 2000 ms, so it is read after some 20 rewrites, not after
/// none and not after waiting for ever.
#[test]
fn leaves_a_page_that_never_stops_changing_when_the_limit_comes() {
    bounded(|ready| async move {
        let mut browser = launch(at("/restless").unwrap(), 0).await.unwrap();
        seen(&mut browser).await.unwrap();
        ready.send(()).unwrap();
        let moved = touch(&mut browser, "click", Some(id("button", "動かす"))).await;
        let o = seen(&mut browser).await.unwrap();
        let ticks = kinds(&o)[1].1.trim_start_matches("tick").parse::<u32>();
        assert_eq!(moved, Ok(()), "no failure at the limit");
        assert_eq!(
            ticks.map(|n| (5..=30).contains(&n)),
            Ok(true),
            "waited while it changed, and no longer than the limit"
        );
    });
}

/// A busy dialog that is not shown is not waited for until the timeout: the
/// touch ends at the rest, before the page rewrites itself at 800 ms, while a
/// wait for the dialog would end at the timeout of 1 s, after it.
#[tokio::test]
async fn does_not_wait_for_a_loading_veil_that_is_hidden() {
    let mut browser = launch(at("/hidden-veil").unwrap(), 0).await.unwrap();
    seen(&mut browser).await.unwrap();
    touch(&mut browser, "click", Some(id("button", "検索")))
        .await
        .unwrap();
    let o = seen(&mut browser).await.unwrap();
    assert_eq!(
        kinds(&o),
        [("button", "検索"), ("button", "待機")],
        "read at the rest"
    );
}
