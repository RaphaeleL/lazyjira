use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::jira::Transition;

pub fn draw_transition_modal(f: &mut Frame, area: Rect, transitions: &[Transition], selected: usize) {
    let popup_area = centered_rect(50, 40, area);
    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(popup_area);

    // Title
    let title_widget = Paragraph::new("Change Status")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(title_widget, chunks[0]);

    // Transition list
    let items: Vec<ListItem> = transitions
        .iter()
        .enumerate()
        .map(|(idx, t)| {
            let style = if idx == selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(t.name.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Status "));

    f.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::raw("j/k: Up/Down | Enter: Confirm | Esc: Cancel"),
    ]));
    f.render_widget(instructions, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
