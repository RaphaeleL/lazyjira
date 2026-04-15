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
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::process;

pub fn reload_issues(state: &mut AppState) {
    let issues = state.jira.search_issues(&state.jql);

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
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Min(0),
                    ratatui::layout::Constraint::Length(3),
                ])
                .split(f.area());

            let middle = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(30),
                    ratatui::layout::Constraint::Percentage(70),
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
                    state.jql = "".to_string();
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

                (Focus::Issues, KeyCode::Char('o'), _) => {
                    if let Some(issue) = state.issues.get(state.selected) {
                        let url = format!("{}/browse/{}", state.jira.jira_url(), issue.key);
                        let _ = process::Command::new("open").arg(&url).spawn();
                    }
                }

                // =========================
                // JQL INPUT (APPEND ONLY)
                // =========================
                (Focus::Jql, KeyCode::Char(c), _) => {
                    state.jql.push(c);
                }

                (Focus::Jql, KeyCode::Backspace, _) => {
                    state.jql.pop();
                }

                // optional: clear line (Ctrl+u like shell)
                // (Focus::Jql, KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                //     state.jql.clear();
                // }

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
