//! What the core noted while taking the facts of an effect.

use webqa_core::event::Event;
use webqa_core::machine::{Note, Step};

use super::detect::note_issue;
use super::{Ports, RunError};

/// Logs what the core noted since `before`, the count of its notes and
/// its stage then: each new note, then each stage it moved on to.
pub(super) async fn after(
    before: (usize, usize),
    next: &Step,
    ports: &mut Ports<'_>,
) -> Result<(), RunError> {
    let state = next.state();
    for note in state.notes().iter().skip(before.0) {
        noted(note.clone(), ports).await?;
    }
    let stages = state.rules().stages.iter().enumerate();
    let reached = stages
        .take(state.stage().saturating_add(1))
        .skip(before.1 + 1);
    for (stage, named) in reached {
        let name = named.name.clone();
        ports.log(Event::StageAdvanced { stage, name }).await?;
    }
    Ok(())
}

async fn noted(note: Note, ports: &mut Ports<'_>) -> Result<(), RunError> {
    match note {
        Note::Rejected(why) => ports.log(Event::Rejected { why }).await,
        Note::Anomaly(anomaly) => ports.log(Event::anomaly(&anomaly)).await,
        Note::Issue(issue) => note_issue(ports, issue).await,
    }
}
