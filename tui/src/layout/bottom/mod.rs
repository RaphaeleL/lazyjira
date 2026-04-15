use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::{AppState, Focus};

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let is_active = matches!(state.focus, Focus::Jql);

    let style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let title = if is_active { " JQL (active) " } else { " JQL " };

    let text: String = state.jql.iter().collect();

    let mut display = text.clone();
    if is_active {
        let byte_pos = text
            .char_indices()
            .nth(state.jql_cursor)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        display.insert(byte_pos, '█');
    }

    let input = Paragraph::new(Line::from(Span::styled(display, style)))
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(input, area);
}
