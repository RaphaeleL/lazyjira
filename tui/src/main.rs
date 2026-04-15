mod layout;
mod state;
mod model;
mod jira;
mod cache;
mod data;
mod config;
mod constants;
mod ui;

use config::Config;
use jira::JiraClient;
use state::{AppState, Focus};
use data::issues::load_issues;

use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::process;

pub fn reload_issues(state: &mut AppState) {
    let jql_str: String = state.jql.iter().collect();
    let issues = state.jira.search_issues(&jql_str);

    state.issues = issues;
    state.selected = 0;
    state.list_offset = 0;
    state.desc_scroll = 0;
}

fn main() -> io::Result<()> {
    let config = Config::load().expect("Failed to load config. Please run 'jira init' first.");
    let jira = JiraClient::new(config.clone());

    let cache_path = Config::get_cache_dir().join("issues.json");
    let cache_path_str = cache_path.to_str().unwrap();
    let issues = load_issues(&jira, cache_path_str);

    let mut state = AppState::new(issues, jira, config.default_project.clone(), config.issue_type.clone());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let (top_height, bottom_height) = match state.focus {
                Focus::Description => (0, 0),
                _ => (3, 3),
            };

            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(top_height),
                    ratatui::layout::Constraint::Min(0),
                    ratatui::layout::Constraint::Length(bottom_height),
                ])
                .split(f.area());

            let (left_percent, right_percent) = match state.focus {
                Focus::Description => (0, 100),
                _ => (30, 70),
            };

            let middle = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(left_percent),
                    ratatui::layout::Constraint::Percentage(right_percent),
                ])
                .split(chunks[1]);

            crate::layout::top::draw(f, chunks[0], &state);
            crate::layout::left::draw(f, middle[0], &state);
            crate::layout::right::draw(f, middle[1], &state);
            crate::layout::bottom::draw(f, chunks[2], &state);

            if state.show_help {
                crate::ui::help::draw_help(f, f.area());
            }

            if state.show_create_form {
                crate::ui::create_form::draw_create_form(f, f.area(), &state.create_form);
            }

            if state.show_transition_modal {
                crate::ui::transition_modal::draw_transition_modal(f, f.area(), &state.transitions, state.transition_selected);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            // Handle JQL input first
            if state.focus == Focus::Jql && state.editing_jql {
                match key.code {
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.jql_cursor = 0;
                        continue;
                    }
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.jql_cursor = state.jql.len();
                        continue;
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.jql.clear();
                        state.jql_cursor = 0;
                        continue;
                    }
                    KeyCode::Char(c) => {
                        state.jql.insert(state.jql_cursor, c);
                        state.jql_cursor += 1;
                        continue;
                    }
                    KeyCode::Backspace => {
                        if state.jql_cursor > 0 {
                            state.jql_cursor -= 1;
                            state.jql.remove(state.jql_cursor);
                        }
                        continue;
                    }
                    KeyCode::Delete => {
                        if state.jql_cursor < state.jql.len() {
                            state.jql.remove(state.jql_cursor);
                        }
                        continue;
                    }
                    KeyCode::Left => {
                        state.jql_cursor = state.jql_cursor.saturating_sub(1);
                        continue;
                    }
                    KeyCode::Right => {
                        state.jql_cursor = (state.jql_cursor + 1).min(state.jql.len());
                        continue;
                    }
                    KeyCode::Home => {
                        state.jql_cursor = 0;
                        continue;
                    }
                    KeyCode::End => {
                        state.jql_cursor = state.jql.len();
                        continue;
                    }
                    KeyCode::Enter => {
                        reload_issues(&mut state);
                        state.focus = Focus::Issues;
                        state.editing_jql = false;
                        continue;
                    }
                    KeyCode::Esc => {
                        state.focus = Focus::Issues;
                        state.editing_jql = false;
                        continue;
                    }
                    _ => continue,
                }
            }

            // Handle create form input first
            if state.show_create_form {
                match key.code {
                    KeyCode::Tab => {
                        state.create_form.next_field();
                        continue;
                    }
                    KeyCode::BackTab => {
                        state.create_form.prev_field();
                        continue;
                    }
                    KeyCode::Char(c) => {
                        state.create_form.get_current_field_mut().push(c);
                        continue;
                    }
                    KeyCode::Backspace => {
                        state.create_form.get_current_field_mut().pop();
                        continue;
                    }
                    KeyCode::Enter => {
                        if !state.create_form.summary.is_empty() {
                            let _ = state.jira.create_issue(
                                &state.create_form.summary,
                                Some(state.create_form.project.clone()),
                                if state.create_form.component.is_empty() { None } else { Some(state.create_form.component.clone()) },
                                Some(state.create_form.description.clone()),
                                Some(state.create_form.issue_type.clone()),
                            );
                            state.show_create_form = false;
                            reload_issues(&mut state);
                        }
                        continue;
                    }
                    KeyCode::Esc => {
                        state.show_create_form = false;
                        continue;
                    }
                    _ => continue,
                }
            }

            // Handle transition modal input
            if state.show_transition_modal {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        state.transition_selected = (state.transition_selected + 1) % state.transitions.len();
                        continue;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if state.transition_selected > 0 {
                            state.transition_selected -= 1;
                        } else {
                            state.transition_selected = state.transitions.len() - 1;
                        }
                        continue;
                    }
                    KeyCode::Enter => {
                        if let Some(issue) = state.selected() {
                            if let Some(transition) = state.transitions.get(state.transition_selected) {
                                let _ = state.jira.transition_issue(&issue.key, &transition.id);
                                state.show_transition_modal = false;
                                reload_issues(&mut state);
                            }
                        }
                        continue;
                    }
                    KeyCode::Esc => {
                        state.show_transition_modal = false;
                        continue;
                    }
                    _ => continue,
                }
            }

            match (&state.focus, key.code, key.modifiers) {

                // =========================
                // EXIT
                // =========================
                (_, KeyCode::Char('q'), _) => break,

                (_, KeyCode::Char('?'), _) => {
                    state.show_help = !state.show_help;
                }

                (_, KeyCode::Char('n'), _) => {
                    state.show_create_form = true;
                }

                (_, KeyCode::Char('f'), _) => {
                    state.focus = match state.focus {
                        Focus::Issues => Focus::Description,
                        Focus::Description => Focus::Issues,
                        Focus::Jql => Focus::Issues,
                    };
                }

                (Focus::Issues, KeyCode::Char('u'), modifiers) if !modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(issue) = state.selected() {
                        match state.jira.get_transitions(&issue.key) {
                            Ok(transitions) => {
                                if !transitions.is_empty() {
                                    state.transitions = transitions;
                                    state.transition_selected = 0;
                                    state.show_transition_modal = true;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }

                // =========================
                // FOCUS SWITCH
                // =========================
                (_, KeyCode::Char('@'), _) => {
                    state.focus = Focus::Jql;
                    state.editing_jql = true;
                }
                (_, KeyCode::Char('#'), _) => {
                    state.focus = Focus::Jql;
                    state.editing_jql = true;
                    state.jql = vec![];
                    state.jql_cursor = 0;
                }

                (_, KeyCode::Esc, _) => {
                    if state.show_help {
                        state.show_help = false;
                    } else {
                        state.focus = Focus::Issues;
                        state.editing_jql = false;
                    }
                }

                // =========================
                // ISSUE NAVIGATION
                // =========================
                (Focus::Issues, KeyCode::Char('j'), _) => state.next(),
                (Focus::Issues, KeyCode::Char('k'), _) => state.prev(),

                (Focus::Issues, KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    state.desc_scroll = state.desc_scroll.saturating_add(1);
                }

                (Focus::Issues, KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    state.desc_scroll = state.desc_scroll.saturating_sub(1);
                }

                (Focus::Issues, KeyCode::Char('o'), modifiers) => {
                    if let Some(issue) = state.issues.get(state.selected) {
                        if modifiers.contains(KeyModifiers::CONTROL) {
                            let _ = process::Command::new("sh").arg("-c").arg(format!("echo '{}' | pbcopy", issue.key)).spawn();
                        } else {
                            let url = format!("{}/browse/{}", state.jira.jira_url(), issue.key);
                            let _ = process::Command::new("open").arg(&url).spawn();
                        }
                    }
                }

                // =========================
                // DESCRIPTION FOCUS
                // =========================
                (Focus::Description, KeyCode::Char('j'), _) => {
                    state.desc_scroll = state.desc_scroll.saturating_add(1);
                }

                (Focus::Description, KeyCode::Char('k'), _) => {
                    state.desc_scroll = state.desc_scroll.saturating_sub(1);
                }

                (Focus::Description, KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    state.desc_scroll = state.desc_scroll.saturating_add(1);
                }

                (Focus::Description, KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    state.desc_scroll = state.desc_scroll.saturating_sub(1);
                }

                // =========================
                // JQL INPUT (handled above in dedicated block)
                // =========================

                // =========================
                // EXECUTE JQL (handled above in dedicated block)
                // =========================

                _ => {}            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
