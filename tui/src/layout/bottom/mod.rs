use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color},
    Frame,
    text::{Span, Line},
};

use crate::state::{AppState, Focus};

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let is_active = matches!(state.focus, Focus::Jql);

    let style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let title = if is_active {
        " JQL (active) "
    } else {
        " JQL "
    };

    let mut text = state.jql.clone();

    // show cursor
    if is_active {
        text.push('█');
    }

    let input = Paragraph::new(Line::from(Span::styled(text, style)))
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(input, area);
}
