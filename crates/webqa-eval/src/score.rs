//! Matches the issues of a ledger with the answer key and the labels, and
//! counts recall and precision.

#[cfg(test)]
mod tests;

use crate::found::Found;
use crate::truth::{KindId, Label, Plant, Verdict};

/// What a run found, against the answer key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    /// The planted issues counted.
    pub plants: usize,
    /// The ids of the plants some issue hit.
    pub found_plants: Vec<String>,
    /// The ids of the plants no issue hit.
    pub missed_plants: Vec<String>,
    /// The issues counted.
    pub issues: usize,
    /// The issues that hit a plant.
    pub hit_issues: usize,
    /// The issues a human labeled as real.
    pub real: usize,
    /// The issues a human labeled as a mistake.
    pub wrong: usize,
    /// The issues that hit no plant and have no label.
    pub unlabeled: Vec<Found>,
}

/// Whether `found` is the issue `plant` planted: the same kind, and the
/// same path and element when the plant has them.
#[must_use]
pub fn hits(plant: &Plant, found: &Found) -> bool {
    plant.kind == found.kind
        && agrees(plant.path.as_ref(), Some(&found.path))
        && agrees(plant.element.as_ref(), found.element.as_ref())
}

/// Counts `found` against `plants` and `labels`. With a `kind`, only that
/// kind's plants and issues count.
#[must_use]
pub fn score(plants: &[Plant], labels: &[Label], found: &[Found], kind: Option<KindId>) -> Score {
    let plants = only(plants, kind);
    let found = only(found, kind);
    let (hit, rest) = split(&found, hit_issue(&plants));
    let (found_plants, missed_plants) = split(&plants, found_plant(&found));
    let (real, wrong, unlabeled) = judge(labels, &rest);
    Score {
        plants: plants.len(),
        found_plants: ids(&found_plants),
        missed_plants: ids(&missed_plants),
        issues: found.len(),
        hit_issues: hit.len(),
        real,
        wrong,
        unlabeled,
    }
}

/// The items of `kind`, or all of them without one.
fn only<T: Kinded>(items: &[T], kind: Option<KindId>) -> Vec<&T> {
    let counted = |item: &&T| kind.is_none_or(|only| only == item.kind());
    items.iter().filter(counted).collect()
}

trait Kinded {
    fn kind(&self) -> KindId;
}

impl Kinded for Plant {
    fn kind(&self) -> KindId {
        self.kind
    }
}

impl Kinded for Found {
    fn kind(&self) -> KindId {
        self.kind
    }
}

fn hit_issue<'a>(plants: &'a [&Plant]) -> impl Fn(&Found) -> bool + 'a {
    |found| plants.iter().any(|plant| hits(plant, found))
}

fn found_plant<'a>(found: &'a [&Found]) -> impl Fn(&Plant) -> bool + 'a {
    |plant| found.iter().any(|found| hits(plant, found))
}

fn split<'a, T>(items: &[&'a T], test: impl Fn(&T) -> bool) -> (Vec<&'a T>, Vec<&'a T>) {
    items.iter().partition(|item| test(item))
}

/// The issues rulings count as real and as mistakes, and the issues
/// without a ruling.
fn judge(labels: &[Label], rest: &[&Found]) -> (usize, usize, Vec<Found>) {
    let (ruled, unlabeled) = split(rest, |f| verdict(labels, f).is_some());
    let real = ruled
        .iter()
        .filter(|f| verdict(labels, f) == Some(Verdict::Real))
        .count();
    let unlabeled = unlabeled.into_iter().cloned().collect();
    (real, ruled.len() - real, unlabeled)
}

impl Score {
    /// Found plants over plants. `None` without plants.
    #[must_use]
    pub fn recall(&self) -> Option<f64> {
        ratio(self.found_plants.len(), self.plants)
    }

    /// Issues that hit a plant over issues. `None` without issues.
    #[must_use]
    pub fn strict_precision(&self) -> Option<f64> {
        ratio(self.hit_issues, self.issues)
    }

    /// Issues that hit a plant or are real, over the issues without a
    /// label that hit no plant. `None` when none is left.
    #[must_use]
    pub fn labeled_precision(&self) -> Option<f64> {
        ratio(
            self.hit_issues + self.real,
            self.issues - self.unlabeled.len(),
        )
    }
}

fn ids(plants: &[&Plant]) -> Vec<String> {
    plants.iter().map(|p| p.id.clone()).collect()
}

fn verdict(labels: &[Label], found: &Found) -> Option<Verdict> {
    labels
        .iter()
        .find(|label| rules(label, found))
        .map(|label| label.verdict)
}

fn rules(label: &Label, found: &Found) -> bool {
    label.kind == found.kind
        && agrees(label.path.as_ref(), Some(&found.path))
        && agrees(label.element.as_ref(), found.element.as_ref())
}

/// A wanted value of `None` agrees with anything.
fn agrees<T: PartialEq>(want: Option<&T>, got: Option<&T>) -> bool {
    want.is_none_or(|want| got == Some(want))
}

/// A count as a float. Counts past `u32::MAX` do not occur.
pub(crate) fn float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

fn ratio(part: usize, whole: usize) -> Option<f64> {
    (whole > 0).then(|| float(part) / float(whole))
}
