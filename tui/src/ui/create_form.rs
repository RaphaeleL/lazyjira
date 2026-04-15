use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::FormField;

pub fn draw_create_form(f: &mut Frame, area: Rect, form: &crate::state::CreateForm) {
    let popup_area = centered_rect(70, 60, area);
    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(popup_area);

    let block = Block::default().title("Create Issue").borders(Borders::ALL);
    let title_widget = Paragraph::new("Create New Issue")
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(title_widget, chunks[0]);

    // Summary field
    draw_form_field(
        f,
        chunks[1],
        "Summary",
        &form.summary,
        form.focused_field == FormField::Summary,
    );

    // Description field
    draw_form_field(
        f,
        chunks[2],
        "Description",
        &form.description,
        form.focused_field == FormField::Description,
    );

    // Project field
    draw_form_field(
        f,
        chunks[3],
        "Project",
        &form.project,
        form.focused_field == FormField::Project,
    );

    // Component field
    draw_form_field(
        f,
        chunks[4],
        "Component",
        &form.component,
        form.focused_field == FormField::Component,
    );

    // Issue Type field
    draw_form_field(
        f,
        chunks[5],
        "Type",
        &form.issue_type,
        form.focused_field == FormField::IssueType,
    );

    // Instructions
    let instructions = Paragraph::new(Line::from(vec![Span::raw(
        "Tab: Next | Shift+Tab: Prev | Enter: Submit | Esc: Cancel",
    )]));
    f.render_widget(instructions, chunks[6]);
}

fn draw_form_field(f: &mut Frame, area: Rect, label: &str, value: &str, is_focused: bool) {
    let style = if is_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let display = if is_focused {
        format!("{} █", value)
    } else {
        value.to_string()
    };

    let widget = Paragraph::new(display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", label))
                .border_style(border_style),
        )
        .style(style);

    f.render_widget(widget, area);
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
