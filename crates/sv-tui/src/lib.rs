//! Configuration TUI (decision D3, milestone H0.4).
//!
//! The TUI stores nothing of its own: it edits the `sv-config` TOML file, which is
//! the single source of truth. It is split so each part can change on its own:
//! - [`app`]: state and editing rules. Terminal-free, tested with simulated keys.
//! - [`ui`]: drawing of that state. It modifies nothing.
//! - this module: the event loop, the only thing that touches the real terminal.
//!
//! To add a new setting: a value in `Field` (app) and, if needed,
//! how it is displayed (ui). The loop does not change.

use std::io;
use std::path::PathBuf;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use sv_config::Config;

pub mod app;
pub mod ui;

pub use app::App;

/// Opens the TUI on `config`, which will be saved to `path`.
///
/// Restores the terminal on exit, also on error.
///
/// # Errors
/// If reading events or drawing fails.
pub fn run(config: Config, path: PathBuf) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, App::new(config, path));
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut DefaultTerminal, mut app: App) -> io::Result<()> {
    while !app.should_quit() {
        terminal.draw(|frame| ui::render(frame, &app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
        }
    }
    Ok(())
}
