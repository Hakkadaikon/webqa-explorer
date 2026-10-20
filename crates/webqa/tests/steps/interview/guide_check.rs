//! The check of the documents: the executable `webqa check-docs`.

use std::process::Command;

use cucumber::{given, then, when};

use crate::rerun::course::{stash, stashed};
use crate::scene::found;
use crate::world::InterviewWorld;

/// The documents, the kind of each, and what the check says of it.
const DOCS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/docs/");

/// A document to check: its kind, its file, and what a check says of it
/// (nothing when it is right).
fn document(world: &mut InterviewWorld, kind: &str, file: &str, said: &str) {
    stash(world, "DOC_KIND", kind);
    stash(world, "DOC_FILE", &format!("{DOCS}{file}"));
    stash(world, "DOC_SAID", said);
}

#[given(expr = "決まった形式で書いた ペルソナの文書 がある")]
fn right_persona(world: &mut InterviewWorld) {
    document(world, "persona", "persona.yaml", "");
}

#[given(expr = "決まった形式で書いた 目的の設定 がある")]
fn right_purpose(world: &mut InterviewWorld) {
    document(world, "purpose", "purpose.yaml", "");
}

#[given(expr = "ペルソナの文書に ITの慣れ が無い のある文書がある")]
fn persona_without_literacy(world: &mut InterviewWorld) {
    document(
        world,
        "persona",
        "persona-no-literacy.yaml",
        "persona.it_literacy を書く",
    );
}

#[given(expr = "目的の設定の done_when に宣言が1つも無い のある文書がある")]
fn purpose_without_declaration(world: &mut InterviewWorld) {
    let said = "purpose.done_when に url、element、text のどれかを1つ以上書く";
    document(world, "purpose", "purpose-no-declaration.yaml", said);
}

#[when(expr = "形式の検査のサブコマンドでその文書を検査する")]
fn check(world: &mut InterviewWorld) {
    let args = [
        "check-docs".to_owned(),
        stashed(world, "DOC_KIND"),
        stashed(world, "DOC_FILE"),
    ];
    let output = Command::new(env!("CARGO_BIN_EXE_webqa"))
        .args(args)
        .output();
    let output = found(output.ok(), "the check ran");
    let code = output.status.code().map_or(-1, i32::from);
    stash(world, "DOC_CODE", &code.to_string());
    stash(
        world,
        "DOC_STDERR",
        String::from_utf8_lossy(&output.stderr).trim_end(),
    );
}

#[then(expr = "検査は成功し、終了コードは {int} である")]
fn succeeds_with(world: &mut InterviewWorld, code: i32) {
    exits_with(world, code);
    assert_eq!(stashed(world, "DOC_STDERR"), "", "nothing was said");
}

#[then(expr = "検査は失敗し、終了コードは {int} である")]
fn fails_with(world: &mut InterviewWorld, code: i32) {
    exits_with(world, code);
}

fn exits_with(world: &mut InterviewWorld, code: i32) {
    assert_eq!(
        stashed(world, "DOC_CODE"),
        code.to_string(),
        "the exit code"
    );
}

#[then(expr = "誤りのある項目の名前が表示されている")]
fn names_the_item(world: &mut InterviewWorld) {
    let said = stashed(world, "DOC_SAID");
    assert_ne!(said, "", "a wrong document is told");
    assert_eq!(stashed(world, "DOC_STDERR"), said, "what the check said");
}
