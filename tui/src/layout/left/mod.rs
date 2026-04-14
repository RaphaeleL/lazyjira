use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, ListItem, List},
    Frame,
};

use crate::constants::ENABLE_COLORS;
use crate::state::AppState;

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if ENABLE_COLORS {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let height = area.height as usize;
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

    let visible_items = state
        .issues
        .iter()
        .skip(list_offset)
        .take(viewport);

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
                prefix,
                issue.key,
                issue.fields.status.name,
                issue.fields.summary
            ))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Issues ")
            .border_style(border_style),
    );

    f.render_widget(list, area);
}
