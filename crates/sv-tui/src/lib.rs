//! TUI de configuración (decisión D3, hito H0.4).
//!
//! La TUI no guarda nada propio: edita el archivo TOML de `sv-config`, que es
//! la única fuente de verdad. Está dividida para que cada parte cambie sola:
//! - [`app`]: estado y reglas de edición. Sin terminal, se prueba con teclas simuladas.
//! - [`ui`]: dibujo de ese estado. No modifica nada.
//! - este módulo: el bucle de eventos, lo único que toca el terminal real.
//!
//! Para añadir un ajuste nuevo: un valor en `Field` (app) y, si hace falta,
//! cómo se muestra (ui). El bucle no cambia.

use std::io;
use std::path::PathBuf;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use sv_config::Config;

pub mod app;
pub mod ui;

pub use app::App;

/// Abre la TUI sobre `config`, que se guardará en `path`.
///
/// Restaura el terminal al salir, también si hay un error.
///
/// # Errors
/// Si falla la lectura de eventos o el dibujo.
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
