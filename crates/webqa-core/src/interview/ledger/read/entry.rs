//! Reading one issue: its first line and its fourteen fields.

use super::value::{At, Field, Lines, count, detail, json, named};
use crate::interview::issue::{
    Act, Count, Issue, KINDS, Kind, Number, Place, ScreenMark, Shot, Target,
};
use crate::interview::ledger::label::{COLON, COMPARED, FIELD, FIELDS, PATTERN_MARK};
use crate::interview::ledger::wire::{WireAct, WireShot, WireTarget, WireTrait};
use crate::interview::ledger::{Compared, Entry, LedgerError};
use crate::interview::natural::{PersonaTrait, Section};

/// The checkboxes, and whether each means selected.
const BOXES: [(&str, bool); 3] = [("- [ ] ", false), ("- [x] ", true), ("- [X] ", true)];

/// The length of a written number.
const NUMBER: usize = 10;

/// The values of the fields, in the order of [`FIELDS`].
type Fields<'a> = [Field<'a>; 14];

/// What follows the place: the screenshot, the path, the count and the
/// confidence.
type Tail = (Option<Shot>, Vec<Act>, Count, Option<f64>);

/// An issue of `section`: its `first` line, taken already, and its fields.
pub(super) fn entry(
    lines: &mut Lines<'_>,
    first: Field<'_>,
    section: Section,
) -> Result<Entry, LedgerError> {
    let (selected, number) = first_line(first)?;
    let fields = fields(lines)?;
    let issue = issue(number, &fields)?;
    annotations(&fields).map(|(persona_trait, compared)| Entry {
        selected,
        section,
        issue,
        persona_trait,
        compared,
    })
}

/// The checkbox and the number. The instruction after them is not read.
fn first_line((text, line): Field<'_>) -> Result<(bool, Number), LedgerError> {
    checkbox(text).at(line)
}

/// The checkbox and the number at the start of `text`.
fn checkbox(text: &str) -> Option<(bool, Number)> {
    let (selected, rest) = BOXES
        .iter()
        .find_map(|&(mark, selected)| text.strip_prefix(mark).map(|rest| (selected, rest)))?;
    Some((selected, number(rest)?))
}

/// The number at the start of `text`, followed by the end or a space.
fn number(text: &str) -> Option<Number> {
    let ended = text.get(NUMBER..).is_some_and(ends_number);
    text.get(..NUMBER).filter(|_| ended).and_then(Number::parse)
}

/// Whether the number is followed by the end of the line or a space.
fn ends_number(after: &str) -> bool {
    after.chars().next().is_none_or(|c| c == ' ')
}

/// The values of the fields, each with its line.
fn fields<'a>(lines: &mut Lines<'a>) -> Result<Fields<'a>, LedgerError> {
    let mut fields = [("", 0); 14];
    for (slot, name) in fields.iter_mut().zip(FIELDS) {
        *slot = field(lines, name)?;
    }
    Ok(fields)
}

/// The value of the field `name` on the next line.
fn field<'a>(lines: &mut Lines<'a>, name: &str) -> Result<Field<'a>, LedgerError> {
    let (text, line) = lines.after(FIELD)?;
    let value = text
        .strip_prefix(name)
        .and_then(|rest| rest.strip_prefix(COLON))
        .at(line)?;
    Ok((value, line))
}

/// The issue numbered `number` from its fields.
fn issue(number: Number, fields: &Fields<'_>) -> Result<Issue, LedgerError> {
    let [
        kind,
        screen,
        mark,
        element,
        _,
        shot,
        path,
        _,
        reason,
        expectation,
        occurred,
        reached,
        confidence,
        _,
    ] = *fields;
    named(&KINDS, kind).and_then(|kind| {
        let (place, target) = spot(kind, screen, mark, element)?;
        let detail = detail(kind, reason, expectation)?;
        let (shot, path, count, confidence) = tail(shot, path, [occurred, reached], confidence)?;
        Ok(Issue {
            number,
            place,
            target,
            detail,
            count,
            shot,
            path,
            confidence,
        })
    })
}

/// The place and the element.
fn spot(
    kind: Kind,
    screen: Field<'_>,
    mark: Field<'_>,
    element: Field<'_>,
) -> Result<(Place, Option<Target>), LedgerError> {
    let place = json::<String>(screen).and_then(|url| place(kind, url, mark))?;
    json::<Option<WireTarget>>(element).map(|target| (place, target.map(Target::from)))
}

/// A URL pattern for a mismatch, a screen with its mark otherwise.
fn place(kind: Kind, url: String, (mark, line): Field<'_>) -> Result<Place, LedgerError> {
    let place = if kind == Kind::Mismatch {
        (mark == PATTERN_MARK).then_some(Place::Pattern(url))
    } else {
        ScreenMark::parse(mark).map(|mark| Place::Screen { url, mark })
    };
    place.at(line)
}

/// The screenshot, the path, the count and the confidence.
fn tail(
    shot: Field<'_>,
    path: Field<'_>,
    [occurred, reached]: [Field<'_>; 2],
    confidence: Field<'_>,
) -> Result<Tail, LedgerError> {
    let shot = json::<Option<WireShot>>(shot)?.map(Shot::from);
    let (path, count) = path_count(path, occurred, reached)?;
    json(confidence).map(|confidence| (shot, path, count, confidence))
}

/// The path and the count.
fn path_count(
    path: Field<'_>,
    occurred: Field<'_>,
    reached: Field<'_>,
) -> Result<(Vec<Act>, Count), LedgerError> {
    let path = json::<Vec<WireAct>>(path).map(|path| path.into_iter().map(Act::from).collect());
    path.and_then(|path| count(occurred, reached).map(|count| (path, count)))
}

/// The trait and the comparison.
fn annotations(fields: &Fields<'_>) -> Result<(Option<PersonaTrait>, Compared), LedgerError> {
    let [_, _, _, _, _, _, _, cited, _, _, _, _, _, compared] = *fields;
    let wire = json::<Option<WireTrait>>(cited);
    let persona_trait = wire.and_then(|wire| wire.map(|it| it.cited().at(cited.1)).transpose());
    persona_trait.and_then(|it| named(&COMPARED, compared).map(|compared| (it, compared)))
}
