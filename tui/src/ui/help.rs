use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::constants::ENABLE_COLORS;
use crate::state::AppState;

pub fn draw_help(f: &mut Frame, area: Rect, state: &AppState) {
    let style = if ENABLE_COLORS {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let help_text = r#"Global:
  q          - Quit application
  ?          - Toggle this help
  f          - Focus/unfocus current pane fullscreen
  c          - Create new issue
  Tab        - Cycle forward through panes
  Shift+Tab  - Cycle backward through panes

Pane Navigation (based on active pane):
  Ctrl+n     - Next (Left pane: next issue, Right/Top: scroll down)
  Ctrl+p     - Previous (Left pane: prev issue, Right/Top: scroll up)

Issue Actions:
  u          - Update issue status (show transitions)
  o          - Open issue in browser
  Ctrl+o     - Copy issue key to clipboard

JQL Search:
  @          - Focus JQL input (append mode)
  #          - Focus JQL input (clear mode)
  Esc        - Return to issues
  Enter      - Execute search
  Arrow keys - Move cursor
  Home/End   - Beginning/end of line
  Ctrl+a/e   - Beginning/end of line
  Ctrl+u     - Clear line
  Delete     - Delete character

Transition Modal:
  Ctrl+n/p   - Select next/previous status
  Enter      - Apply transition
  Esc        - Cancel

Panes:
  Top     - Current ticket info
  Left   - Issue list
  Right  - Description
  Bottom - JQL input

Press Esc to close this help.
"#;

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(style);

    let paragraph = Paragraph::new(Text::raw(help_text))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((state.help_scroll, 0));

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
