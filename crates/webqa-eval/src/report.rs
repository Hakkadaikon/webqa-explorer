//! The scores of several runs, as a Markdown table and two lists.

#[cfg(test)]
mod tests;

use std::fmt::Write as _;

use crate::found::Found;
use crate::score::{Score, float, score};
use webqa_core::interview::issue::Kind;

use crate::truth::{KindId, Label, Plant, Verdict};

const KINDS: [KindId; 4] = [
    KindId::Visibility,
    KindId::Hesitant,
    KindId::Unreached,
    KindId::Mismatch,
];

/// A table of recall and precision for each kind and the total, then the
/// plants no run found and the issues that hit no plant, with the ruling.
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
        let name = kind.map_or("全体", label);
        text.push_str(&row(name, &rows(kind)));
    }
    let all = rows(None);
    let _ = write!(
        text,
        "\n## 見つからなかった仕込み\n\n{}\n## 仕込みに当たらなかった課題\n\n{}",
        lines(&missed(plants, &all)),
        lines(&others(&all)),
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

fn label(kind: KindId) -> &'static str {
    Kind::from(kind).label()
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
    if values.is_empty() {
        return "-".to_owned();
    }
    let n = float(values.len());
    let mean = values.iter().sum::<f64>() / n;
    let spread =
        (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
    if values.len() > 1 {
        format!("{mean:.2} ± {spread:.2}")
    } else {
        format!("{mean:.2}")
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

/// The issues that hit no plant over the runs, each with how often it came,
/// the number of its first issue and the ruling on it, in the order they
/// first came.
fn others(scores: &[Score]) -> Vec<String> {
    let all: Vec<&Ruled> = scores.iter().flat_map(|s| &s.others).collect();
    distinct(&all)
        .iter()
        .map(|(found, verdict)| {
            let count = all.iter().filter(|(other, _)| same(other, found)).count();
            describe(found, *verdict, count)
        })
        .collect()
}

/// An issue with the ruling on it, if any.
type Ruled = (Found, Option<Verdict>);

/// The first of each issue, in order.
fn distinct<'a>(all: &[&'a Ruled]) -> Vec<&'a Ruled> {
    all.iter().fold(Vec::new(), |mut firsts, item| {
        if !firsts.iter().any(|first| same(&first.0, &item.0)) {
            firsts.push(item);
        }
        firsts
    })
}

fn same(a: &Found, b: &Found) -> bool {
    (a.kind, &a.path, &a.element) == (b.kind, &b.path, &b.element)
}

fn describe(found: &Found, verdict: Option<Verdict>, count: usize) -> String {
    let element = found
        .element
        .as_ref()
        .map_or_else(String::new, |e| format!(" {} {}", e.role, e.name));
    let ruling = match verdict {
        Some(Verdict::Real) => "本当に使いにくい",
        Some(Verdict::False) => "誤り",
        None => "未判定",
    };
    format!(
        "{} {}{element} {count}件 {} {ruling}",
        label(found.kind),
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
