//! TUI state and editing rules. Does not depend on the terminal.

use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sv_config::{Config, HookMode};
use sv_types::Level;

/// Editable settings, in the order they are displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Enabled,
    MaxLevel,
    BudgetFraction,
    VramFloorMb,
    VramMarginMb,
    SettleFrames,
    PanicFrames,
    Hook,
    TelemetryEnabled,
    Overlay,
}

impl Field {
    pub const ALL: [Field; 10] = [
        Field::Enabled,
        Field::MaxLevel,
        Field::BudgetFraction,
        Field::VramFloorMb,
        Field::VramMarginMb,
        Field::SettleFrames,
        Field::PanicFrames,
        Field::Hook,
        Field::TelemetryEnabled,
        Field::Overlay,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Field::Enabled => "StarVektor enabled",
            Field::MaxLevel => "Maximum level",
            Field::BudgetFraction => "Time budget",
            Field::VramFloorMb => "Minimum free VRAM",
            Field::VramMarginMb => "VRAM margin to rise",
            Field::SettleFrames => "Frames to rise",
            Field::PanicFrames => "Frames to drop",
            Field::Hook => "Hook",
            Field::TelemetryEnabled => "Write log",
            Field::Overlay => "Overlay",
        }
    }

    /// One-line explanation of the selected setting.
    #[must_use]
    pub fn help(self) -> &'static str {
        match self {
            Field::Enabled => "Master switch. When off, everything passes through untouched.",
            Field::MaxLevel => {
                "Regulator cap: 0 off, 1 classic, 2 minimal network, 3 full network."
            }
            Field::BudgetFraction => {
                "Share of the game's base frame time StarVektor may spend per frame."
            }
            Field::VramFloorMb => "If free VRAM drops below this, the regulator drops a level.",
            Field::VramMarginMb => "Extra free VRAM above the minimum needed to rise a level.",
            Field::SettleFrames => {
                "Consecutive good frames before rising. Higher means less oscillation."
            }
            Field::PanicFrames => {
                "Consecutive bad frames before dropping. Higher means more tolerance to spikes."
            }
            Field::Hook => {
                "auto picks based on the measurement; vulkan = implicit layer; gamescope = inside gamescope."
            }
            Field::TelemetryEnabled => "Writes one JSONL log per run (no images or game data).",
            Field::Overlay => "Shows frame time, active level and the artifact mark key.",
        }
    }
}

/// Result of the last action, for the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    None,
    Saved,
    Reloaded,
    Error(String),
    /// Quit was requested with unsaved changes; another `q` confirms.
    ConfirmQuit,
}

#[derive(Debug)]
pub struct App {
    config: Config,
    path: PathBuf,
    selected: usize,
    dirty: bool,
    status: Status,
    quit: bool,
}

impl App {
    #[must_use]
    pub fn new(config: Config, path: PathBuf) -> Self {
        Self {
            config,
            path,
            selected: 0,
            dirty: false,
            status: Status::None,
            quit: false,
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }

    #[must_use]
    pub fn selected_field(&self) -> Field {
        Field::ALL[self.selected]
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[must_use]
    pub fn status(&self) -> &Status {
        &self.status
    }

    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Value of the setting, ready to display.
    #[must_use]
    pub fn value(&self, field: Field) -> String {
        let r = &self.config.regulator;
        match field {
            Field::Enabled => on_off(self.config.general.enabled),
            Field::MaxLevel => level_text(r.max_level).to_owned(),
            Field::BudgetFraction => format!("{:.0} %", r.budget_fraction * 100.0),
            Field::VramFloorMb => format!("{} MB", r.vram_floor_mb),
            Field::VramMarginMb => format!("{} MB", r.vram_margin_mb),
            Field::SettleFrames => r.settle_frames.to_string(),
            Field::PanicFrames => r.panic_frames.to_string(),
            Field::Hook => self.config.hook.mode.as_str().to_owned(),
            Field::TelemetryEnabled => on_off(self.config.telemetry.enabled),
            Field::Overlay => on_off(self.config.telemetry.overlay),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }

        let was_confirming = self.status == Status::ConfirmQuit;
        if !matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
            self.status = Status::None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select(-1),
            KeyCode::Down | KeyCode::Char('j') => self.select(1),
            KeyCode::Left | KeyCode::Char('h') => self.adjust(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') | KeyCode::Enter => {
                self.adjust(1)
            }
            KeyCode::Char('s') => self.save(),
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.dirty && !was_confirming {
                    self.status = Status::ConfirmQuit;
                } else {
                    self.quit = true;
                }
            }
            _ => {}
        }
    }

