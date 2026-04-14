use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;
use crate::constants::ENABLE_COLORS;

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {

    let border_style = if ENABLE_COLORS {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let header = if let Some(issue) = state.selected() {
        let assignee = issue
            .fields
            .assignee
            .as_ref()
            .and_then(|u| u.display_name.as_deref())
            .unwrap_or("Unassigned");

        format!("{} >>> {} [{}]", issue.key, issue.fields.summary, assignee)
    } else {
        "No issue selected".to_string()
    };

    let widget = Paragraph::new(header).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Current Ticket ")
            .border_style(border_style),
    );

    f.render_widget(widget, area);
}
