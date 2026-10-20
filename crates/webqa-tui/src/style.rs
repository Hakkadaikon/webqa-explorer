//! The looks both screens share: the frame, the key hints, the labels.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;

/// The color of frames and labels.
pub const ACCENT: Color = Color::Cyan;

/// A framed block titled `title`.
#[must_use]
pub fn framed(title: &str) -> Block<'static> {
    Block::bordered()
        .border_style(Style::new().fg(ACCENT))
        .title(Span::styled(format!(" {title} "), Style::new().bold()))
}

/// One line of key hints: each key, then what it does.
#[must_use]
pub fn hints(keys: &[(&str, &str)]) -> Line<'static> {
    let hint = |(key, does): &(&str, &str)| {
        [
            Span::styled(format!(" {key} "), Style::new().black().on_cyan()),
            Span::styled(
                format!(" {does}  "),
                Style::new().add_modifier(Modifier::DIM),
            ),
        ]
    };
    Line::from(keys.iter().flat_map(hint).collect::<Vec<_>>())
}

/// A `label value` text with the label in the accent color; the text
/// from its first space on is the value.
#[must_use]
pub fn labelled(text: &str) -> Line<'static> {
    let (label, value) = text.split_once(' ').unwrap_or((text, ""));
    Line::from(vec![
        Span::styled(format!("{label} "), Style::new().fg(ACCENT).bold()),
        Span::raw(value.to_owned()),
    ])
}

#[cfg(test)]
mod tests;
