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

    let mut state = AppState::new(issues, jira);

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
        })?;

        if let Event::Key(key) = event::read()? {
            match (&state.focus, key.code, key.modifiers) {

                // =========================
                // EXIT
                // =========================
                (_, KeyCode::Char('q'), _) => break,

                (_, KeyCode::Char('?'), _) => {
                    state.show_help = !state.show_help;
                }

                (_, KeyCode::Char('f'), _) => {
                    state.focus = match state.focus {
                        Focus::Issues => Focus::Description,
                        Focus::Description => Focus::Issues,
                        Focus::Jql => Focus::Issues,
                    };
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

                (Focus::Issues, KeyCode::Char('d'), _) => {
                    state.desc_scroll = state.desc_scroll.saturating_add(1);
                }

                (Focus::Issues, KeyCode::Char('u'), _) => {
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

                (Focus::Description, KeyCode::Char('d'), _) => {
                    state.desc_scroll = state.desc_scroll.saturating_add(1);
                }

                (Focus::Description, KeyCode::Char('u'), _) => {
                    state.desc_scroll = state.desc_scroll.saturating_sub(1);
                }

                // =========================
                // JQL INPUT
                // =========================
                (Focus::Jql, KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                    state.jql_cursor = 0;
                }

                (Focus::Jql, KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                    state.jql_cursor = state.jql.len();
                }

                (Focus::Jql, KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    state.jql.clear();
                    state.jql_cursor = 0;
                }

                (Focus::Jql, KeyCode::Char(c), _) => {
                    state.jql.insert(state.jql_cursor, c);
                    state.jql_cursor += 1;
                }

                (Focus::Jql, KeyCode::Backspace, _) => {
                    if state.jql_cursor > 0 {
                        state.jql_cursor -= 1;
                        state.jql.remove(state.jql_cursor);
                    }
                }

                (Focus::Jql, KeyCode::Delete, _) => {
                    if state.jql_cursor < state.jql.len() {
                        state.jql.remove(state.jql_cursor);
                    }
                }

                (Focus::Jql, KeyCode::Left, _) => {
                    state.jql_cursor = state.jql_cursor.saturating_sub(1);
                }

                (Focus::Jql, KeyCode::Right, _) => {
                    state.jql_cursor = (state.jql_cursor + 1).min(state.jql.len());
                }

                (Focus::Jql, KeyCode::Home, _) => {
                    state.jql_cursor = 0;
                }

                (Focus::Jql, KeyCode::End, _) => {
                    state.jql_cursor = state.jql.len();
                }

                // =========================
                // EXECUTE JQL
                // =========================
                (Focus::Jql, KeyCode::Enter, _) => {
                    reload_issues(&mut state);
                    state.focus = Focus::Issues;
                    state.editing_jql = false;
                }

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
