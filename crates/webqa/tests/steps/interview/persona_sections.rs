//! Where an issue goes in the ledger by the judge's answer about it, and
//! the trait of the persona it names.

use cucumber::{given, then};
use webqa_core::interview::natural::{NO, Section, Trait, YES};
use webqa_core::judge::JudgeError;

use super::{OFF_PERSONA, heading_at, issue_at, ledger_lines, section_of};
use crate::questions::replies::{IT_LITERACY, entry_of, ledger_of, natural, one_issue};
use crate::world::InterviewWorld;

#[given(expr = "自然さの問いに、判断モデルが自然だと答える")]
fn answers_natural(world: &mut InterviewWorld) {
    one_issue(&mut world.scene, "進む", Ok(natural(YES, IT_LITERACY)));
}

#[then(expr = "その課題は台帳の本体にある")]
async fn in_body(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(section_of(world), Section::Body, "the section");
}

#[given(expr = "課題 {string} の自然さの問いに、判断モデルが自然でないと答える")]
fn answers_unnatural(world: &mut InterviewWorld, expectation: String) {
    let answer = Ok(natural(NO, "しないこと"));
    one_issue(&mut world.scene, &expectation, answer);
    drop(expectation);
}

#[then(expr = "その課題は台帳の本体に無い")]
async fn not_in_body(world: &mut InterviewWorld) {
    world.run().await;
    assert_ne!(section_of(world), Section::Body, "the section");
}

#[then(expr = "その課題は台帳の最後の、ペルソナから外れると判断した課題の節にある")]
async fn in_last_section(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(section_of(world), Section::OffPersona, "the section");
    let number = world.scene.aliases.written("その課題");
    let (heading, issue) = (heading_at(world, OFF_PERSONA), issue_at(world, &number));
    assert!(heading < issue, "the issue is under the last heading");
}

#[given(expr = "自然さの問いへの判断モデルの応答が、形式の崩れた応答である")]
fn answers_malformed(world: &mut InterviewWorld) {
    one_issue(&mut world.scene, "進む", Err(JudgeError::Malformed));
}

#[then(expr = "その課題は、確かめられなかった区分にある")]
async fn issue_is_unchecked(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(section_of(world), Section::Unchecked, "the section");
}

#[given(expr = "判断モデルが、迷った理由として、ペルソナの ITの慣れ {string} を挙げる")]
fn names_it_literacy(world: &mut InterviewWorld, content: String) {
    let document = &world.scene.persona.it_literacy;
    assert_eq!(document, &content, "the persona's document");
    one_issue(&mut world.scene, "進む", Ok(natural(YES, IT_LITERACY)));
    drop(content);
}

/// The line of the ledger that tells the trait of the persona of the issue.
fn trait_line(world: &InterviewWorld) -> String {
    let first = issue_at(world, &world.scene.aliases.written("その課題"));
    let lines = ledger_lines(world);
    let after = lines.into_iter().skip(first);
    let found = after
        .into_iter()
        .find(|line| line.starts_with("  - ペルソナの性質: "));
    found.unwrap_or_default()
}

#[then(expr = "その課題に、ペルソナの性質 {string} とその中身が書かれている")]
async fn trait_written(world: &mut InterviewWorld, label: String) {
    world.run().await;
    let ledger = ledger_of(world);
    let entry = entry_of(&ledger, world.scene.aliases.number("その課題"));
    let named = entry.persona_trait.as_ref();
    let which = named.map(|persona_trait| persona_trait.which);
    assert_eq!(which, Some(Trait::ItLiteracy), "the trait");
    assert_eq!(Trait::ItLiteracy.label(), label, "its name");
    let literacy = world.scene.persona.it_literacy.clone();
    let content = named.map(|persona_trait| persona_trait.content.clone());
    assert_eq!(content, Some(literacy.clone()), "its content");
    let line = trait_line(world);
    assert!(line.contains(&label) && line.contains(&literacy), "{line}");
}
