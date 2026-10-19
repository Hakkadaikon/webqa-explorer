//! Tests of the real clock and the wall-clock strings.

use super::*;

/// Which of two waits ends first. Racing them both ways covers both arms.
async fn first(mut a: RealClock, at_a: Duration, mut b: RealClock, at_b: Duration) -> &'static str {
    tokio::select! {
        biased;
        () = a.sleep_until(at_a) => "a",
        () = b.sleep_until(at_b) => "b",
    }
}

#[tokio::test]
async fn real_clock_measures_and_waits() {
    let mut clock = RealClock::start();
    assert!(clock.elapsed() < Duration::from_secs(1), "just started");
    let target = clock.elapsed() + Duration::from_millis(20);
    clock.sleep_until(target).await;
    assert!(clock.elapsed() >= target, "waited until {target:?}");
    let copy = clock;
    let gap = copy.elapsed().abs_diff(clock.elapsed());
    assert!(gap < Duration::from_secs(1), "the copy shares the start");
    let soon = clock.elapsed() + Duration::from_millis(10);
    let never = first(clock, Duration::MAX, copy, soon).await;
    assert_eq!(never, "b", "an overflowing wait keeps waiting");
    let soon = clock.elapsed() + Duration::from_millis(10);
    let again = first(copy, soon, clock, Duration::MAX).await;
    assert_eq!(again, "a", "either way round");
}

/// Whether `s` has the form of `form`, where `d` stands for any digit and
/// every other character for itself.
fn shaped(s: &str, form: &str) -> bool {
    s.chars().count() == form.chars().count() && s.chars().zip(form.chars()).all(fits)
}

fn fits((c, f): (char, char)) -> bool {
    if f == 'd' { c.is_ascii_digit() } else { c == f }
}

#[test]
fn wall_strings_have_fixed_forms() {
    let at: jiff::Timestamp = "2026-10-18T02:00:00.123Z".parse().unwrap();
    let wall = Wall::at(at);
    let want = Wall {
        started_at: "2026-10-18T02:00:00Z".to_owned(),
        run_id: "20261018T020000.123Z".to_owned(),
    };
    assert_eq!(wall, want, "fixed strings");
    let now = Wall::now();
    let started = &now.started_at;
    assert!(
        shaped(started, "dddd-dd-ddTdd:dd:ddZ"),
        "started_at {started}"
    );
    let run_id = &now.run_id;
    assert!(shaped(run_id, "ddddddddTdddddd.dddZ"), "run_id {run_id}");
    assert!(!shaped("2026", "dddd-"), "a shorter string");
    assert!(!shaped("x", "d"), "not a digit");
}
