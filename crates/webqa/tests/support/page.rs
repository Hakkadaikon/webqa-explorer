//! A page a scene shows, built into the browser's `Observation`.

use url::Url;
use webqa_core::guard::forbidden::Candidate;
use webqa_core::guard::identity::Node;
use webqa_core::guard::paywall::PaywallFacts;
use webqa_core::ports::Observation;
use webqa_core::screen::{Element, Rect, Screen, Viewport};

use crate::scene::found;

/// The viewport of every page of a scene.
const VIEWPORT: Viewport = Viewport {
    width: 800,
    height: 600,
};

/// Where the elements below the viewport start.
const BELOW: i64 = 1000;

/// A page to show, built into an `Observation`.
#[derive(Debug, Clone, Default)]
pub struct PageSpec {
    /// The path of the URL. Empty for a URL that is all in `origin`.
    pub path: String,
    /// The origin. `None` is the first allowed origin.
    pub origin: Option<String>,
    /// The operable elements, in page order.
    pub elements: Vec<ElementSpec>,
    /// The headings as (name, below the viewport).
    pub headings: Vec<(String, bool)>,
    /// The names inside a modal. Empty is no modal.
    pub dialog: Vec<String>,
    /// Whether every main operation is covered.
    pub blocked: bool,
    /// The console errors of this observation.
    pub console: Vec<String>,
    /// The text of the whole page. `None` is the names of the elements and
    /// headings, one per line.
    pub text: Option<String>,
}

/// An operable element of a page.
#[derive(Debug, Clone)]
pub struct ElementSpec {
    /// The ARIA role.
    pub role: String,
    /// The accessible name.
    pub name: String,
    /// Whether it is below the viewport.
    pub below: bool,
    /// The link target.
    pub href: Option<String>,
    /// The form action.
    pub form_action: Option<String>,
    /// Whether the configuration lists it as forbidden.
    pub forbidden_listed: bool,
    /// Whether the configuration lists it as allowed.
    pub allowed_listed: bool,
    /// The element reference. `None` is `e<position in the page>`.
    pub node: Option<String>,
}

impl PageSpec {
    /// An empty page at `path`.
    pub fn at(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            ..Self::default()
        }
    }

    /// The page with `element` added.
    #[must_use]
    pub fn with(mut self, element: ElementSpec) -> Self {
        self.elements.push(element);
        self
    }

    /// A button.
    pub fn button(name: &str) -> ElementSpec {
        ElementSpec::new("button", name)
    }

    /// A link.
    pub fn link(name: &str) -> ElementSpec {
        ElementSpec::new("link", name)
    }

    /// A text box.
    pub fn textbox(name: &str) -> ElementSpec {
        ElementSpec::new("textbox", name)
    }

    /// The observation of the page. An unreadable URL fails the scenario.
    pub fn build(&self, default_origin: &str) -> Observation {
        let url = self.url(default_origin);
        let elements: Vec<Element> = self.elements.iter().zip(0..).map(shape).collect();
        let headings: Vec<Element> = self.headings.iter().zip(0..).map(heading).collect();
        let all: Vec<Element> = elements.iter().chain(&headings).cloned().collect();
        let names: Vec<&str> = all.iter().map(|e| e.name.as_str()).collect();
        Observation {
            nodes: self
                .elements
                .iter()
                .zip(1..)
                .map(ElementSpec::node_of)
                .collect(),
            candidates: self.elements.iter().map(|e| e.candidate(&url)).collect(),
            paywall: self.paywall(&url),
            console: self.console.clone(),
            text: self.text.clone().unwrap_or_else(|| names.join("\n")),
            outline: all,
            screen: Screen { url, elements },
            viewport: VIEWPORT,
        }
    }

    /// What the paywall check reads: the headings, the names in the modal,
    /// the buttons and whether each main operation can be reached, all
    /// inside the viewport.
    fn paywall(&self, url: &Url) -> PaywallFacts {
        let shown: Vec<&ElementSpec> = self.elements.iter().filter(|e| !e.below).collect();
        PaywallFacts {
            url: url.clone(),
            headings: self.headings_in_view(),
            dialogs: self.dialog.clone(),
            buttons: button_names(&shown),
            hits: vec![!self.blocked; shown.len()],
        }
    }

    fn headings_in_view(&self) -> Vec<String> {
        let shown = self.headings.iter().filter(|(_, below)| !below);
        shown.map(|(name, _)| name.clone()).collect()
    }

    fn url(&self, default_origin: &str) -> Url {
        let origin = self.origin.as_deref().unwrap_or(default_origin);
        let parsed = Url::parse(&format!("{origin}{}", self.path));
        found(parsed.ok(), "the URL of the page reads")
    }
}

impl ElementSpec {
    /// An element with `role` and `name`.
    pub fn new(role: &str, name: &str) -> Self {
        Self {
            role: role.to_owned(),
            name: name.to_owned(),
            below: false,
            href: None,
            form_action: None,
            forbidden_listed: false,
            allowed_listed: false,
            node: None,
        }
    }

    /// The element placed below the viewport.
    #[must_use]
    pub fn below(self) -> Self {
        Self {
            below: true,
            ..self
        }
    }

    /// The element linking to `to`.
    #[must_use]
    pub fn href(self, to: &str) -> Self {
        Self {
            href: Some(to.to_owned()),
            ..self
        }
    }

    /// The element submitting to `to`.
    #[must_use]
    pub fn form_action(self, to: &str) -> Self {
        Self {
            form_action: Some(to.to_owned()),
            ..self
        }
    }

    /// The element the configuration lists as forbidden.
    #[must_use]
    pub fn forbidden_listed(self) -> Self {
        Self {
            forbidden_listed: true,
            ..self
        }
    }

    /// The element the configuration lists as allowed.
    #[must_use]
    pub fn allowed_listed(self) -> Self {
        Self {
            allowed_listed: true,
            ..self
        }
    }

    /// The element with the reference `reference`.
    #[must_use]
    pub fn node(self, reference: &str) -> Self {
        Self {
            node: Some(reference.to_owned()),
            ..self
        }
    }

    fn node_of((spec, position): (&Self, usize)) -> Node {
        Node {
            role: spec.role.clone(),
            name: spec.name.clone(),
            node: spec.node.clone().unwrap_or_else(|| format!("e{position}")),
        }
    }

    fn candidate(&self, page: &Url) -> Candidate {
        let read = |to: &String| found(page.join(to).ok(), "the link reads against the page");
        Candidate {
            role: self.role.clone(),
            name: self.name.clone(),
            href: self.href.as_ref().map(read),
            form_action: self.form_action.as_ref().map(read),
            forbidden_listed: self.forbidden_listed,
            allowed_listed: self.allowed_listed,
        }
    }
}

/// The element at `position`: 40px apart from the top, or below the viewport.
fn button_names(shown: &[&ElementSpec]) -> Vec<String> {
    let buttons = shown.iter().filter(|e| e.role == "button");
    buttons.map(|e| e.name.clone()).collect()
}

fn shape((spec, position): (&ElementSpec, i64)) -> Element {
    placed(&spec.role, &spec.name, spec.below, position)
}

fn heading((heading, position): (&(String, bool), i64)) -> Element {
    placed("heading", &heading.0, heading.1, position)
}

fn placed(role: &str, name: &str, below: bool, position: i64) -> Element {
    let top = if below { BELOW } else { 0 };
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: None,
        rect: Some(Rect {
            x: 0,
            y: top + 40 * position,
            width: 100,
            height: 20,
        }),
    }
}

#[path = "page_checks.rs"]
pub mod checks;
