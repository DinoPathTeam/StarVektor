//! Dibujo de la TUI. Solo lee el estado de [`App`]; no lo modifica.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::{App, Field, Status};

/// Ancho de la columna de etiquetas.
const LABEL_WIDTH: usize = 32;

pub fn render(frame: &mut Frame, app: &App) {
    let [header, body, help, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    let dirty_mark = if app.is_dirty() { " *" } else { "" };
    let title = Paragraph::new(Line::from(vec![
        Span::styled("StarVektor", Style::new().add_modifier(Modifier::BOLD)),
        Span::raw(" · configuración"),
        Span::styled(dirty_mark, Style::new().fg(Color::Yellow)),
    ]))
    .block(Block::bordered().title(app.path().display().to_string()));
    frame.render_widget(title, header);

    let items: Vec<ListItem> = Field::ALL
        .iter()
        .map(|field| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<LABEL_WIDTH$}", field.label())),
                Span::styled(app.value(*field), Style::new().fg(Color::Cyan)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title("Ajustes"))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    let mut state = ListState::default().with_selected(Some(app.selected()));
    frame.render_stateful_widget(list, body, &mut state);

    let explanation = Paragraph::new(app.selected_field().help())
        .wrap(Wrap { trim: true })
        .block(Block::bordered().title("Qué hace"));
    frame.render_widget(explanation, help);

    let keys = "↑↓ mover · ←→ cambiar · espacio alterna · s guardar · r recargar · q salir";
    let (status_text, status_style) = match app.status() {
        Status::None => (String::new(), Style::new()),
        Status::Saved => ("Guardado.".to_owned(), Style::new().fg(Color::Green)),
        Status::Reloaded => (
            "Recargado desde el archivo.".to_owned(),
            Style::new().fg(Color::Green),
        ),
        Status::ConfirmQuit => (
            "Hay cambios sin guardar: q otra vez sale sin guardar, s guarda.".to_owned(),
            Style::new().fg(Color::Yellow),
        ),
        Status::Error(message) => (format!("Error: {message}"), Style::new().fg(Color::Red)),
    };
    let footer_text = Paragraph::new(vec![
        Line::styled(status_text, status_style),
        Line::styled(keys, Style::new().add_modifier(Modifier::DIM)),
    ]);
    frame.render_widget(footer_text, footer);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use sv_config::Config;

    use super::*;

    fn draw(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let mut text = String::new();
        for y in 0..height {
            for x in 0..width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    fn app() -> App {
        App::new(
            Config::default(),
            PathBuf::from("/tmp/sv-ui-test/config.toml"),
        )
    }

    #[test]
    fn shows_every_setting_with_its_value() {
        let text = draw(&app(), 90, 24);
        for field in Field::ALL {
            assert!(
                text.contains(field.label()),
                "falta {:?}\n{text}",
                field.label()
            );
        }
        assert!(text.contains("3 · red completa"), "{text}");
        assert!(text.contains("12 %"), "{text}");
        assert!(
            text.contains("config.toml"),
            "muestra la ruta del archivo\n{text}"
        );
    }

    #[test]
    fn marks_the_selected_row_and_explains_it() {
        let mut app = app();
        app.select(1);
        let text = draw(&app, 90, 24);
        assert!(text.contains("▶ Nivel máximo"), "{text}");
        assert!(text.contains("Tope del regulador"), "{text}");
    }

    #[test]
    fn shows_the_unsaved_marker_and_the_confirmation() {
        let mut app = app();
        assert!(!draw(&app, 90, 24).contains("configuración *"));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(draw(&app, 90, 24).contains("configuración *"));
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(draw(&app, 90, 24).contains("cambios sin guardar"));
    }

    #[test]
    fn does_not_panic_on_a_tiny_terminal() {
        let _ = draw(&app(), 20, 6);
        let _ = draw(&app(), 1, 1);
    }
}