    /// Moves the selection, wrapping around at the edge.
    pub fn select(&mut self, delta: i32) {
        let len = i32::try_from(Field::ALL.len()).unwrap_or(1);
        let current = i32::try_from(self.selected).unwrap_or(0);
        self.selected = usize::try_from((current + delta).rem_euclid(len)).unwrap_or(0);
    }

    /// Changes the selected setting by one step (`delta` = -1 or +1).
    pub fn adjust(&mut self, delta: i32) {
        let before = self.config.clone();
        let field = self.selected_field();
        let r = &mut self.config.regulator;
        match field {
            Field::Enabled => self.config.general.enabled = !self.config.general.enabled,
            Field::MaxLevel => {
                r.max_level = if delta < 0 {
                    r.max_level.down()
                } else {
                    r.max_level.up()
                };
            }
            Field::BudgetFraction => {
                let hundredths = (r.budget_fraction * 100.0).round() + f64::from(delta);
                r.budget_fraction = (hundredths / 100.0).clamp(0.01, 0.50);
            }
            Field::VramFloorMb => r.vram_floor_mb = step(r.vram_floor_mb, delta * 50, 0, 65_536),
            Field::VramMarginMb => r.vram_margin_mb = step(r.vram_margin_mb, delta * 50, 0, 65_536),
            Field::SettleFrames => r.settle_frames = step(r.settle_frames, delta * 10, 1, 3_600),
            Field::PanicFrames => r.panic_frames = step(r.panic_frames, delta, 1, 60),
            Field::Hook => self.config.hook.mode = cycle_hook(self.config.hook.mode, delta),
            Field::TelemetryEnabled => {
                self.config.telemetry.enabled = !self.config.telemetry.enabled
            }
            Field::Overlay => self.config.telemetry.overlay = !self.config.telemetry.overlay,
        }
        if self.config != before {
            self.dirty = true;
        }
    }

    /// Saves to the file. If it fails, the changes stay on screen.
    pub fn save(&mut self) {
        match self.config.save(&self.path) {
            Ok(()) => {
                self.dirty = false;
                self.status = Status::Saved;
            }
            Err(err) => self.status = Status::Error(err.to_string()),
        }
    }

    /// Discards the changes and re-reads the file (or the defaults if it does not exist).
    pub fn reload(&mut self) {
        match Config::load_or_default(&self.path) {
            Ok(config) => {
                self.config = config;
                self.dirty = false;
                self.status = Status::Reloaded;
            }
            Err(err) => self.status = Status::Error(err.to_string()),
        }
    }
}

fn on_off(value: bool) -> String {
    if value { "yes" } else { "no" }.to_owned()
}

fn level_text(level: Level) -> &'static str {
    match level {
        Level::Off => "0 · off",
        Level::Classic => "1 · classic",
        Level::MiniNet => "2 · minimal network",
        Level::FullNet => "3 · full network",
    }
}

fn step(value: u32, delta: i32, min: u32, max: u32) -> u32 {
    let next = i64::from(value) + i64::from(delta);
    u32::try_from(next.clamp(i64::from(min), i64::from(max))).unwrap_or(min)
}

