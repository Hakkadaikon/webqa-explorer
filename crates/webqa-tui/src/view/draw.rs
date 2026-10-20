//! Draws the run screen: a header, the progress, the latest operations and
//! the keys.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, Paragraph};
use webqa_core::machine::Rejection;

use super::{Outcome, Recent, State};
use crate::style::{framed, hints, labelled};

/// Below this width the two panes stack instead of standing side by side.
const NARROW: u16 = 70;

/// Draws `state` over the whole frame.
pub fn render(state: &State, frame: &mut Frame) {
    let [head, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    frame.render_widget(Paragraph::new(header(state)), head);
    let [left, right] = panes(body);
    let progress = Paragraph::new(progress(state)).block(framed("進み具合"));
    frame.render_widget(progress, left);
    let latest = List::new(state.recent.iter().rev().map(recent_line));
    frame.render_widget(latest.block(framed("直近の操作")), right);
    frame.render_widget(Paragraph::new(hints(&[("Ctrl-C", "閉じる")])), keys);
}

fn panes(body: Rect) -> [Rect; 2] {
    let layout = if body.width < NARROW {
        Layout::vertical([Constraint::Length(13), Constraint::Min(3)])
    } else {
        Layout::horizontal([Constraint::Length(34), Constraint::Min(20)])
    };
    layout.areas(body)
}

fn header(state: &State) -> Line<'static> {
    let mode = state.mode.map_or("-", |mode| mode.as_str());
    let status = state.finished.as_ref().map_or_else(
        || Span::styled(" 実行中 ", Style::new().black().on_yellow()),
        |reason| {
            Span::styled(
                format!(" 終わった {reason} "),
                Style::new().black().on_green(),
            )
        },
    );
    Line::from(vec![
        Span::styled(" webqa ", Style::new().black().on_cyan().bold()),
        Span::raw(format!(" {mode} ")),
        status,
        Span::raw(format!(" 経過 {}", clock(state.elapsed_ms))),
    ])
}

/// `mm:ss` of `ms`.
fn clock(ms: u64) -> String {
    let seconds = ms / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn progress(state: &State) -> Vec<Line<'static>> {
    let counts = &state.counts;
    let dash = |value: Option<String>| value.unwrap_or_else(|| "-".to_owned());
    let trial = state
        .trial
        .map(|trial| format!("{trial}(終わった{})", state.trials_ended));
    [
        format!("試行 {}", dash(trial)),
        format!("段階 {}", dash(state.stage.clone())),
        format!("ステップ {}", dash(state.step.map(|step| step.to_string()))),
        format!("判断 {}", counts.decided),
        format!("画面が変わった {}", counts.changed),
        format!("止められた {}", counts.rejected),
        format!("応答の異常 {}", counts.anomalies),
        format!("回復 {}", counts.recovered),
        format!("課題 {}", counts.issues),
        format!("通信 {}", counts.accesses),
        format!("トークン {}/{}", counts.input_tokens, counts.output_tokens),
    ]
    .iter()
    .map(|text| labelled(text))
    .collect()
}

fn recent_line(recent: &Recent) -> Line<'static> {
    let step = recent
        .step
        .map_or_else(String::new, |step| format!("#{step} "));
    let (said, color) = outcome(recent.outcome);
    Line::from(vec![
        Span::styled(step, Style::new().dim()),
        Span::raw(format!("{} ", recent.act)),
        Span::styled(said, Style::new().fg(color)),
    ])
}

/// What is said of an outcome, and its color.
fn outcome(outcome: Outcome) -> (String, Color) {
    match outcome {
        Outcome::Pending => ("…".to_owned(), Color::Gray),
        Outcome::Changed => ("変わった".to_owned(), Color::Green),
        Outcome::Unchanged => ("変わらない".to_owned(), Color::DarkGray),
        Outcome::Rejected(why) => (format!("止めた({})", rejection(why)), Color::Red),
    }
}

/// The reasons of stopping, as said.
const REJECTIONS: [(Rejection, &str); 4] = [
    (Rejection::Forbidden, "禁止"),
    (Rejection::LowConfidence, "確信が低い"),
    (Rejection::Missing, "要素が無い"),
    (Rejection::Replaced, "別の要素"),
];

fn rejection(why: Rejection) -> &'static str {
    let found = REJECTIONS.iter().find(|(of, _)| *of == why);
    found.map_or("-", |(_, said)| *said)
}

#[cfg(test)]
mod tests;
