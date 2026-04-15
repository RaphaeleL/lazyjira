use crate::jira::JiraClient;
use crate::model::issue::Issue;

pub struct AppState {
    pub issues: Vec<Issue>,
    pub selected: usize,
    pub desc_scroll: u16,
    pub list_offset: usize,
    pub jql: Vec<char>,
    pub jql_cursor: usize,
    pub editing_jql: bool,
    pub jira: JiraClient,
    pub focus: Focus,
    pub active_pane: Pane,
    pub focused_pane: Option<Pane>,
    pub show_help: bool,
    pub help_scroll: u16,
    pub show_create_form: bool,
    pub create_form: CreateForm,
    pub show_transition_modal: bool,
    pub transitions: Vec<crate::jira::Transition>,
    pub transition_selected: usize,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Pane {
    Top,
    Left,
    Right,
    Bottom,
}

impl Pane {
    pub fn next(self) -> Self {
        match self {
            Pane::Top => Pane::Left,
            Pane::Left => Pane::Right,
            Pane::Right => Pane::Bottom,
            Pane::Bottom => Pane::Top,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Pane::Top => Pane::Bottom,
            Pane::Left => Pane::Top,
            Pane::Right => Pane::Left,
            Pane::Bottom => Pane::Right,
        }
    }
}

#[derive(PartialEq)]
pub enum Focus {
    Issues,
    Jql,
    Description,
}

impl AppState {
    pub fn new(
        issues: Vec<Issue>,
        jira: JiraClient,
        default_project: String,
        default_issue_type: String,
    ) -> Self {
        Self {
            issues,
            selected: 0,
            list_offset: 0,
            desc_scroll: 0,
            jql: "assignee = currentUser() ORDER BY updated DESC"
                .chars()
                .collect(),
            jql_cursor: "assignee = currentUser() ORDER BY updated DESC".len(),
            editing_jql: false,
            jira,
            focus: Focus::Issues,
            active_pane: Pane::Left,
            focused_pane: None,
            show_help: false,
            help_scroll: 0,
            show_create_form: false,
            create_form: CreateForm::new(default_project, default_issue_type),
            show_transition_modal: false,
            transitions: vec![],
            transition_selected: 0,
        }
    }

    pub fn jql_insert(&mut self, c: char) {
        self.jql.insert(self.jql_cursor, c);
        self.jql_cursor += 1;
    }

    pub fn jql_backspace(&mut self) {
        if self.jql_cursor > 0 {
            self.jql_cursor -= 1;
            self.jql.remove(self.jql_cursor);
        }
    }

    pub fn next(&mut self) {
        if self.selected + 1 < self.issues.len() {
            self.selected += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn ensure_visible(&mut self, viewport: usize) {
        if self.issues.is_empty() {
            return;
        }

        if self.selected < self.list_offset {
            self.list_offset = self.selected;
        }

        if self.selected >= self.list_offset + viewport {
            self.list_offset = self.selected - viewport + 1;
        }

        let max_offset = self.issues.len().saturating_sub(viewport);
        if self.list_offset > max_offset {
            self.list_offset = max_offset;
        }
    }

    pub fn update_list_scroll(&mut self, viewport: usize) {
        if self.issues.is_empty() {
            return;
        }

        if self.selected < self.list_offset {
            self.list_offset = self.selected;
        }

        if self.selected >= self.list_offset + viewport {
            self.list_offset = self.selected - viewport + 1;
        }

        let max_offset = self.issues.len().saturating_sub(viewport);
        self.list_offset = self.list_offset.min(max_offset);
    }

    pub fn selected(&self) -> Option<&Issue> {
        self.issues.get(self.selected)
    }

    pub fn assignee(&self) -> Option<&str> {
        self.selected()
            .and_then(|i| i.fields.assignee.as_ref())
            .and_then(|u| u.display_name.as_deref())
    }

    pub fn reporter(&self) -> Option<&str> {
        self.selected()
            .and_then(|i| i.fields.reporter.as_ref())
            .and_then(|u| u.display_name.as_deref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormField {
    Summary,
    Description,
    Project,
    Component,
    IssueType,
}

pub struct CreateForm {
    pub summary: String,
    pub description: String,
    pub project: String,
    pub component: String,
    pub issue_type: String,
    pub focused_field: FormField,
}

impl CreateForm {
    pub fn new(default_project: String, default_issue_type: String) -> Self {
        Self {
            summary: String::new(),
            description: String::new(),
            project: default_project,
            component: String::new(),
            issue_type: default_issue_type,
            focused_field: FormField::Summary,
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            FormField::Summary => FormField::Description,
            FormField::Description => FormField::Project,
            FormField::Project => FormField::Component,
            FormField::Component => FormField::IssueType,
            FormField::IssueType => FormField::Summary,
        };
    }

    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            FormField::Summary => FormField::IssueType,
            FormField::Description => FormField::Summary,
            FormField::Project => FormField::Description,
            FormField::Component => FormField::Project,
            FormField::IssueType => FormField::Component,
        };
    }

    pub fn get_current_field_mut(&mut self) -> &mut String {
        match self.focused_field {
            FormField::Summary => &mut self.summary,
            FormField::Description => &mut self.description,
            FormField::Project => &mut self.project,
            FormField::Component => &mut self.component,
            FormField::IssueType => &mut self.issue_type,
        }
    }
}
