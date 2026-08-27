//! Key event to action mapping.

pub use crate::tui::app::Action;

use crossterm::event::KeyEvent;

use crate::tui::app::App;

/// Map a terminal key event to an application action.
pub fn key_to_action(app: &App, key: KeyEvent) -> Action {
    App::map_key(key, app.screen, app.filter_active, app.quit_confirm)
}
