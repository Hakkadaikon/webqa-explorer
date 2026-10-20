//! The scripts the scenarios of the issues and of the ledger run: the pages
//! of a trial, and what the judge answers at each of its steps.

use std::iter;

use serde_json::Value;
use webqa_core::interview::ask::{EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS};
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::{Entry, Ledger};
use webqa_core::interview::natural::{NATURAL, PERSONA_TRAIT, YES};

use crate::judge::Said;
use crate::page::PageSpec;
use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// How many answers to the naturalness questions a script ends with. More
/// than the issues of any scenario need; the ones left over are not read.
const NATURALS: usize = 8;

/// The trait of the persona the naturalness answers name.
const TRAIT: &str = "ITの慣れ";

/// The screens the feature names, with the path of each.
const SCREENS: [(&str, &str); 5] = [
    ("商品一覧", "/products"),
    ("商品詳細", "/products/1"),
    ("配送先の入力", "/shipping"),
    ("注文確認", "/confirm"),
    ("カート", "/cart"),
];

/// The path of the screen the feature calls `name`.
pub fn path_of(name: &str) -> &'static str {
    let row = SCREENS.iter().find(|(screen, _)| *screen == name);
    found(row, "the feature names a known screen").1
}

/// The screen `name` with a button for each of `buttons`, in order.
pub fn screen(name: &str, buttons: &[&str]) -> PageSpec {
    let page = PageSpec::at(path_of(name));
    buttons
        .iter()
        .fold(page, |page, button| page.with(PageSpec::button(button)))
}

/// What the judge answers at one step.
#[derive(Debug, Clone)]
pub struct Step {
    op: &'static str,
    target: Option<usize>,
    hesitation: f64,
    confidence: f64,
    expectation: String,
    hard_to_see: Option<String>,
}

impl Step {
    /// A step that does `op` and is sure of it, without hesitation.
    pub fn of(op: &'static str) -> Self {
        Self {
            op,
            target: None,
            hesitation: 0.0,
            confidence: 0.9,
            expectation: "進む".to_owned(),
            hard_to_see: Some(String::new()),
        }
    }

    /// A step that clicks the element number `target` (from 1).
    pub fn click(target: usize) -> Self {
        Self {
            target: Some(target),
            ..Self::of("click")
        }
    }

    /// The step, with the hesitation `degree`.
    #[must_use]
    pub fn hesitating(self, degree: f64) -> Self {
        Self {
            hesitation: degree,
            ..self
        }
    }

    /// The step, with the confidence `p` of its operation.
    #[must_use]
    pub fn sure(self, p: f64) -> Self {
        Self {
            confidence: p,
            ..self
        }
    }

    /// The step, expecting `text`.
    #[must_use]
    pub fn expecting(self, text: &str) -> Self {
        Self {
            expectation: text.to_owned(),
            ..self
        }
    }

    /// The step, listing `lines` as hard to see (`number: reason`).
    #[must_use]
    pub fn seeing(self, lines: &str) -> Self {
        Self {
            hard_to_see: Some(lines.to_owned()),
            ..self
        }
    }

    /// The step, whose answers leave out the elements hard to see even
    /// where they are asked.
    #[must_use]
    pub fn without_sight(self) -> Self {
        Self {
            hard_to_see: None,
            ..self
        }
    }

    /// The answers of the step. The elements hard to see are answered
    /// only where the judge reads images, as only there are they asked.
    fn said(&self, sight: bool) -> Said {
        let said = Said::default()
            .choose("operation", self.op, self.confidence)
            .choose("error_shown", "true", 0.0)
            .choose("goal_reached", "true", 0.0)
            .choose(HESITATION, "true", self.hesitation)
            .write(EXPECTATION, &self.expectation)
            .write(PURPOSE_GUESS, "商品を買う");
        let said = self
            .target
            .iter()
            .fold(said, |said, n| said.choose("target", &n.to_string(), 0.9));
        let seen = self.hard_to_see.as_deref().filter(|_| sight);
        seen.iter()
            .fold(said, |said, lines| said.write(HARD_TO_SEE, lines))
    }
}

