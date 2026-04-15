use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::constants::ENABLE_COLORS;
use crate::state::{AppState, Pane};

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focused_pane == Some(Pane::Left);
    let is_active = state.active_pane == Pane::Left;
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

    let height = draw_area.height as usize;
    let viewport = height.saturating_sub(2);
    let max_offset = state.issues.len().saturating_sub(viewport);
    let list_offset = state.list_offset.min(max_offset);
    let selected = state.selected;

    let list_offset = if selected < list_offset {
        selected
    } else if selected >= list_offset + viewport {
        selected - viewport + 1
    } else {
        list_offset
    };

    let list_offset = list_offset.min(max_offset);

    let visible_items = state.issues.iter().skip(list_offset).take(viewport);

    let items: Vec<ListItem> = visible_items
        .enumerate()
        .map(|(i, issue)| {
            let global_index = list_offset + i;

            let prefix = if global_index == state.selected {
                "> "
            } else {
                "  "
            };

            ListItem::new(format!(
                "{}{} [{}] {}",
                prefix, issue.key, issue.fields.status.name, issue.fields.summary
            ))
        })
        .collect();

    let title = format!("{}Issues ", title_prefix);

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style),
    );

    f.render_widget(list, draw_area);
}
