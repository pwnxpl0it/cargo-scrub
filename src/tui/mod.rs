//! TUI entry point and event loop.

mod app;
mod events;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use cargo_scrub::engine::{clean_selected, discover_crates, ScrubEvent, ScrubOptions};
use cargo_scrub::report::format_size;

use app::App;
use events::key_to_action;

/// Run the full-screen TUI dashboard.
pub async fn run_tui(options: ScrubOptions) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<ScrubEvent>();
    let mut app = App::new(options.clone());

    let discover_options = options.clone();
    let discover_tx = event_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = discover_crates(&discover_options, None, Some(discover_tx)).await {
            let _ = event_tx.send(ScrubEvent::Error {
                message: e.to_string(),
            });
        }
    });

    let mut clean_rx: Option<mpsc::UnboundedReceiver<ScrubEvent>> = None;

    let result = loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        while let Ok(scrub_event) = event_rx.try_recv() {
            app.handle_scrub_event(scrub_event);
        }

        if let Some(ref mut rx) = clean_rx {
            while let Ok(scrub_event) = rx.try_recv() {
                app.handle_scrub_event(scrub_event);
            }
        }

        if app.pending_clean {
            app.begin_clean();
            let paths = app.selected_paths();
            let clean_options = options.clone();
            let (tx, rx) = mpsc::unbounded_channel();
            clean_rx = Some(rx);
            tokio::spawn(async move {
                let _ = clean_selected(&clean_options, paths, Some(tx), false, None).await;
            });
        }

        if app.should_quit {
            break Ok(());
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if app.show_help
                    && matches!(key.code, event::KeyCode::Esc | event::KeyCode::Char('?'))
                {
                    app.show_help = false;
                    continue;
                }
                let action = key_to_action(&app, key);
                app.apply_action(action);
            }
        }
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Some(ref report) = app.summary {
        println!(
            "cargo-scrub: cleaned {} crate(s), reclaimed {}",
            report.cleaned,
            format_size(app.reclaimed_bytes)
        );
    }

    result
}
