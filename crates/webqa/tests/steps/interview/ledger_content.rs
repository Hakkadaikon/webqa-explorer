//! Steps of the Rule 台帳の書式 about a ledger read back, and about what
//! the judge's sentences do to its lines.

use cucumber::{given, then, when};
use webqa_core::interview::ask::EXPECTATION;
use webqa_core::interview::clue::Clue;
use webqa_core::interview::issue::{Detail, Kind, Number};
use webqa_core::interview::ledger::{set_selection, write};
use webqa_core::interview::natural::Section;
use webqa_core::judge::QuestionId;
use webqa_core::redact::Redactor;

use super::text::{self, blocks, stray};
use super::{HOME, NEXT, hesitation_number, home};
use crate::issues::script::{Step, entries, journey, ledger};
use crate::ledger::{number_of, read_back};
use crate::scene::found;
use crate::world::InterviewWorld;

/// The buttons of the scene of the three issues.
const THREE: [&str; 2] = [NEXT, "戻る"];

/// The numbers of the three issues of the scene: the hesitation about
/// each button, and the trials that did not reach the purpose.
fn three_numbers() -> [Number; 3] {
    let elements = THREE.map(|name| ("button", name));
    let hesitation = |name| number_of("/", &elements, Some(("button", name, 0)), Kind::Hesitant);
    let unreached = number_of("/", &elements, None, Kind::Unreached);
    [hesitation(THREE[0]), hesitation(THREE[1]), unreached]
}

#[given(expr = "課題が3件あり、そのうち1件を選んだ台帳がある")]
async fn three_one_selected(world: &mut InterviewWorld) {
    let steps = [
        Step::click(1).hesitating(0.9),
        Step::click(2).hesitating(0.9),
        Step::of("stuck"),
    ];
    journey(&mut world.scene, &[home(&THREE)], &steps);
    world.run().await;
    let text = text::text(world);
    let selected = set_selection(&text, three_numbers()[1], true);
    let ran = found(world.ran.as_mut(), "the scene ran");
    ran.store.ledger = selected.ok();
}

#[then(expr = "3件の課題の番号、種類、項目の中身が書いたときと同じに戻る")]
async fn three_come_back(world: &mut InterviewWorld) {
    world.run().await;
    let text = text::text(world);
    let read = read_back(&text);
    let mut numbers: Vec<Number> = read.entries.iter().map(|e| e.issue.number).collect();
    numbers.sort();
    let mut expected = three_numbers();
    expected.sort();
    assert_eq!(numbers, expected, "the numbers");
    let of = |kind: Kind| read.entries.iter().filter(move |e| e.issue.kind() == kind);
    assert_eq!(of(Kind::Hesitant).count(), 2, "the two hesitations");
    assert_eq!(of(Kind::Unreached).count(), 1, "the purpose not reached");
    assert_eq!(
        write(&read, &Redactor::default()),
        text,
        "every item as it was written"
    );
}

#[then(expr = "選んだ課題は1件だけである")]
async fn one_selected(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(
        ledger(world).selected(),
        [three_numbers()[1]],
        "the selected issue"
    );
}

#[given(expr = "判断モデルが、期待の文として {string} を返す")]
fn expects(world: &mut InterviewWorld, text: String) {
    let steps = [
        Step::click(1).hesitating(0.9).expecting(&text),
        Step::of("done"),
    ];
    journey(&mut world.scene, &[home(&[NEXT])], &steps);
    drop(text);
}

#[when(expr = "台帳を書き、それを読み戻す")]
async fn writes_and_reads(world: &mut InterviewWorld) {
    world.run().await;
    assert!(world.ledger().is_some(), "the ledger reads back");
}

#[then(expr = "読み戻した台帳の課題の数と選択は、書いたときと同じである")]
async fn count_and_selection_stay(world: &mut InterviewWorld) {
    world.run().await;
    let ledger = ledger(world);
    assert_eq!(ledger.entries.len(), 1, "one issue as written");
    assert_eq!(ledger.selected(), [], "none of them selected");
    let sections: Vec<Section> = ledger.entries.iter().map(|e| e.section).collect();
    assert_eq!(
        sections,
        [Section::Body],
        "in the section it was written in"
    );
}

#[then(expr = "その文は1つの課題の期待として、元の文字列のまま読み戻せる")]
async fn sentence_stays(world: &mut InterviewWorld) {
    world.run().await;
    let sent = found(world.scene.replies.front(), "the scene has a reply");
    let sent = found(sent.as_ref().ok(), "the reply answers");
    let id = QuestionId::new(EXPECTATION);
    let text = found(sent.text(&id), "an expectation");
    let expected = Detail::Hesitant {
        clues: vec![Clue::Hesitant],
        expectation: text.to_owned(),
    };
    let details: Vec<Detail> = entries(world).into_iter().map(|e| e.issue.detail).collect();
    assert_eq!(
        details,
        [expected],
        "the one issue, with the sentence as written"
    );
}

#[given(expr = "直す指示の文の元になる判断モデルの文に改行が入っている")]
fn expects_lines(world: &mut InterviewWorld) {
    let steps = [
        Step::click(1)
            .hesitating(0.9)
            .expecting("1行目\n2行目\r\n3行目"),
        Step::of("done"),
    ];
    journey(&mut world.scene, &[home(&[NEXT])], &steps);
}

#[then(expr = "課題の1行目の直す指示の文に改行は無く、2行目以降に指示の続きは書かれていない")]
async fn instruction_is_one_line(world: &mut InterviewWorld) {
    world.run().await;
    let text = text::text(world);
    let all = blocks(&text);
    let firsts: Vec<&str> = all.iter().map(|b| b.first.as_str()).collect();
    let first = format!(
        "- [ ] {} {HOME} の画面で、button \"{NEXT}\" について、このペルソナが迷わずに次の操作を選べるようにする(期待: 1行目 2行目  3行目)",
        hesitation_number()
    );
    assert_eq!(firsts, [first.as_str()], "the instruction is in one line");
    let fields = all.iter().flat_map(|b| b.names()).count();
    assert_eq!(fields, 14, "the fixed fields follow it");
    assert_eq!(
        stray(&text),
        Vec::<String>::new(),
        "no line goes on with it"
    );
}
