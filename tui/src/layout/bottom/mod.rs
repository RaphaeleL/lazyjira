use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::constants::ENABLE_COLORS;
use crate::state::{AppState, Focus, Pane};

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focused_pane == Some(Pane::Bottom);
    let is_active_pane = state.active_pane == Pane::Bottom;
    let show_pane = state.focused_pane.is_none() || is_focused;

    if !show_pane {
        return;
    }

    let draw_area = if is_focused { f.area() } else { area };

    let is_focus_jql = matches!(state.focus, Focus::Jql);

    let style = if ENABLE_COLORS {
        if is_focused {
            Style::default().fg(Color::Green)
        } else if is_focus_jql || is_active_pane {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        }
    } else {
        if is_focused || is_active_pane {
            Style::default().bold()
        } else {
            Style::default()
        }
    };

    let title_prefix = if is_focused {
        ">>> "
    } else if is_active_pane {
        ">> "
    } else {
        ""
    };

    let title = if is_focus_jql {
        format!("{}JQL (active)", title_prefix)
    } else {
        format!("{}JQL", title_prefix)
    };

    let text: String = state.jql.iter().collect();

    let mut display = text.clone();
    if is_focus_jql {
        let byte_pos = text
            .char_indices()
            .nth(state.jql_cursor)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        display.insert(byte_pos, '█');
    }

    let input = Paragraph::new(Line::from(Span::styled(display, style))).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(style),
    );

    f.render_widget(input, draw_area);
}
