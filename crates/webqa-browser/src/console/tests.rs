//! Tests of keeping the console errors between observations.

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
