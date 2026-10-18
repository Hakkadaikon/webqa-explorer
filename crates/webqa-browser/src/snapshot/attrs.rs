//! The ` [name=value]` attributes after a key.

use webqa_core::screen::Rect;

use super::quoted::Bad;

/// The attributes the reader uses; the others are skipped.
#[derive(Default)]
pub(super) struct Attrs {
    /// `[ref=…]`.
    pub node: Option<String>,
    /// `[box=x,y,width,height]`.
    pub rect: Option<Rect>,
    /// `[disabled]`.
    pub disabled: bool,
    /// `"checked"` for `[checked]`, `"mixed"` for `[checked=mixed]`.
    pub checked: Option<&'static str>,
    /// Whether a used attribute had a value that does not read.
    unreadable: bool,
}

/// Reads ` [attribute]` after ` [attribute]` up to the end of the key.
pub(super) fn read(mut rest: &str) -> Result<Attrs, Bad> {
    let mut attrs = Attrs::default();
    while !rest.is_empty() {
        rest = attrs.next(rest)?;
    }
    (!attrs.unreadable).then_some(attrs).ok_or(Bad)
}

type Apply = fn(&mut Attrs, Option<&str>);

/// What each used attribute sets, by its name.
const APPLY: [(&str, Apply); 4] = [
    ("ref", set_ref),
    ("box", set_box),
    ("disabled", set_disabled),
    ("checked", set_checked),
];

impl Attrs {
    /// Reads the first ` [attribute]` of `rest` and returns what follows it.
    fn next<'a>(&mut self, rest: &'a str) -> Result<&'a str, Bad> {
        let (attr, after) = rest
            .strip_prefix(" [")
            .and_then(|s| s.split_once(']'))
            .ok_or(Bad)?;
        self.apply(attr);
        Ok(after)
    }

    fn apply(&mut self, attr: &str) {
        let (name, value) = attr
            .split_once('=')
            .map_or((attr, None), |(n, v)| (n, Some(v)));
        if let Some((_, set)) = APPLY.iter().find(|(n, _)| *n == name) {
            set(self, value);
        }
    }
}

fn set_ref(attrs: &mut Attrs, value: Option<&str>) {
    attrs.node = value.map(str::to_owned);
}

fn set_box(attrs: &mut Attrs, value: Option<&str>) {
    let rect = value.and_then(rect);
    attrs.unreadable |= rect.is_none();
    attrs.rect = rect;
}

const fn set_disabled(attrs: &mut Attrs, _: Option<&str>) {
    attrs.disabled = true;
}

fn set_checked(attrs: &mut Attrs, value: Option<&str>) {
    attrs.checked = Some(value.map_or("checked", |_| "mixed"));
}

/// `x,y,width,height` as four integers.
fn rect(value: &str) -> Option<Rect> {
    let numbers: Vec<i64> = value
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let [x, y, width, height] = <[i64; 4]>::try_from(numbers).ok()?;
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}