fn cycle_hook(mode: HookMode, delta: i32) -> HookMode {
    let all = HookMode::ALL;
    let len = i32::try_from(all.len()).unwrap_or(1);
    let index = all.iter().position(|m| *m == mode).unwrap_or(0);
    let next = (i32::try_from(index).unwrap_or(0) + delta).rem_euclid(len);
    all[usize::try_from(next).unwrap_or(0)]
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sv-tui-{}-{id}-{name}", std::process::id()))
    }

    fn app() -> App {
        App::new(Config::default(), temp_dir("unused").join("config.toml"))
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn select(app: &mut App, field: Field) {
        while app.selected_field() != field {
            app.handle_key(key(KeyCode::Down));
        }
    }

    #[test]
    fn navigation_wraps_in_both_directions() {
        let mut app = app();
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.selected(), Field::ALL.len() - 1);
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.selected(), 0);
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.selected(), 1);
    }

    #[test]
    fn toggles_flip_and_mark_dirty() {
        let mut app = app();
        assert!(!app.is_dirty());
        app.handle_key(key(KeyCode::Enter));
        assert!(!app.config().general.enabled);
        assert!(app.is_dirty());
        app.handle_key(key(KeyCode::Char(' ')));
        assert!(app.config().general.enabled);
    }

    #[test]
    fn level_steps_and_clamps() {
        let mut app = app();
        select(&mut app, Field::MaxLevel);
        for _ in 0..10 {
            app.handle_key(key(KeyCode::Left));
        }
        assert_eq!(app.config().regulator.max_level, Level::Off);
        for _ in 0..10 {
            app.handle_key(key(KeyCode::Right));
        }
        assert_eq!(app.config().regulator.max_level, Level::FullNet);
        assert_eq!(app.value(Field::MaxLevel), "3 · full network");
    }

    #[test]
    fn budget_moves_in_one_percent_steps_within_bounds() {
        let mut app = app();
        select(&mut app, Field::BudgetFraction);
        assert_eq!(app.value(Field::BudgetFraction), "12 %");
        app.handle_key(key(KeyCode::Right));
        assert!((app.config().regulator.budget_fraction - 0.13).abs() < 1e-9);
        for _ in 0..100 {
            app.handle_key(key(KeyCode::Right));
        }
        assert!((app.config().regulator.budget_fraction - 0.50).abs() < 1e-9);
        for _ in 0..100 {
            app.handle_key(key(KeyCode::Left));
        }
        assert!((app.config().regulator.budget_fraction - 0.01).abs() < 1e-9);
        assert!(
            app.config().validate().is_empty(),
            "nothing the TUI allows is invalid"
        );
    }

    #[test]
    fn numeric_fields_respect_their_limits() {
        let mut app = app();
        select(&mut app, Field::VramFloorMb);
        for _ in 0..20 {
            app.handle_key(key(KeyCode::Left));
        }
        assert_eq!(app.config().regulator.vram_floor_mb, 0);
        select(&mut app, Field::PanicFrames);
        for _ in 0..10 {
            app.handle_key(key(KeyCode::Left));
        }
        assert_eq!(app.config().regulator.panic_frames, 1);
        select(&mut app, Field::SettleFrames);
        for _ in 0..1000 {
            app.handle_key(key(KeyCode::Right));
        }
        assert_eq!(app.config().regulator.settle_frames, 3_600);
        assert!(app.config().validate().is_empty());
    }

    #[test]
    fn hook_cycles_through_all_modes() {
        let mut app = app();
        select(&mut app, Field::Hook);
        let mut seen = vec![app.config().hook.mode];
        for _ in 0..3 {
            app.handle_key(key(KeyCode::Right));
            seen.push(app.config().hook.mode);
        }
        assert_eq!(
            seen,
            [
                HookMode::Auto,
                HookMode::Vulkan,
                HookMode::Gamescope,
                HookMode::Auto
            ]
        );
        app.handle_key(key(KeyCode::Left));
        assert_eq!(app.config().hook.mode, HookMode::Gamescope);
    }

    #[test]
    fn quitting_without_changes_is_immediate() {
        let mut app = app();
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit());
    }

    #[test]
    fn quitting_with_changes_asks_to_confirm() {
        let mut app = app();
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('q')));
        assert!(!app.should_quit());
        assert_eq!(*app.status(), Status::ConfirmQuit);
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit());
    }

    #[test]
    fn any_other_key_cancels_the_quit_confirmation() {
        let mut app = app();
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Esc));
        app.handle_key(key(KeyCode::Down));
        app.handle_key(key(KeyCode::Char('q')));
        assert!(!app.should_quit(), "the confirmation was reset");
    }

    #[test]
    fn ctrl_c_quits_even_with_changes() {
        let mut app = app();
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit());
    }

    #[test]
    fn save_writes_the_file_and_clears_dirty() {
        let dir = temp_dir("save");
        let path = dir.join("sub").join("config.toml");
        let mut app = App::new(Config::default(), path.clone());
        select(&mut app, Field::MaxLevel);
        app.handle_key(key(KeyCode::Left));
        app.handle_key(key(KeyCode::Char('s')));
        assert_eq!(*app.status(), Status::Saved);
        assert!(!app.is_dirty());
        assert_eq!(
            Config::load(&path).unwrap().regulator.max_level,
            Level::MiniNet
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_failed_save_keeps_the_changes_and_reports_the_error() {
        // The parent "directory" is a file: creating the path must fail.
        let dir = temp_dir("fail");
        std::fs::create_dir_all(&dir).unwrap();
        let blocker = dir.join("file");
        std::fs::write(&blocker, "x").unwrap();
        let mut app = App::new(Config::default(), blocker.join("config.toml"));
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('s')));
        assert!(matches!(app.status(), Status::Error(_)));
        assert!(app.is_dirty());
        assert!(!app.config().general.enabled);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reload_discards_unsaved_changes() {
        let dir = temp_dir("reload");
        let path = dir.join("config.toml");
        Config::default().save(&path).unwrap();
        let mut app = App::new(Config::default(), path);
        app.handle_key(key(KeyCode::Enter));
        assert!(app.is_dirty());
        app.handle_key(key(KeyCode::Char('r')));
        assert_eq!(*app.status(), Status::Reloaded);
        assert!(!app.is_dirty());
        assert!(app.config().general.enabled);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn every_field_has_a_label_help_and_value() {
        let app = app();
        for field in Field::ALL {
            assert!(!field.label().is_empty());
            assert!(!field.help().is_empty());
            assert!(!app.value(field).is_empty());
        }
    }
}
