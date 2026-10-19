//! The persona's lines that lead every question of a step.

use crate::config::persona::Persona;

/// The lines that lead every question of a step: who the judge plays,
/// what the person aims at, and that the judge answers as that person.
#[must_use]
pub fn preamble(persona: &Persona, goal: &str) -> String {
    let head = [
        "あなたは次の人として、この画面を使っている。".to_owned(),
        format!("名前: {}", persona.name.trim()),
        format!("どんな人か: {}", persona.profile.trim()),
        format!("ITの慣れ: {}", persona.it_literacy.trim()),
        format!("達成したいこと: {}", persona.wants.trim()),
        "しないこと:".to_owned(),
    ];
    let avoids = persona
        .avoids
        .iter()
        .map(|avoid| format!("- {}", avoid.trim()));
    let tail = [
        format!("目指すこと: {}", goal.trim()),
        "この人ならどうするか、どう感じるかで答える。".to_owned(),
    ];
    let lines: Vec<String> = head.into_iter().chain(avoids).chain(tail).collect();
    lines.join("\n")
}
