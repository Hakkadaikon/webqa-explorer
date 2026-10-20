//! Tests of how the steps of a trial read the operations the judge chose.

use std::time::Duration;

use webqa_core::config::raw::Op;
use webqa_core::interview::answer::Reply;
use webqa_core::interview::clue::Thresholds;
use webqa_core::judge::{Usage, Verdict};
use webqa_core::machine::Cause;
use webqa_core::screen::{ElementId, Viewport, element_ids};

use super::super::record::steps;
use super::*;
use crate::record::Act;
use crate::run::Taken;

const SEEN: Viewport = Viewport {
    width: 100,
    height: 100,
};

fn thresholds() -> Thresholds {
    Thresholds {
        hesitation: 0.5,
        confidence: 0.5,
    }
}

fn reply() -> Reply {
    Reply {
        verdict: Verdict {
            op: Op::Click,
            confidence: Some(0.9),
            targets: Vec::new(),
            error_shown: None,
            goal_reached: None,
            usage: Usage::default(),
        },
        hesitation: 0.0,
        expectation: String::new(),
        hard_to_see: None,
        purpose_guess: String::new(),
    }
}

/// The page of the elements, 20px apart from the top. The first one has no
/// box on the screen, and the last one is below the viewport.
fn taken(step: u64, elements: &[(&str, &str)]) -> Taken {
    let mut screen = fakes::page(&Url::parse(ORIGIN).unwrap(), elements).screen;
    screen.elements[0].rect = None;
    Taken {
        step,
        screen: Redactor::default().screen(screen),
        viewport: SEEN,
        reply: Some(reply()),
        image: None,
    }
}

fn act(step: u64, target: Option<ElementId>, cause: Cause) -> Act {
    Act {
        step,
        op: "click".to_owned(),
        target,
        p: 1.0,
        cause,
        act: Duration::ZERO,
        changed: false,
        settle: Duration::ZERO,
    }
}

/// The page: a hidden button, two buttons of one name and a link, then a
/// button below the viewport.
fn page(step: u64) -> Taken {
    let elements = [
        ("button", "隠れ"),
        ("button", "保存"),
        ("link", "ヘルプ"),
        ("button", "保存"),
        ("button", "下"),
        ("button", "下"),
    ];
    let mut at = taken(step, &elements);
    at.screen = {
        let mut screen = at.screen.as_inner().clone();
        screen.elements[5].rect.as_mut().unwrap().y = 100;
        Redactor::default().screen(screen)
    };
    at
}

/// The number the step of `at` gives the element `index` of its page.
fn numbered(index: usize) -> Option<usize> {
    let at = page(1);
    let target = element_ids(&at.screen)[index].clone();
    let acts = [act(1, Some(target), Cause::Judge)];
    let steps = steps(vec![at], &acts, thresholds());
    steps[0].ran.as_ref().unwrap().target
}

#[test]
fn element_is_numbered_from_one_among_the_ones_in_the_viewport() {
    // The hidden button has no number, so the others count from it on.
    let numbers: Vec<_> = (0..6).map(numbered).collect();
    assert_eq!(
        numbers,
        [None, Some(1), Some(2), Some(3), Some(4), None],
        "hidden and below the viewport have none; the same name is told by its place"
    );
}

#[test]
fn operation_without_an_element_has_no_number() {
    let acts = [act(1, None, Cause::Judge)];
    let steps = steps(vec![page(1)], &acts, thresholds());
    assert_eq!(steps[0].ran.as_ref().unwrap().target, None);
}

#[test]
fn each_step_reads_the_operation_of_its_own_step() {
    let first = element_ids(&page(1).screen)[1].clone();
    let second = element_ids(&page(2).screen)[2].clone();
    let acts = [
        act(1, Some(second.clone()), Cause::Paywall),
        act(2, Some(second), Cause::Judge),
        act(1, Some(first), Cause::Judge),
    ];
    let steps = steps(vec![page(1), page(2)], &acts, thresholds());
    let targets: Vec<_> = steps
        .iter()
        .map(|s| s.ran.as_ref().unwrap().target)
        .collect();
    assert_eq!(
        targets,
        [Some(1), Some(2)],
        "the operation the judge chose at the step, not another step's or the core's"
    );
}
