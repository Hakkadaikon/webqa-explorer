//! Tests of the arguments and the checks of `check-docs`.

use clap::Parser;
use clap::error::ErrorKind;

use super::*;
use crate::cli::{Cli, Command, DocKind, Other};

fn read(args: &[&str]) -> Result<Cli, ErrorKind> {
    Cli::try_parse_from(args).map_err(|e| e.kind())
}

const DOCS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/docs/");

#[test]
fn check_docs_arguments_are_read() {
    let read_ok = |args: &[&str]| read(args).map(|cli| cli.command);
    let docs = |kind| {
        Command::Other(Other::CheckDocs(CheckDocsArgs {
            kind,
            file: "a".into(),
        }))
    };
    assert_eq!(
        read_ok(&["webqa", "check-docs", "persona", "a"]),
        Ok(docs(DocKind::Persona))
    );
    assert_eq!(
        read_ok(&["webqa", "check-docs", "purpose", "a"]),
        Ok(docs(DocKind::Purpose))
    );
    assert_eq!(
        read(&["webqa", "check-docs", "a"]),
        Err(ErrorKind::InvalidValue)
    );
}

#[test]
fn documents_are_checked_for_their_form() {
    let check = |kind, file: &str| {
        let file = format!("{DOCS}{file}").into();
        checked(&CheckDocsArgs { kind, file }).err()
    };
    assert_eq!(check(DocKind::Persona, "persona.yaml"), None);
    assert_eq!(check(DocKind::Purpose, "purpose.yaml"), None);
    assert_eq!(
        check(DocKind::Persona, "persona-no-literacy.yaml"),
        Some(Stop::Config("persona.it_literacy を書く".into()))
    );
    assert_eq!(
        check(DocKind::Persona, "missing.yaml"),
        Some(Stop::Config("文書を読めない".into()))
    );
    assert_eq!(
        check(DocKind::Purpose, "purpose-no-declaration.yaml"),
        Some(Stop::Config(
            "purpose.done_when に url、element、text のどれかを1つ以上書く".into()
        ))
    );
}

#[test]
fn a_wrong_shape_is_said_without_the_text_of_the_document() {
    let file = format!("{DOCS}persona-secret-avoids.yaml").into();
    let said = checked(&CheckDocsArgs {
        kind: DocKind::Persona,
        file,
    })
    .err()
    .map(Stop::into_parts);
    let text = said.map(|(_, text)| text).unwrap_or_default();
    assert!(!text.contains("秘密の文"), "{text}");
}
