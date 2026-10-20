//! What the judge says of its choice: the expectation it writes, the
//! elements it may choose from and the kinds of operation.

use cucumber::{given, then};
use webqa_core::effect::Touch;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::Detail;
use webqa_core::screen::ElementId;

use super::replies::{
    entry_of, first_call, ledger_of, natural_yes, one_issue, options_of, reply, step,
};
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

#[given(expr = "判断モデルが、期待の問いに自由な文 {string} で答える")]
fn expects(world: &mut InterviewWorld, text: String) {
    one_issue(&mut world.scene, &text, Ok(natural_yes()));
    drop(text);
}

#[then(expr = "その文が、そのステップの期待として記録されている")]
async fn expectation_recorded(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger_of(world);
    let number = world.scene.aliases.number("その課題");
    let entry = entry_of(&ledger, number);
    let clues = vec![Clue::Hesitant];
    let expectation = "押すと注文が確定すると思った".to_owned();
    let detail = Detail::Hesitant { clues, expectation };
    assert_eq!(entry.issue.detail, detail, "what the persona expected");
}

#[given(expr = "表示領域に、操作済みの要素 {string} と未操作の要素 {string} がある")]
fn operated_and_not(world: &mut InterviewWorld, operated: String, fresh: String) {
    let scene = &mut world.scene;
    scene.start.elements.push(PageSpec::link(&operated));
    scene.start.elements.push(PageSpec::link(&fresh));
    // The first step operates the first element; the second is the one
    // that is asked.
    reply(scene, step("click", Some("1"), 0.0));
    reply(scene, step("click", Some("1"), 0.0));
    drop((operated, fresh));
}

#[then(expr = "対象要素の問いの選択肢に {string} と {string} の両方が含まれる")]
async fn both_offered(world: &mut InterviewWorld, operated: String, fresh: String) {
    world.run().await;
    let asked = found(world.ran().judge.decision(1), "the second step asked");
    assert_eq!(options_of(asked, "target"), ["1", "2"], "both are offered");
    // The numbered elements come first; a blank line ends them.
    let lines: Vec<&str> = asked.screen.lines().take_while(|l| !l.is_empty()).collect();
    let names = [
        format!(r#"1: link "{operated}""#),
        format!(r#"2: link "{fresh}""#),
    ];
    assert_eq!(lines, names, "the numbers are the two elements");
    drop((operated, fresh));
}

#[then(
    expr = "判断モデルが {string} を選べば、ガードレールを通ったあとに {string} をクリックしている"
)]
async fn clicks_after_guard(world: &mut InterviewWorld, chosen: String, clicked: String) {
    world.run().await;
    assert_eq!(chosen, clicked, "the element chosen is the one clicked");
    let target = ElementId {
        role: "link".to_owned(),
        name: clicked,
        nth: 0,
    };
    let click = Touch {
        op: "click".to_owned(),
        target: Some(target),
        confidence: 0.9,
    };
    let touches: Vec<&Touch> = world.ran().touches.iter().map(|(_, touch)| touch).collect();
    assert_eq!(
        touches,
        [&click, &click],
        "the first step and the chosen one"
    );
}

#[then(expr = "操作種別の問いの選択肢に done が含まれる")]
async fn done_offered(world: &mut InterviewWorld) {
    world.run().await;
    let options = options_of(first_call(world), "operation");
    assert!(
        options.contains(&"done".to_owned()),
        "done is offered: {options:?}"
    );
}
