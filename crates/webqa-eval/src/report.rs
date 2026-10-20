//! The scores of several runs, as a Markdown table and two lists.

#[cfg(test)]
mod tests;

use std::fmt::Write as _;

use crate::found::Found;
use crate::score::{Score, float, score};
use crate::truth::{KindId, Label, Plant};

const KINDS: [KindId; 4] = [
    KindId::Visibility,
    KindId::Hesitant,
    KindId::Unreached,
    KindId::Mismatch,
];

/// A table of recall and precision for each kind and the total, then the
/// plants no run found and the issues without a plant or a label.
///
/// It starts with the number of runs. A cell is the mean over the runs,
/// with the sample standard deviation when there are two values or more.
/// A run without a denominator is left out of the mean; a cell without
/// any is `-`.
#[must_use]
pub fn report(runs: &[Vec<Found>], plants: &[Plant], labels: &[Label]) -> String {
    let rows = |kind: Option<KindId>| scores(runs, plants, labels, kind);
    let mut text = format!(
        "採点した回: {}\n\n| 種類 | 再現率 | 厳しい適合率 | 人の判断を引いた適合率 |\n|---|---|---|---|\n",
        runs.len()
    );
    for kind in KINDS.map(Some).into_iter().chain([None]) {
        let name = kind.map_or("全体", name);
        text.push_str(&row(name, &rows(kind)));
    }
    let all = rows(None);
    let _ = write!(
        text,
        "\n## 見つからなかった仕込み\n\n{}\n## 未判定の課題\n\n{}",
        lines(&missed(plants, &all)),
        lines(&unlabeled(&all)),
    );
    text
}

fn scores(
    runs: &[Vec<Found>],
    plants: &[Plant],
    labels: &[Label],
    kind: Option<KindId>,
) -> Vec<Score> {
    runs.iter()
        .map(|found| score(plants, labels, found, kind))
        .collect()
}

const fn name(kind: KindId) -> &'static str {
    match kind {
        KindId::Visibility => "視認性が悪い",
        KindId::Hesitant => "判断に迷う",
        KindId::Unreached => "目的に届かない",
        KindId::Mismatch => "目的のずれ",
    }
}

fn row(name: &str, scores: &[Score]) -> String {
    let ratios: [fn(&Score) -> Option<f64>; 3] = [
        Score::recall,
        Score::strict_precision,
        Score::labeled_precision,
    ];
    let cells = ratios.map(|ratio| cell(&scores.iter().filter_map(ratio).collect::<Vec<_>>()));
    format!("| {name} | {} |\n", cells.join(" | "))
}

fn cell(values: &[f64]) -> String {
    let n = float(values.len());
    let mean = values.iter().sum::<f64>() / n;
    let spread =
        (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
    match (values.is_empty(), values.len() > 1) {
        (true, _) => "-".to_owned(),
        (false, false) => format!("{mean:.2}"),
        (false, true) => format!("{mean:.2} ± {spread:.2}"),
    }
}

fn missed(plants: &[Plant], scores: &[Score]) -> Vec<String> {
    let missed_by_all = |plant: &&Plant| scores.iter().all(|s| s.missed_plants.contains(&plant.id));
    plants
        .iter()
        .filter(missed_by_all)
        .map(|plant| plant.id.clone())
        .collect()
}

/// The unlabeled issues over the runs, each with how often it came and the
/// number of its first issue, in the order they first came.
fn unlabeled(scores: &[Score]) -> Vec<String> {
    let mut tally: Vec<(&Found, usize)> = vec![];
    scores
        .iter()
        .flat_map(|s| &s.unlabeled)
        .for_each(|found| count_in(&mut tally, found));
    tally
        .iter()
        .map(|(found, count)| describe(found, *count))
        .collect()
}

fn count_in<'a>(tally: &mut Vec<(&'a Found, usize)>, found: &'a Found) {
    match tally.iter_mut().find(|(seen, _)| same(seen, found)) {
        Some((_, count)) => *count += 1,
        None => tally.push((found, 1)),
    }
}

fn same(a: &Found, b: &Found) -> bool {
    (a.kind, &a.path, &a.element) == (b.kind, &b.path, &b.element)
}

fn describe(found: &Found, count: usize) -> String {
    let element = found
        .element
        .as_ref()
        .map_or_else(String::new, |e| format!(" {} {}", e.role, e.name));
    format!(
        "{} {}{element} {count}件 {}",
        name(found.kind),
        found.path,
        found.number
    )
}

/// A bullet list, or `- なし`, closed by a blank line.
fn lines(items: &[String]) -> String {
    if items.is_empty() {
        return "- なし\n".to_owned();
    }
    items
        .iter()
        .map(|item| ["- ", item, "\n"].concat())
        .collect()
}
