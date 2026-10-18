//! What the screen shows, folded from the event log one line at a time.

use ratatui::{
    Frame,
    widgets::{Block, Paragraph},
};
use webqa_core::event::{Event, Line, Mode};

/// The last operation the judge chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The operation.
    pub op: String,
    /// The target element, if the operation needs one.
    pub target: Option<String>,
}

/// The run as seen through the lines read so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    /// The kind of run, once the start line is read.
    pub mode: Option<Mode>,
    /// The current step.
    pub step: Option<u64>,
    /// The last decision.
    pub decision: Option<Decision>,
    /// The number of steps that changed the screen.
    pub changed_steps: u64,
    /// Why the run ended, once it has.
    pub finished: Option<String>,
}

impl State {
    /// Applies one line of the log.
    #[must_use]
    pub fn fold(self, line: &Line) -> Self {
        let event = &line.event;
        Self {
            mode: mode(event).or(self.mode),
            step: line.step.or(self.step),
            decision: decision(event).or(self.decision),
            changed_steps: self.changed_steps + changed(event),
            finished: reason(event).or(self.finished),
        }
    }
}

const fn mode(event: &Event) -> Option<Mode> {
    if let Event::RunStarted { mode, .. } = event {
        Some(*mode)
    } else {
        None
    }
}

fn decision(event: &Event) -> Option<Decision> {
    if let Event::Decided { op, target, .. } = event {
        Some(Decision {
            op: op.clone(),
            target: target.clone(),
        })
    } else {
        None
    }
}

const fn changed(event: &Event) -> u64 {
    if let Event::Executed { changed: true, .. } = event {
        1
    } else {
        0
    }
}

fn reason(event: &Event) -> Option<String> {
    if let Event::RunFinished { reason } = event {
        Some(reason.clone())
    } else {
        None
    }
}

/// Draws `state` over the whole frame.
pub fn render(state: &State, frame: &mut Frame) {
    let text = rows(state)
        .map(|(name, value)| format!("{name:<9}{}", value.as_deref().unwrap_or("-")))
        .join("\n");
    let view = Paragraph::new(text).block(Block::bordered().title("webqa"));
    frame.render_widget(view, frame.area());
}

fn rows(state: &State) -> [(&'static str, Option<String>); 5] {
    [
        ("mode", state.mode.map(mode_name)),
        ("step", state.step.map(|step| step.to_string())),
        ("decided", state.decision.as_ref().map(decision_text)),
        ("changed", Some(state.changed_steps.to_string())),
        ("finished", state.finished.clone()),
    ]
}

fn mode_name(mode: Mode) -> String {
    // The name comes from the serde derive, as a JSON string.
    serde_json::to_string(&mode)
        .unwrap_or_default()
        .trim_matches('"')
        .to_owned()
}

fn decision_text(decision: &Decision) -> String {
    format!(
        "{} {}",
        decision.op,
        decision.target.as_deref().unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use webqa_core::event::parse_line;

    use super::*;
    use crate::EXAMPLE;

    #[test]
    fn fold_tracks_step_and_finish() {
        let state = EXAMPLE
            .iter()
            .map(|text| parse_line(text).unwrap())
            .fold(State::default(), |state, line| state.fold(&line));
        assert_eq!(
            state,
            State {
                mode: Some(Mode::Explore),
                step: Some(1),
                decision: Some(Decision {
                    op: "click".to_owned(),
                    target: Some("保存".to_owned()),
                }),
                changed_steps: 1,
                finished: Some("goal_achieved".to_owned()),
            },
            "every line updates its part of the state"
        );
    }
}
