//! Tests of where the files of the store go.

use super::*;

#[test]
fn memory_file_encodes_origin() {
    let cases = [
        (
            "https://app.example.test",
            "https%3A%2F%2Fapp.example.test.json",
        ),
        (
            "http://127.0.0.1:8080",
            "http%3A%2F%2F127.0.0.1%3A8080.json",
        ),
        (
            "https://*.app.example.test",
            "https%3A%2F%2F*.app.example.test.json",
        ),
    ];
    for (origin, expected) in cases {
        assert_eq!(memory_file(origin), expected, "the file of {origin}");
    }
}
