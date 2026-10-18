//! Tests of keeping the console errors between observations.

use std::thread;

use super::*;

#[test]
fn keeps_errors_until_taken() {
    let console = Console::default();
    let writer = console.clone();
    for (kind, text) in [
        ("error", "boom"),
        ("warning", "w"),
        ("log", "l"),
        ("error", "x"),
    ] {
        writer.write(kind, text);
    }
    assert_eq!(console.take(), ["boom", "x"], "errors only, in order");
    assert_eq!(console.take(), Vec::<String>::new(), "taken once");
}

#[test]
fn keeps_errors_after_a_writer_panicked() {
    let console = Console::default();
    console.write("error", "before");
    let shared = console.clone();
    let panicked = thread::spawn(move || {
        let _held = shared.0.lock().unwrap();
        panic!("a writer panicked while holding the list");
    })
    .join();
    assert_eq!(
        (panicked.is_err(), console.0.is_poisoned()),
        (true, true),
        "the list is poisoned"
    );
    console.write("error", "after");
    assert_eq!(console.take(), ["before", "after"], "both errors kept");
}
