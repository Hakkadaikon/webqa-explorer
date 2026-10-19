//! Pages and observations for the tests of the state machine.

use url::Url;

use crate::machine::{Facts, Material, Seen, Whole};
use crate::redact::Redactor;
use crate::screen::{Element, Rect, Screen, Viewport};

/// The viewport of every test page.
pub const VIEWPORT: Viewport = Viewport {
    width: 800,
    height: 600,
};

/// The allowed origin of the background.
pub const APP: &str = "https://app.example.test";

fn at(role: &str, name: &str, y: i64) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect: Some(Rect {
            x: 0,
            y,
            width: 10,
            height: 10,
        }),
    }
}

/// An element inside the viewport.
pub fn inside(role: &str, name: &str) -> Element {
    at(role, name, 0)
}

/// The heading `name` far below the viewport, at (0, 1200), 100 by 20.
pub fn heading(name: &str) -> Element {
    Element {
        rect: Some(Rect {
            x: 0,
            y: 1200,
            width: 100,
            height: 20,
        }),
        ..at("heading", name, 0)
    }
}

/// An element below the viewport.
pub fn below(role: &str, name: &str) -> Element {
    at(role, name, 700)
}

/// A page at `url` with the elements inside the viewport.
pub fn page(url: &str, elements: &[(&str, &str)]) -> Screen {
    Screen {
        url: Url::parse(url).unwrap(),
        elements: elements
            .iter()
            .map(|(role, name)| inside(role, name))
            .collect(),
    }
}

/// `screen` with `element` added at the end.
pub fn with(mut screen: Screen, element: Element) -> Screen {
    screen.elements.push(element);
    screen
}

/// The start page: a button "保存" and a link "ヘルプ".
pub fn top() -> Screen {
    page(
        &format!("{APP}/"),
        &[("button", "保存"), ("link", "ヘルプ")],
    )
}

/// A page outside the allowed origins.
pub fn away() -> Screen {
    page("https://other.example.net/", &[("link", "記事")])
}

/// A paywall by its URL, with no elements.
pub fn pricing() -> Screen {
    page(&format!("{APP}/pricing"), &[])
}

/// The observation of `screen`, with the material and the whole page made
/// from it.
pub fn seen(screen: &Screen) -> Seen {
    let redacted = Redactor::default().screen(screen.clone());
    Seen {
        whole: Whole::plain(&redacted),
        screen: redacted,
        viewport: VIEWPORT,
        material: Material::plain(screen, VIEWPORT),
    }
}

/// The observation of `screen`, with the whole page of the elements
/// `outline` and the text `text`.
pub fn seen_whole(screen: &Screen, outline: &[Element], text: &str) -> Seen {
    let redactor = Redactor::default();
    let outline = Screen {
        url: screen.url.clone(),
        elements: outline.to_vec(),
    };
    Seen {
        whole: Whole {
            outline: redactor.screen(outline),
            text: redactor.text(text),
        },
        ..seen(screen)
    }
}

/// The facts of observing `screen`.
pub fn shows(screen: &Screen) -> Facts {
    Facts::Page(Box::new(seen(screen)))
}