/// Whether the judge of the scene says it reads images.
pub fn reads_images(scene: &Scene) -> bool {
    scene.bridge.unwrap_or(scene.images)
}

/// The answers to the naturalness questions of an issue: natural for the
/// persona.
fn natural() -> Said {
    Said::default()
        .choose(NATURAL, YES, 0.9)
        .choose(PERSONA_TRAIT, TRAIT, 0.9)
}

/// Makes the judge answer the steps of each trial in order, then
/// `naturals` times the naturalness questions.
fn answers(scene: &mut Scene, trials: &[Vec<Step>], naturals: usize) {
    let sight = reads_images(scene);
    let steps = trials.iter().flatten().map(|step| step.said(sight));
    let naturals = iter::repeat_n(natural(), naturals);
    scene.replies = steps.chain(naturals).map(|said| Ok(said.build())).collect();
}

/// Makes the judge answer the steps of each trial in order, then the
/// naturalness questions.
pub fn script(scene: &mut Scene, trials: &[Vec<Step>]) {
    answers(scene, trials, NATURALS);
}

/// The pages the browser shows one after the other while `steps` run on
/// `pages`: the page of each step, and the same page again where the step
/// operates on an element, as the page is fetched once more just before.
/// The last page stays.
fn shown(pages: &[PageSpec], steps: &[Step]) -> Vec<PageSpec> {
    let last = found(pages.last(), "a trial has a page");
    let shown = |(at, step): (usize, &Step)| {
        let page = pages.get(at).unwrap_or(last).clone();
        let again = step.target.map(|_| page.clone());
        iter::once(page).chain(again)
    };
    steps.iter().enumerate().flat_map(shown).collect()
}

/// Makes the trials show the pages and answer the steps of `runs`, one
/// run for each trial. They all start on the same page.
pub fn trials(scene: &mut Scene, runs: &[(Vec<PageSpec>, Vec<Step>)]) {
    let browsers: Vec<Vec<PageSpec>> = runs.iter().map(|(p, s)| shown(p, s)).collect();
    let starts: Vec<&PageSpec> = browsers.iter().filter_map(|pages| pages.first()).collect();
    scene.start = found(starts.first().copied(), "a trial has a page").clone();
    scene.after = browsers
        .iter()
        .map(|pages| pages.get(1..).unwrap_or_default().to_vec())
        .collect();
    let steps: Vec<Vec<Step>> = runs.iter().map(|(_, steps)| steps.clone()).collect();
    script(scene, &steps);
}

/// Makes every trial go over `pages` and answer `steps`.
pub fn journey(scene: &mut Scene, pages: &[PageSpec], steps: &[Step]) {
    let runs: Vec<_> = (0..scene.trials)
        .map(|_| (pages.to_vec(), steps.to_vec()))
        .collect();
    trials(scene, &runs);
}

/// As [`journey`], but only the first trial, which ends when the judge has
/// nothing more to answer.
pub fn open_journey(scene: &mut Scene, pages: &[PageSpec], steps: &[Step]) {
    trials(scene, &[(pages.to_vec(), steps.to_vec())]);
    answers(scene, &[steps.to_vec()], 0);
}

/// The ledger the run wrote.
pub fn ledger(world: &InterviewWorld) -> Ledger {
    found(world.ledger(), "the run wrote a ledger")
}

/// The issues of the ledger the run wrote.
pub fn entries(world: &InterviewWorld) -> Vec<Entry> {
    ledger(world).entries
}

/// The kinds of the issues of the ledger the run wrote, in its order.
pub fn kinds(world: &InterviewWorld) -> Vec<Kind> {
    entries(world)
        .iter()
        .map(|entry| entry.issue.kind())
        .collect()
}

/// How many trials the ledger says finished.
pub fn finished(world: &InterviewWorld) -> usize {
    ledger(world).head.finished
}

/// How each trial ended, as the log names it, in order.
pub fn endings(world: &InterviewWorld) -> Vec<Value> {
    let ended = |line: &Value| line.get("kind") == Some(&Value::from("trial_finished"));
    let lines = world.lines();
    let ends = lines.iter().filter(|line| ended(line));
    ends.filter_map(|line| line.get("ending").cloned())
        .collect()
}
