use crate::model::issue::Issue;
use crate::jira::JiraClient;

pub struct AppState {
    pub issues: Vec<Issue>,
    pub selected: usize,
    pub desc_scroll: u16,
    pub list_offset: usize,
    pub jql: String,
    pub jql_cursor: usize,
    pub editing_jql: bool,
    pub jira: JiraClient,
    pub focus: Focus,
    pub show_help: bool,
}

pub enum Focus {
    Issues,
    Jql,
}

impl AppState {
    pub fn new(issues: Vec<Issue>, jira: JiraClient) -> Self {
        Self {
            issues,
            selected: 0,
            list_offset: 0,
            desc_scroll: 0,
            jql: "assignee = currentUser() ORDER BY updated DESC".to_string(),
            jql_cursor: 0,
            editing_jql: false,
            jira,
            focus: Focus::Issues,
            show_help: false,
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
