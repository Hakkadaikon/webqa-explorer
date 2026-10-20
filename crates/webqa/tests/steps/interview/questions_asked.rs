//! What a step asks the judge: the persona, the questions added to the
//! four of every mode and the purpose.

use cucumber::gherkin::Step;
use cucumber::{given, then};
use webqa_core::config::purpose::{Aim, Written};
use webqa_core::config::raw::{DoneWhen, Element};

use super::replies::{SAVE, button, first_call, options_of, prompt};
use crate::judge::{Asked, Kind};
use crate::scene::found;
use crate::world::InterviewWorld;

/// The items of the table of the feature: the id and the kind of the
/// question each stands for.
const ITEMS: [(&str, &str, Kind); 8] = [
    ("操作種別", "operation", Kind::Choice),
    ("対象要素", "target", Kind::Choice),
    ("エラー表示の有無", "error_shown", Kind::Truth),
    ("ゴール達成の有無", "goal_reached", Kind::Truth),
    ("次に何をするかで迷った度合い", "hesitation", Kind::Truth),
    ("何を期待してその操作を選んだか", "expectation", Kind::Text),
    (
        "見えにくい、目立たない、読みにくい要素",
        "hard_to_see",
        Kind::Text,
    ),
    ("画面の目的の推測", "purpose_guess", Kind::Text),
];

/// The lines of the persona that lead every question: what the document
/// of the Background says.
fn persona_lines(world: &InterviewWorld) -> Vec<String> {
    let persona = &world.scene.persona;
    let avoids: Vec<String> = persona.avoids.iter().map(|a| format!("- {a}")).collect();
    let lines = [
        format!("名前: {}", persona.name),
        format!("どんな人か: {}", persona.profile),
        format!("ITの慣れ: {}", persona.it_literacy),
        format!("達成したいこと: {}", persona.wants),
        "しないこと:".to_owned(),
    ];
    lines.into_iter().chain(avoids).collect()
}

/// Every question of the first call has `line` as a line of its prompt.
fn leads_with(world: &InterviewWorld, lines: &[String]) {
    for question in &first_call(world).questions {
        let prompt: Vec<&str> = prompt(question).lines().collect();
        for line in lines {
            assert!(prompt.contains(&line.as_str()), "{line} leads the prompt");
        }
    }
}

#[then(
    expr = "判断モデルへの問いに、ペルソナの文書の名前、どんな人か、ITの慣れ、達成したいこと、しないことが添えられている"
)]
async fn persona_leads(world: &mut InterviewWorld) {
    world.run().await;
    leads_with(world, &persona_lines(world));
}

#[then(expr = "問いは、このペルソナならどうするかを尋ねる形である")]
async fn asks_as_persona(world: &mut InterviewWorld) {
    world.run().await;
    let ask = "この人ならどうするか、どう感じるかで答える。".to_owned();
    leads_with(world, &[ask]);
}

/// The id and the kind of each question the table of the step lists, in
/// the order of the table, without its header.
fn listed(step: &Step) -> Vec<(String, Kind)> {
    let table = found(step.table.as_ref(), "the step has a table");
    let item = |row: &Vec<String>| {
        let name = found(row.first(), "a row has an item");
        let (_, id, kind) = found(ITEMS.iter().find(|(item, ..)| item == name), "a known item");
        ((*id).to_owned(), *kind)
    };
    table.rows.iter().skip(1).map(item).collect()
}

/// The id and the kind of each question of `asked`, in order.
fn sent(asked: &Asked) -> Vec<(String, Kind)> {
    let ids = asked.questions.iter().map(|q| q.id().as_str().to_owned());
    ids.zip(asked.kinds()).collect()
}

#[then(expr = "判断モデルに次の項目を1回の問い合わせで問うている")]
async fn asks_items(world: &mut InterviewWorld, step: &Step) {
    world.run().await;
    let asked = first_call(world);
    assert_eq!(sent(asked), listed(step), "the questions of the one call");
    assert_eq!(world.ran().judge.asked.len(), 1, "the one call");
    let ops = [
        "click",
        "input",
        "scroll_down",
        "scroll_up",
        "back",
        "enter",
        "wait",
        "done",
        "stuck",
    ];
    assert_eq!(
        options_of(asked, "operation"),
        ops,
        "the kinds of operation"
    );
    assert_eq!(
        options_of(asked, "target"),
        ["1"],
        "the numbers of the elements"
    );
}

#[given(expr = "目的として {string} とその done_when を書いている")]
fn purpose_written(world: &mut InterviewWorld, goal: String) {
    let element = Element {
        role: "button".to_owned(),
        name: "注文を確定する".to_owned(),
    };
    let done_when = DoneWhen {
        url: None,
        element: Some(element),
        text: None,
    };
    world.scene.purpose.aim = Aim::Written(Written { goal, done_when });
    button(&mut world.scene, SAVE);
}

#[then(expr = "判断モデルへの問いに、画面の目的の推測が含まれる")]
async fn asks_purpose_guess(world: &mut InterviewWorld) {
    world.run().await;
    let asked = first_call(world);
    let guess = ("purpose_guess".to_owned(), Kind::Text);
    assert!(
        sent(asked).contains(&guess),
        "the guess is asked as a sentence"
    );
    let goal = format!("目指すこと: {}", world.scene.purpose.stage().goal);
    leads_with(world, &[goal]);
}
