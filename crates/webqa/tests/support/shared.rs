//! What the steps of more than one Rule use: the stage "ログイン", a page
//! of another origin, the first wait of a run, and small readers.

use serde_json::Value;
use webqa_core::config::raw::{DoneWhen, Stage};

use crate::page::{ElementSpec, PageSpec};
use crate::scene::found;

/// The `done_when` that the page at `path` meets.
pub fn url_done(path: &str) -> DoneWhen {
    DoneWhen {
        url: Some(path.to_owned()),
        element: None,
        text: None,
    }
}

/// The `done_when` that a page showing `text` meets.
pub fn text_done(text: &str) -> DoneWhen {
    DoneWhen {
        url: None,
        element: None,
        text: Some(text.to_owned()),
    }
}

/// The scenario with the one stage "ログイン", done when `done_when`.
pub fn login_only(done_when: DoneWhen) -> Vec<Stage> {
    vec![Stage {
        name: "ログイン".to_owned(),
        goal: "ログインする".to_owned(),
        done_when,
    }]
}

/// The origin outside the allowed ones.
pub const OTHER: &str = "https://other.example.net";

/// The page of an article at `origin`, a whole URL of another origin.
pub fn foreign_page(origin: &str) -> PageSpec {
    let page = PageSpec {
        origin: Some(origin.to_owned()),
        ..PageSpec::at("")
    };
    page.with(PageSpec::button("記事"))
}

/// The position of the first `waited` in `kinds`, the log kinds of a run.
pub fn first_wait(kinds: &[String]) -> usize {
    found(kinds.iter().position(|kind| kind == "waited"), "a wait")
}

/// A button named `name`.
pub fn button(name: impl Into<String>) -> ElementSpec {
    PageSpec::button(&name.into())
}

/// The text at `keys` of `value`, one key after the other. Nothing there is "".
pub fn text_at(value: &Value, keys: &[&str]) -> String {
    let at = keys.iter().try_fold(value, |value, key| value.get(*key));
    at.and_then(Value::as_str).unwrap_or_default().to_owned()
}
