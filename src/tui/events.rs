//! Key event to action mapping.

pub use crate::tui::app::Action;

use crossterm::event::{KeyEvent, KeyEventKind};

use crate::tui::app::App;

/// Map a terminal key event to an application action.
///
/// On Windows, crossterm reports both `KeyEventKind::Press` and
/// `KeyEventKind::Release` (and, while held, `KeyEventKind::Repeat`)
/// for a single physical key press. Only presses should drive actions,
/// otherwise a single arrow-key press moves the selection twice.
pub fn key_to_action(app: &App, key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }
    App::map_key(key, app.screen, app.filter_active, app.quit_confirm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_scrub::engine::{ScrubOptions, WorkspaceMode};
    use cargo_scrub::filter::CrateFilter;
    use crossterm::event::{KeyCode, KeyModifiers};
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

    fn key(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind)
    }

    /// Regression: on Windows a single arrow-key press emits Press + Release
    /// (and Repeat while held), which must not both move the selection.
    #[test]
    fn non_press_arrow_keys_do_not_move_selection() {
        use cargo_scrub::engine::CrateInfo;

        let mut app = App::new(sample_options());
        app.screen = crate::tui::app::Screen::Review;
        app.crates = vec![
            crate::tui::app::CrateRow {
                info: CrateInfo {
                    path: PathBuf::from("/a"),
                    is_workspace_root: false,
                    target_size: 0,
                    selected: false,
                },
                status: crate::tui::app::CrateStatus::Pending,
            },
            crate::tui::app::CrateRow {
                info: CrateInfo {
                    path: PathBuf::from("/b"),
                    is_workspace_root: false,
                    target_size: 0,
                    selected: false,
                },
                status: crate::tui::app::CrateStatus::Pending,
            },
        ];
        app.table_state.select(Some(0));

        let action = key_to_action(&app, key(KeyCode::Down, KeyEventKind::Press));
        assert_eq!(action, Action::MoveDown);
        app.apply_action(action);

        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let action = key_to_action(&app, key(KeyCode::Down, kind));
            assert_eq!(action, Action::None);
            app.apply_action(action);
        }

        assert_eq!(app.table_state.selected(), Some(1));
    }
}
