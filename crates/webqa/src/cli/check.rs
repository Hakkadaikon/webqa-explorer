//! `check-docs`: the form of a persona or purpose document.

#[cfg(test)]
mod tests;

use std::fs;

use webqa_core::config::{persona, purpose};

use super::app::Done;
use super::start::Stop;
use super::{CheckDocsArgs, DocKind};

/// Reads the document and says nothing when its form is right. Neither a
/// browser nor a judge starts.
#[must_use]
pub fn app(args: &CheckDocsArgs) -> Done {
    let code = checked(args).map_or_else(
        |stop| {
            let (code, text) = stop.into_parts();
            super::say(&text);
            code
        },
        |()| 0,
    );
    Done {
        code,
        abandoned: false,
    }
}

fn checked(args: &CheckDocsArgs) -> Result<(), Stop> {
    let text = fs::read_to_string(&args.file).map_err(|_| Stop::config("文書を読めない"))?;
    match args.kind {
        DocKind::Persona => persona::parse(&text).map(drop),
        DocKind::Purpose => purpose::parse(&text).map(drop),
    }
    .map_err(Stop::config)
}
