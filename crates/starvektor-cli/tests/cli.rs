//! Pruebas del ejecutable real (`starvektor`), de punta a punta.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Directorio temporal propio de cada prueba.
struct Sandbox {
    dir: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("sv-cli-{}-{id}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Ejecuta `starvektor` aislado: sin terminal y con un HOME propio, para
    /// no tocar la configuración real ni heredar variables STARVEKTOR_*.
    fn run(&self, args: &[&str]) -> Output {
        self.run_with_env(args, &[])
    }

    fn run_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_starvektor"));
        command
            .args(args)
            .stdin(Stdio::null())
            .env("HOME", &self.dir)
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("XDG_STATE_HOME")
            .env_remove("STARVEKTOR_CONFIG")
            .env_remove("STARVEKTOR_ENABLED")
            .env_remove("STARVEKTOR_MAX_LEVEL")
            .env_remove("STARVEKTOR_HOOK")
            .env_remove("STARVEKTOR_BUDGET_FRACTION");
        for (key, value) in env {
            command.env(key, value);
        }
        command.output().unwrap()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn s(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn selftest_writes_a_log_that_validates() {
    let sb = Sandbox::new("selftest");
    let log = sb.path("run.jsonl");

    let out = sb.run(&["selftest", "--frames", "300", "--out", s(&log)]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("selftest ok"), "{stdout}");
    assert!(stdout.contains("nivel 0 -> 1 (needed)"), "{stdout}");

    let validated = sb.run(&["log", "validate", s(&log)]);
    assert!(validated.status.success(), "{}", text(&validated.stderr));
    assert!(
        text(&validated.stdout).starts_with("ok:"),
        "{}",
        text(&validated.stdout)
    );
}

#[test]
fn selftest_defaults_to_the_state_directory() {
    let sb = Sandbox::new("default-dir");
    let out = sb.run(&["selftest", "--frames", "50"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let logs = sb.dir.join(".local/state/starvektor/logs");
    let files: Vec<_> = fs::read_dir(&logs).unwrap().collect();
    assert_eq!(files.len(), 1);
}

#[test]
fn validate_rejects_a_corrupted_log_with_line_numbers() {
    let sb = Sandbox::new("corrupt");
    let log = sb.path("run.jsonl");
    assert!(
        sb.run(&["selftest", "--frames", "100", "--out", s(&log)])
            .status
            .success()
    );

    let mut lines: Vec<String> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    lines[3] = "{esto no es json".to_owned();
    fs::write(&log, lines.join("\n") + "\n").unwrap();

    let out = sb.run(&["log", "validate", s(&log)]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("línea 4"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn validate_reports_a_missing_file_as_an_error() {
    let sb = Sandbox::new("missing");
    let out = sb.run(&["log", "validate", s(&sb.path("nada.jsonl"))]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("no se pudo abrir"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn config_init_show_and_force() {
    let sb = Sandbox::new("config");
    let cfg = sb.path("sub/config.toml");

    let init = sb.run(&["config", "init", "--config", s(&cfg)]);
    assert!(init.status.success(), "{}", text(&init.stderr));
    assert!(cfg.exists());

    let again = sb.run(&["config", "init", "--config", s(&cfg)]);
    assert!(!again.status.success());
    assert!(
        text(&again.stderr).contains("--force"),
        "{}",
        text(&again.stderr)
    );
    assert!(
        sb.run(&["config", "init", "--force", "--config", s(&cfg)])
            .status
            .success()
    );

    let show = sb.run(&["config", "show", "--config", s(&cfg)]);
    assert!(show.status.success());
    let shown = text(&show.stdout);
    assert!(
        shown.contains("[regulator]") && shown.contains("max_level = 3"),
        "{shown}"
    );
}

#[test]
fn environment_variables_override_the_file_in_show() {
    let sb = Sandbox::new("env");
    let cfg = sb.path("config.toml");
    assert!(
        sb.run(&["config", "init", "--config", s(&cfg)])
            .status
            .success()
    );

    let out = sb.run_with_env(
        &["config", "show", "--config", s(&cfg)],
        &[("STARVEKTOR_MAX_LEVEL", "1")],
    );
    assert!(
        text(&out.stdout).contains("max_level = 1"),
        "{}",
        text(&out.stdout)
    );

    let bad = sb.run_with_env(
        &["config", "show", "--config", s(&cfg)],
        &[("STARVEKTOR_MAX_LEVEL", "9")],
    );
    assert!(!bad.status.success());
    assert!(
        text(&bad.stderr).contains("STARVEKTOR_MAX_LEVEL"),
        "{}",
        text(&bad.stderr)
    );
}

#[test]
fn an_invalid_config_file_is_reported_not_ignored() {
    let sb = Sandbox::new("invalid");
    let cfg = sb.path("config.toml");
    fs::write(&cfg, "[regulator]\nbudget_fraction = 0.9\n").unwrap();
    let out = sb.run(&["config", "show", "--config", s(&cfg)]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("regulator.budget_fraction"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn config_path_follows_the_environment() {
    let sb = Sandbox::new("path");
    let default = sb.run(&["config", "path"]);
    assert_eq!(
        text(&default.stdout).trim(),
        sb.dir
            .join(".config/starvektor/config.toml")
            .to_str()
            .unwrap()
    );

    let explicit = sb.run_with_env(
        &["config", "path"],
        &[("STARVEKTOR_CONFIG", "/etc/sv.toml")],
    );
    assert_eq!(text(&explicit.stdout).trim(), "/etc/sv.toml");
}

#[test]
fn tui_refuses_to_run_without_a_terminal() {
    let sb = Sandbox::new("tui");
    let out = sb.run(&["tui"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("terminal interactivo"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn help_and_version_work() {
    let sb = Sandbox::new("help");
    let help = sb.run(&["--help"]);
    assert!(help.status.success());
    for command in ["tui", "config", "log", "selftest"] {
        assert!(
            text(&help.stdout).contains(command),
            "{}",
            text(&help.stdout)
        );
    }
    assert!(sb.run(&["--version"]).status.success());
}
