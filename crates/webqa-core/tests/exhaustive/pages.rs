//! The pages the browser may show, as observations.

use url::Url;
use webqa_core::machine::{Material, Seen, Whole};
use webqa_core::redact::Redactor;
use webqa_core::screen::{Element, Rect, Screen, Viewport};

/// The start page, with a forbidden button third.
pub const TOP: usize = 0;
/// The settings page.
pub const SETTINGS: usize = 1;
/// The paywall, found by its URL.
pub const PRICING: usize = 2;
/// The page outside the allowed origin.
pub const AWAY: usize = 3;
/// The page with only a forbidden button.
pub const CART: usize = 5;

/// The pages whose operated marks are read: those with elements, the
/// start page with "保存" replaced being the same screen as the start page.
pub const MARKED: [usize; 4] = [TOP, SETTINGS, AWAY, CART];
/// The positions of the start page and of the cart in [`MARKED`].
pub const TOP_MARKS: usize = 0;
pub const CART_MARKS: usize = 3;

/// The name of the forbidden button.
pub const BUY: &str = "購入する";

const VIEWPORT: Viewport = Viewport {
    width: 800,
    height: 600,
};

/// The URL of the page outside the allowed origin.
pub const AWAY_URL: &str = "https://other.example.net/";

/// An element inside the viewport.
fn element((role, name): &(&str, &str)) -> Element {
    Element {
        role: (*role).to_owned(),
        name: (*name).to_owned(),
        value: None,
        rect: Some(Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        }),
    }
}

/// A page at `url` with the elements, all inside the viewport.
fn screen(url: Url, elements: &[(&str, &str)]) -> Screen {
    Screen {
        url,
        elements: elements.iter().map(element).collect(),
    }
}

/// The observation of `screen`, with the material and the whole page made
/// from it.
fn seen(screen: Screen) -> Seen {
    let material = Material::plain(&screen, VIEWPORT);
    let screen = Redactor::default().screen(screen);
    Seen {
        whole: Whole::plain(&screen),
        screen,
        viewport: VIEWPORT,
        material,
    }
}

/// `seen` with the reference of "保存" changed, as if another node took its place.
fn swapped(mut seen: Seen) -> Seen {
    seen.material
        .nodes
        .iter_mut()
        .filter(|node| node.name == "保存")
        .for_each(|node| "x0".clone_into(&mut node.node));
    seen
}

/// The start page: "保存", "ヘルプ" and the forbidden button.
const TOP_ELEMENTS: [(&str, &str); 3] = [("button", "保存"), ("link", "ヘルプ"), ("button", BUY)];

/// The URLs and the elements of the pages, in the order of the positions.
const PAGES: [(&str, &[(&str, &str)]); 6] = [
    ("https://app.example.test/", &TOP_ELEMENTS),
    ("https://app.example.test/settings", &[("button", "保存")]),
    ("https://app.example.test/pricing", &[]),
    (AWAY_URL, &[("link", "ヘルプ")]),
    ("https://app.example.test/empty", &[]),
    ("https://app.example.test/cart", &[("button", BUY)]),
];

/// The pages: the start page, settings, the paywall, outside, empty, only
/// the forbidden button, and the start page with "保存" replaced.
pub fn seen_all() -> Vec<Seen> {
    let screens: Vec<Screen> = PAGES
        .iter()
        .flat_map(|(url, elements)| Url::parse(url).map(|url| screen(url, elements)))
        .collect();
    let replaced = screens.first().cloned().map(seen).map(swapped);
    screens.into_iter().map(seen).chain(replaced).collect()
}
