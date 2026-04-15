use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::constants::ENABLE_COLORS;
use crate::state::{AppState, Pane};
use crate::ui::markdown::render_markdown;

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focused_pane == Some(Pane::Right);
    let is_active = state.active_pane == Pane::Right;
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

    let issue = match state.selected() {
        Some(i) => i,
        None => {
            let widget = Paragraph::new("No issue selected").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{}Issue", title_prefix))
                    .border_style(border_style),
            );
            f.render_widget(widget, draw_area);
            return;
        }
    };

    let desc_raw = issue
        .fields
        .description
        .as_deref()
        .unwrap_or("No description");

    let desc = render_markdown(desc_raw);
    let viewport_height = draw_area.height.saturating_sub(2) as usize;
    let content_height = desc.lines().count();
    let max_scroll = content_height.saturating_sub(viewport_height);
    let scroll = state.desc_scroll.min(max_scroll as u16);

    let desc_widget = Paragraph::new(desc)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} {} ", title_prefix, issue.key))
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));

    f.render_widget(desc_widget, draw_area);
}
