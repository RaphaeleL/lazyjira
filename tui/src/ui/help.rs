use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = r#"
Keybindings:

Global:
  q          - Quit application
  ?          - Toggle this help
  f          - Toggle full screen description view

Issue Navigation (when focused on issues):
  j          - Next issue
  k          - Previous issue
  d          - Scroll description down
  u          - Scroll description up
  o          - Open issue in browser
  Ctrl+o     - Copy issue key to clipboard

JQL Search:
  @          - Focus JQL input (append mode)
  #          - Focus JQL input (clear mode)
  Esc        - Return to issues
  Enter      - Execute search

Description Focus (full screen):
  j/k        - Scroll description up/down
  d/u        - Scroll description up/down

Press Esc to close this help.
"#;

    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(Text::raw(help_text))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    let popup_area = centered_rect(60, 80, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
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
