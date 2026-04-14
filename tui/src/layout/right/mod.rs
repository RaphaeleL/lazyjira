use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;
use crate::ui::markdown::render_markdown;

pub fn draw(f: &mut Frame, area: Rect, state: &AppState) {
    let issue = match state.selected() {
        Some(i) => i,
        None => {
            let widget = Paragraph::new("No issue selected")
                .block(Block::default().borders(Borders::ALL).title("Issue"));
            f.render_widget(widget, area);
            return;
        }
    };

    let desc_raw = issue
        .fields
        .description
        .as_deref()
        .unwrap_or("No description");

    let desc = render_markdown(desc_raw);
    let viewport_height = area.height.saturating_sub(2) as usize;
    let content_height = desc.lines().count();
    let max_scroll = content_height.saturating_sub(viewport_height);
    let scroll = state.desc_scroll.min(max_scroll as u16);

    let desc_widget = Paragraph::new(desc)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", issue.key)),
        )
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));

    f.render_widget(desc_widget, area);
}
