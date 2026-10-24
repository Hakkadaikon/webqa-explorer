//! The container image has the browsers of the Playwright version that
//! playwright-rs drives (docs/container.md).

use std::fs;
use std::path::PathBuf;

const DOCKERFILE: &str = "docker/Dockerfile";

#[test]
fn image_matches_the_playwright_version() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(DOCKERFILE);
    let text = fs::read_to_string(path).expect("read the Dockerfile");
    let base = format!(
        "FROM mcr.microsoft.com/playwright:v{}-noble",
        playwright_rs::PLAYWRIGHT_VERSION
    );
    let from: Vec<&str> = text.lines().filter(|l| l.starts_with("FROM ")).collect();
    assert_eq!(from, [base.as_str()], "the base image of {DOCKERFILE}");
}
