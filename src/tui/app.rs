//! TUI application state and event reducers.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;

use cargo_scrub::engine::{CrateInfo, ScrubEvent, ScrubOptions};
use cargo_scrub::report::{format_size, SummaryReport};

/// Current screen in the TUI flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Scanning,
    Review,
    Running,
    Summary,
    Empty,
}

/// Per-crate cleaning status shown in the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrateStatus {
    Pending,
    Cleaning,
    Done,
    Skipped,
    Error(String),
}

/// A row in the crate table.
#[derive(Debug, Clone)]
pub struct CrateRow {
    pub info: CrateInfo,
    pub status: CrateStatus,
}

/// User-facing actions mapped from key events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    ToggleSelect,
    SelectAll,
    SelectNone,
    StartClean,
    ToggleDryRun,
    ToggleHelp,
    StartFilter,
    ConfirmFilter,
    CancelFilter,
    FilterInput(char),
    FilterBackspace,
    ConfirmQuit,
    CancelQuit,
    None,
}

/// Main TUI application state.
pub struct App {
    pub screen: Screen,
    pub options: ScrubOptions,
    pub crates: Vec<CrateRow>,
    pub table_state: TableState,
    pub show_help: bool,
    pub filter_active: bool,
    pub filter_input: String,
    pub filter_query: Option<String>,
    pub quit_confirm: bool,
    pub should_quit: bool,
    pub pending_clean: bool,
    pub summary: Option<SummaryReport>,
    pub reclaimed_bytes: u64,
    pub scan_visited: u64,
    pub run_start: Option<Instant>,
    pub status_line: String,
    pub tick_count: usize,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

impl App {
    pub fn new(options: ScrubOptions) -> Self {
        Self {
            screen: Screen::Scanning,
            options,
            crates: Vec::new(),
            table_state: TableState::default(),
            show_help: false,
            filter_active: false,
            filter_input: String::new(),
            filter_query: None,
            quit_confirm: false,
            should_quit: false,
            pending_clean: false,
            summary: None,
            reclaimed_bytes: 0,
            scan_visited: 0,
            run_start: None,
            status_line: String::from("Scanning for Rust projects..."),
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn spinner(&self) -> &'static str {
        SPINNER_FRAMES[(self.tick_count / 3) % SPINNER_FRAMES.len()]
    }

    pub fn selected_count(&self) -> usize {
        self.crates.iter().filter(|r| r.info.selected).count()
    }

    pub fn cleaned_count(&self) -> usize {
        self.crates.iter().filter(|r| r.status == CrateStatus::Done).count()
    }

    pub fn active_count(&self) -> usize {
        self.crates.iter().filter(|r| r.status == CrateStatus::Cleaning).count()
    }

    pub fn reclaimable_bytes(&self) -> u64 {
        self.crates
            .iter()
            .filter(|r| r.info.selected)
            .map(|r| r.info.target_size)
            .sum()
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        let query = self
            .filter_query
            .as_deref()
            .or(self.filter_input.as_str().trim().is_empty().then_some(""))
            .filter(|q| !q.is_empty());

        self.crates
            .iter()
            .enumerate()
            .filter_map(|(i, row)| {
                if let Some(q) = query {
                    let path = row.info.path.to_string_lossy();
                    if path.contains(q) {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    Some(i)
                }
            })
            .collect()
    }

    pub fn selected_crates(&self) -> Vec<CrateInfo> {
        self.crates
            .iter()
            .filter(|r| r.info.selected)
            .map(|r| r.info.clone())
            .collect()
    }

    pub fn elapsed(&self) -> Option<Duration> {
        self.run_start.map(|s| s.elapsed())
    }

    pub fn map_key(key: KeyEvent, screen: Screen, filter_active: bool, quit_confirm: bool) -> Action {
        if quit_confirm {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmQuit,
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => Action::CancelQuit,
                _ => Action::None,
            };
        }

        if filter_active {
            return match key.code {
                KeyCode::Enter => Action::ConfirmFilter,
                KeyCode::Esc => Action::CancelFilter,
                KeyCode::Backspace => Action::FilterBackspace,
                KeyCode::Char(c) => Action::FilterInput(c),
                _ => Action::None,
            };
        }

        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Esc => Action::Quit,
            KeyCode::Char('?') => Action::ToggleHelp,
            KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::MoveBottom,
            KeyCode::Char('g') => Action::MoveTop,
            KeyCode::Char('G') => Action::MoveBottom,
            KeyCode::Char(' ') if screen == Screen::Review => Action::ToggleSelect,
            KeyCode::Char('a') if screen == Screen::Review => Action::SelectAll,
            KeyCode::Char('A') if screen == Screen::Review => Action::SelectNone,
            KeyCode::Char('/') if screen == Screen::Review => Action::StartFilter,
            KeyCode::Char('d') if screen == Screen::Review => Action::ToggleDryRun,
            KeyCode::Char('c') | KeyCode::Enter if screen == Screen::Review => Action::StartClean,
            _ => Action::None,
        }
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => {
                if self.screen == Screen::Running {
                    self.quit_confirm = true;
                } else {
                    self.should_quit = true;
                }
            }
            Action::ConfirmQuit => self.should_quit = true,
            Action::CancelQuit => self.quit_confirm = false,
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            Action::MoveTop => self.select_index(0),
            Action::MoveBottom => {
                let visible = self.visible_indices();
                if let Some(&last) = visible.last() {
                    self.select_row(last);
                }
            }
            Action::ToggleSelect => self.toggle_current(),
            Action::SelectAll => {
                for row in &mut self.crates {
                    row.info.selected = true;
                }
            }
            Action::SelectNone => {
                for row in &mut self.crates {
                    row.info.selected = false;
                }
            }
            Action::StartClean => {
                if self.selected_count() > 0 {
                    self.pending_clean = true;
                }
            }
            Action::ToggleDryRun => {
                self.options.dry_run = !self.options.dry_run;
            }
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::StartFilter => {
                self.filter_active = true;
                self.filter_input.clear();
            }
            Action::ConfirmFilter => {
                let trimmed = self.filter_input.trim().to_string();
                self.filter_query = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                };
                self.filter_active = false;
                self.select_index(0);
            }
            Action::CancelFilter => {
                self.filter_active = false;
                self.filter_input.clear();
            }
            Action::FilterInput(c) => self.filter_input.push(c),
            Action::FilterBackspace => {
                self.filter_input.pop();
            }
            Action::None => {}
        }
    }

    pub fn handle_scrub_event(&mut self, event: ScrubEvent) {
        match event {
            ScrubEvent::ScanStarted => {
                self.screen = Screen::Scanning;
                self.status_line = String::from("Scanning for Rust projects...");
            }
            ScrubEvent::ScanProgress { visited } => {
                self.scan_visited = visited;
            }
            ScrubEvent::ScanComplete { crates } => {
                if crates.is_empty() {
                    self.screen = Screen::Empty;
                    self.status_line = String::from("No Rust projects found.");
                } else {
                    self.screen = Screen::Review;
                    self.crates = crates
                        .into_iter()
                        .map(|info| CrateRow {
                            info,
                            status: CrateStatus::Pending,
                        })
                        .collect();
                    self.table_state.select(Some(0));
                    self.status_line = format!(
                        "Found {} projects ({} reclaimable). Press c to clean.",
                        self.crates.len(),
                        format_size(self.reclaimable_bytes())
                    );
                }
            }
            ScrubEvent::CleanStarted { path } => {
                if let Some(row) = self.crates.iter_mut().find(|r| r.info.path == path) {
                    row.status = CrateStatus::Cleaning;
                }
                self.status_line = format!("Cleaning {}", path.display());
            }
            ScrubEvent::CleanFinished {
                path,
                success,
                error,
                ..
            } => {
                if let Some(row) = self.crates.iter_mut().find(|r| r.info.path == path) {
                    row.status = if let Some(err) = error {
                        if err == "skipped by user" {
                            CrateStatus::Skipped
                        } else if success {
                            CrateStatus::Done
                        } else {
                            CrateStatus::Error(err)
                        }
                    } else if success {
                        CrateStatus::Done
                    } else {
                        CrateStatus::Skipped
                    };
                }
            }
            ScrubEvent::AllComplete {
                report,
                reclaimed_bytes,
            } => {
                self.screen = Screen::Summary;
                self.summary = Some(report);
                self.reclaimed_bytes = reclaimed_bytes;
                self.status_line = format!(
                    "Done. Reclaimed {}. Press q to exit.",
                    format_size(reclaimed_bytes)
                );
            }
            ScrubEvent::Error { message } => {
                self.status_line = message;
            }
        }
    }

    fn visible_selection_pos(&self) -> Option<usize> {
        let selected = self.table_state.selected()?;
        let visible = self.visible_indices();
        visible.iter().position(|&i| i == selected)
    }

    fn move_selection(&mut self, delta: i32) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let current = self
            .visible_selection_pos()
            .unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, visible.len() as i32 - 1) as usize;
        self.select_row(visible[next]);
    }

    fn select_index(&mut self, visible_index: usize) {
        let visible = self.visible_indices();
        if let Some(&row) = visible.get(visible_index) {
            self.select_row(row);
        }
    }

    fn select_row(&mut self, row_index: usize) {
        self.table_state.select(Some(row_index));
        if let Some(row) = self.crates.get(row_index) {
            self.status_line = row.info.path.display().to_string();
        }
    }

    fn toggle_current(&mut self) {
        if let Some(i) = self.table_state.selected() {
            if let Some(row) = self.crates.get_mut(i) {
                row.info.selected = !row.info.selected;
            }
        }
    }

    pub fn begin_clean(&mut self) {
        self.screen = Screen::Running;
        self.run_start = Some(Instant::now());
        self.pending_clean = false;
        for row in &mut self.crates {
            if row.info.selected {
                row.status = CrateStatus::Pending;
            } else {
                row.status = CrateStatus::Skipped;
            }
        }
        self.status_line = String::from("Cleaning selected projects...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_scrub::engine::WorkspaceMode;
    use cargo_scrub::filter::CrateFilter;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;

    fn sample_options() -> ScrubOptions {
        ScrubOptions {
            root: PathBuf::from("."),
            max_depth: None,
            dry_run: false,
            jobs: 4,
            skip_workspaces: false,
            workspace_mode: WorkspaceMode::Members,
            filter: CrateFilter::from_options(None, None).unwrap(),
            selected: None,
        }
    }

    fn sample_row(path: &str, selected: bool) -> CrateRow {
        CrateRow {
            info: CrateInfo {
                path: PathBuf::from(path),
                is_workspace_root: false,
                target_size: 1024,
                selected,
            },
            status: CrateStatus::Pending,
        }
    }

    #[test]
    fn test_toggle_select_action() {
        let mut app = App::new(sample_options());
        app.crates = vec![sample_row("/a", true), sample_row("/b", true)];
        app.table_state.select(Some(1));
        app.apply_action(Action::ToggleSelect);
        assert!(!app.crates[1].info.selected);
    }

    #[test]
    fn test_select_all_none() {
        let mut app = App::new(sample_options());
        app.crates = vec![sample_row("/a", false), sample_row("/b", false)];
        app.apply_action(Action::SelectAll);
        assert_eq!(app.selected_count(), 2);
        app.apply_action(Action::SelectNone);
        assert_eq!(app.selected_count(), 0);
    }

    #[test]
    fn test_filter_visible_indices() {
        let mut app = App::new(sample_options());
        app.crates = vec![
            sample_row("/projects/foo", true),
            sample_row("/projects/bar", true),
        ];
        app.filter_query = Some("foo".to_string());
        let visible = app.visible_indices();
        assert_eq!(visible.len(), 1);
        assert_eq!(app.crates[visible[0]].info.path, PathBuf::from("/projects/foo"));
    }

    #[test]
    fn test_quit_confirm_while_running() {
        let mut app = App::new(sample_options());
        app.screen = Screen::Running;
        let action = App::map_key(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            Screen::Running,
            false,
            false,
        );
        app.apply_action(action);
        assert!(app.quit_confirm);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_scan_complete_empty() {
        let mut app = App::new(sample_options());
        app.handle_scrub_event(ScrubEvent::ScanComplete { crates: vec![] });
        assert_eq!(app.screen, Screen::Empty);
    }

    #[test]
    fn test_move_navigation_and_top_bottom() {
        let mut app = App::new(sample_options());
        app.crates = vec![
            sample_row("/a", true),
            sample_row("/b", true),
            sample_row("/c", true),
        ];
        app.table_state.select(Some(0));

        app.apply_action(Action::MoveDown);
        assert_eq!(app.table_state.selected(), Some(1));

        app.apply_action(Action::MoveDown);
        assert_eq!(app.table_state.selected(), Some(2));

        app.apply_action(Action::MoveUp);
        assert_eq!(app.table_state.selected(), Some(1));

        app.apply_action(Action::MoveBottom);
        assert_eq!(app.table_state.selected(), Some(2));

        app.apply_action(Action::MoveTop);
        assert_eq!(app.table_state.selected(), Some(0));
    }

    #[test]
    fn test_toggle_dry_run_action() {
        let mut app = App::new(sample_options());
        assert!(!app.options.dry_run);
        app.apply_action(Action::ToggleDryRun);
        assert!(app.options.dry_run);
        app.apply_action(Action::ToggleDryRun);
        assert!(!app.options.dry_run);
    }

    #[test]
    fn test_clean_lifecycle_events() {
        let mut app = App::new(sample_options());
        let path = PathBuf::from("/a");
        app.crates = vec![sample_row("/a", true)];
        app.begin_clean();
        assert_eq!(app.screen, Screen::Running);
        assert_eq!(app.crates[0].status, CrateStatus::Pending);

        app.handle_scrub_event(ScrubEvent::CleanStarted { path: path.clone() });
        assert_eq!(app.crates[0].status, CrateStatus::Cleaning);

        app.handle_scrub_event(ScrubEvent::CleanFinished {
            path: path.clone(),
            success: true,
            error: None,
            duration: std::time::Duration::from_millis(50),
        });
        assert_eq!(app.crates[0].status, CrateStatus::Done);

        let report = SummaryReport {
            cleaned: 1,
            skipped: 0,
            errors: 0,
            total: 1,
            duration: std::time::Duration::from_millis(50),
            details: vec![(path, true, None)],
        };
        app.handle_scrub_event(ScrubEvent::AllComplete {
            report,
            reclaimed_bytes: 1024,
        });
        assert_eq!(app.screen, Screen::Summary);
        assert_eq!(app.reclaimed_bytes, 1024);
    }

    #[test]
    fn test_filter_typing_and_cancel() {
        let mut app = App::new(sample_options());
        app.apply_action(Action::StartFilter);
        assert!(app.filter_active);

        app.apply_action(Action::FilterInput('f'));
        app.apply_action(Action::FilterInput('o'));
        app.apply_action(Action::FilterInput('o'));
        assert_eq!(app.filter_input, "foo");

        app.apply_action(Action::FilterBackspace);
        assert_eq!(app.filter_input, "fo");

        app.apply_action(Action::ConfirmFilter);
        assert!(!app.filter_active);
        assert_eq!(app.filter_query, Some("fo".to_string()));

        app.apply_action(Action::StartFilter);
        app.apply_action(Action::CancelFilter);
        assert!(!app.filter_active);
        assert!(app.filter_input.is_empty());
    }
}
