//! Helpers of the Rule on memory: what the judge was offered, and the
//! memory a previous run left.

use webqa_core::judge::QuestionId;

use super::{MOVES, button};
use crate::page::{ElementSpec, PageSpec};
use crate::remembered::remember;
use crate::scene::found;
use crate::world::QaWorld;

/// The elements of the start page as (role, name).
pub(super) fn shown(world: &QaWorld) -> Vec<(String, String)> {
    let elements = &world.scene.start.elements;
    let pair = |e: &ElementSpec| (e.role.clone(), e.name.clone());
    elements.iter().map(pair).collect()
}

/// Puts the memory `screens` and the clicks `edges` in the scene.
fn remember_owned(
    world: &mut QaWorld,
    screens: &[(&str, Vec<(String, String)>)],
    edges: &[(usize, &str, usize)],
) {
    let lists: Vec<Vec<(&str, &str)>> = screens
        .iter()
        .map(|(_, list)| list.iter().map(|(r, n)| (r.as_str(), n.as_str())).collect())
        .collect();
    let given: Vec<(&str, &[(&str, &str)])> = screens
        .iter()
        .zip(&lists)
        .map(|((path, _), list)| (*path, list.as_slice()))
        .collect();
    remember(&mut world.scene, &given, edges);
}

/// The memory of the start page and the settings page, with the click
/// `pressed` from the one to the other.
pub(super) fn remember_pressed(world: &mut QaWorld, pressed: impl Into<String>) {
    let pressed = pressed.into();
    let screens = [
        ("/", shown(world)),
        ("/settings", vec![("button".to_owned(), "通知".to_owned())]),
    ];
    remember_owned(world, &screens, &[(0, &pressed, 1)]);
}

/// The options of the question `id` at the `k`-th call of the judge.
pub(super) fn options(world: &QaWorld, k: usize, id: &QuestionId) -> Vec<String> {
    let questions = found(world.ran().judge.questions.get(k), "the judge was asked");
    let asked = questions.iter().filter(|question| question.id() == id);
    let written = asked.filter_map(|question| serde_json::to_value(question).ok());
    let lists = written.filter_map(|value| value.get("options").cloned());
    lists
        .filter_map(|list| serde_json::from_value::<Vec<String>>(list).ok())
        .flatten()
        .collect()
}

/// The name of the element numbered `number` in the screen text.
pub(super) fn name_at(text: &str, number: &str) -> String {
    let prefix = format!("{number}: ");
    let line = text.lines().find(|line| line.starts_with(&prefix));
    let line = found(line, "the number is in the screen text");
    line.split('"').nth(1).unwrap_or_default().to_owned()
}

/// The names of the elements the judge was offered at the `k`-th call.
pub(super) fn offered_names(world: &QaWorld, k: usize) -> Vec<String> {
    let text = found(world.ran().judge.texts.get(k), "the judge was sent a text");
    let numbers = options(world, k, &QuestionId::target());
    numbers.iter().map(|number| name_at(text, number)).collect()
}

/// The operations the judge was offered at the `k`-th call.
pub(super) fn offered_ops(world: &QaWorld, k: usize) -> Vec<String> {
    options(world, k, &QuestionId::operation())
}

/// The last call of the judge.
pub(super) fn last_call(world: &QaWorld) -> usize {
    world.ran().judge.questions.len().saturating_sub(1)
}

/// The memory of the top page with the link `unoperated`, and the settings
/// page as the start page shows it; the settings button leads to itself.
pub(super) fn remember_back(world: &mut QaWorld, unoperated: impl Into<String>) {
    let top = vec![
        ("button".to_owned(), "保存".to_owned()),
        ("link".to_owned(), unoperated.into()),
    ];
    let screens = [("/", top), ("/settings", shown(world))];
    remember_owned(world, &screens, &[(0, "保存", 1), (1, "通知", 1)]);
}

/// The start page is the settings page with the buttons `names`.
pub(super) fn settings_with(world: &mut QaWorld, names: &[&str]) {
    let page = PageSpec::at("/settings");
    let shown = names
        .iter()
        .fold(page, |page, name| page.with(button(*name)));
    world.scene.start = shown;
}

/// The operations offered at the first call are some of the moves, and no
/// element is.
pub(super) fn assert_moves(world: &QaWorld) {
    let ops = offered_ops(world, 0);
    let strays = ops.iter().filter(|op| !MOVES.contains(&op.as_str()));
    assert_eq!(
        (ops.is_empty(), strays.count()),
        (false, 0),
        "only moves: {ops:?}"
    );
    let names = offered_names(world, 0);
    assert_eq!(names, Vec::<String>::new(), "no element is offered");
}
