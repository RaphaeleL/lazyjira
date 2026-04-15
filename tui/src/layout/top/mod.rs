use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::constants::ENABLE_COLORS;
use crate::state::{AppState, Pane};

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focused_pane == Some(Pane::Top);
    let is_active = state.active_pane == Pane::Top;
    let show_pane = state.focused_pane.is_none() || is_focused;

    if !show_pane {
        return;
    }

    let draw_area = if is_focused { f.area() } else { area };

    let border_style = if ENABLE_COLORS {
        if is_focused {
            Style::default().fg(Color::Green)
        } else if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        }
    } else {
        if is_focused || is_active {
            Style::default().bold()
        } else {
            Style::default()
        }
    };

    let title_prefix = if is_focused {
        ">>> "
    } else if is_active {
        ">> "
    } else {
        ""
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

    let title = format!("{} Current Ticket ", title_prefix);

    let widget = Paragraph::new(header).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style),
    );

    f.render_widget(widget, draw_area);
}
