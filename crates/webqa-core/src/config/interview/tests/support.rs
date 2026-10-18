//! Building configurations and reading documents for the tests.

use std::io;
use std::path::Path;

use super::super::{Interview, load};
use crate::config::persona::tests::PERSONA;
use crate::config::raw::parse;
use crate::config::raw::tests::example;
use crate::config::validate::validate;
use crate::config::{Config, ConfigError};

/// A storage state in Playwright's format.
const STATE: &str = r#"{"cookies":[],"origins":[]}"#;

/// Reads the files in `files` by path, and nothing else.
fn table<'a>(files: &'a [(&'a str, &'a str)]) -> impl Fn(&Path) -> io::Result<String> + 'a {
    move |path| lookup(files, path)
}

fn lookup(files: &[(&str, &str)], path: &Path) -> io::Result<String> {
    files
        .iter()
        .find(|(name, _)| Path::new(name) == path)
        .map(|&(_, text)| text.to_owned())
        .ok_or(io::ErrorKind::NotFound)
        .map_err(io::Error::from)
}

/// `base` with `section` appended, validated with every environment
/// variable set.
pub fn validated(base: &str, section: &str) -> Result<Config, ConfigError> {
    let read = table(&[("./auth.json", STATE)]);
    validate(
        parse(&format!("{base}{section}"))?,
        &|name| Some(name.to_owned()),
        &read,
    )
}

/// `base` with `section` appended, validated.
pub fn config_with(base: &str, section: &str) -> Config {
    validated(base, section).unwrap()
}

/// Loads the interview with `files` readable.
pub fn load_files(config: &Config, files: &[(&str, &str)]) -> Result<Interview, ConfigError> {
    load(config, &table(files))
}

/// Loads the interview with the persona document `persona`, and the purpose
/// document `purpose` if any.
pub fn load_with(
    config: &Config,
    persona: &str,
    purpose: Option<&str>,
) -> Result<Interview, ConfigError> {
    let mut files = vec![("persona.yaml", persona)];
    files.extend(purpose.map(|text| ("purpose.yaml", text)));
    load_files(config, &files)
}

/// An `interview` section with the persona and `extra` lines.
pub fn section(extra: &str) -> String {
    format!("interview:\n  persona: persona.yaml\n{extra}")
}

/// The example with an `interview` section with `extra` lines, loaded with
/// [`PERSONA`].
pub fn loaded(extra: &str) -> Result<Interview, ConfigError> {
    load_with(&config_with(example(), &section(extra)), PERSONA, None)
}

/// The message of loading [`loaded`].
pub fn rejected(extra: &str) -> String {
    loaded(extra).unwrap_err().to_string()
}

/// An error from reading the YAML.
pub fn yaml_error() -> ConfigError {
    parse("[").unwrap_err()
}
