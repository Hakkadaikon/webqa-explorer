//! Reading a ledger the run wrote, and the numbers of its issues.
//!
//! The numbers in the feature (`I-0001`) are examples. A number of the
//! product is `I-` and 8 hex digits made from the screen, the element and
//! the kind, so a step binds each example to the number it computes, and
//! the steps after it say the example again.

use std::collections::BTreeMap;

use webqa_core::interview::issue::{Key, Kind, Number, Spot};
use webqa_core::interview::ledger::{Ledger, read};
use webqa_core::screen::{ElementId, ScreenId};

use crate::scene::found;

/// The ledger of `text`. A ledger that cannot be read fails the scenario.
pub fn read_back(text: &str) -> Ledger {
    let ledger = read(text);
    assert!(
        ledger.is_ok(),
        "the ledger reads back: {:?}",
        ledger.as_ref().err()
    );
    found(ledger.ok(), "the ledger reads back")
}

/// The number of the issue of `kind` on the screen at `path` with the
/// `elements` (role, name), about `target` if it has an element.
pub fn number_of(
    path: &str,
    elements: &[(&str, &str)],
    target: Option<(&str, &str, usize)>,
    kind: Kind,
) -> Number {
    let screen = ScreenId {
        path: path.to_owned(),
        elements: elements
            .iter()
            .map(|&(role, name)| (role.to_owned(), name.to_owned()))
            .collect(),
    };
    let element = target.map(|(role, name, nth)| ElementId {
        role: role.to_owned(),
        name: name.to_owned(),
        nth,
    });
    let spot = Spot::Screen {
        screen: &screen,
        element: element.as_ref(),
    };
    Number::of(Key { spot, kind })
}

/// The numbers the examples of the feature stand for.
#[derive(Debug, Default)]
pub struct Aliases(BTreeMap<String, Number>);

impl Aliases {
    /// Makes `example` stand for `number`.
    pub fn bind(&mut self, example: &str, number: Number) {
        self.0.insert(example.to_owned(), number);
    }

    /// The number `example` stands for. An example never bound fails the
    /// scenario.
    pub fn number(&self, example: &str) -> Number {
        found(self.0.get(example).copied(), "the example number is bound")
    }

    /// The number as the ledger writes it.
    pub fn written(&self, example: &str) -> String {
        self.number(example).to_string()
    }
}
