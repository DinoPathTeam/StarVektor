//! Loading and saving of the configuration file, and default paths.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ConfigError;
use crate::model::Config;

/// Injectable environment variable reader (to test without touching the process).
pub type EnvGet<'a> = &'a dyn Fn(&str) -> Option<String>;

/// Reader of the process's real environment.
#[must_use]
pub fn process_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

/// Path of the configuration file:
/// `$STARVEKTOR_CONFIG`, or `$XDG_CONFIG_HOME/starvektor/config.toml`,
/// or `$HOME/.config/starvektor/config.toml`.
#[must_use]
pub fn default_config_path(get: EnvGet<'_>) -> Option<PathBuf> {
    if let Some(explicit) = get("STARVEKTOR_CONFIG") {
        return Some(PathBuf::from(explicit));
    }
    base_dir(get, "XDG_CONFIG_HOME", ".config").map(|d| d.join("starvektor").join("config.toml"))
}

/// Default log directory:
/// `$XDG_STATE_HOME/starvektor/logs` or `$HOME/.local/state/starvektor/logs`.
#[must_use]
pub fn default_log_dir(get: EnvGet<'_>) -> Option<PathBuf> {
    base_dir(get, "XDG_STATE_HOME", ".local/state").map(|d| d.join("starvektor").join("logs"))
}

fn base_dir(get: EnvGet<'_>, xdg_var: &str, home_relative: &str) -> Option<PathBuf> {
    if let Some(xdg) = get(xdg_var).filter(|v| Path::new(v).is_absolute()) {
        return Some(PathBuf::from(xdg));
    }
    get("HOME").map(|home| PathBuf::from(home).join(home_relative))
}

impl Config {
    /// Reads and validates a TOML.
    ///
    /// # Errors
    /// If the TOML cannot be read, has unknown fields or out-of-range values.
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        let cfg: Config = toml::from_str(text)?;
        cfg.check()
    }

    /// Writes the TOML. Fails if the configuration is not valid.
    ///
    /// # Errors
    /// If the configuration is invalid or cannot be serialized.
    pub fn to_toml_string(&self) -> Result<String, ConfigError> {
        self.clone().check()?;
        Ok(toml::to_string_pretty(self)?)
    }

    /// Loads the file at `path`.
    ///
    /// # Errors
    /// If it cannot be read or its content is not valid.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml_str(&text)
    }

    /// Like [`Config::load`], but a missing file yields the default configuration.
    ///
    /// # Errors
    /// If the file exists and cannot be read or is not valid.
    pub fn load_or_default(path: &Path) -> Result<Self, ConfigError> {
        match fs::read_to_string(path) {
            Ok(text) => Self::from_toml_str(&text),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Io {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    /// Saves atomically (temporary file and rename), creating any missing
    /// directories. Comments from the previous file are not preserved.
    ///
    /// # Errors
    /// If the configuration is invalid or the write fails.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = self.to_toml_string()?;
        let io_err = |source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        };

        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        fs::write(&tmp, text).map_err(io_err)?;
        fs::rename(&tmp, path).map_err(|source| {
            let _ = fs::remove_file(&tmp);
            io_err(source)
        })
    }

    fn check(self) -> Result<Self, ConfigError> {
        let issues = self.validate();
        if issues.is_empty() {
            Ok(self)
        } else {
            Err(ConfigError::Invalid(issues))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use sv_types::Level;

    use super::*;
    use crate::model::{HookMode, Profile};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sv-config-{}-{id}-{name}", std::process::id()))
    }

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn default_config_roundtrips_through_toml() {
        let cfg = Config::default();
        let text = cfg.to_toml_string().unwrap();
        assert_eq!(Config::from_toml_str(&text).unwrap(), cfg);
    }

    #[test]
    fn a_full_config_with_profiles_roundtrips() {
        let mut cfg = Config::default();
        cfg.regulator.max_level = Level::MiniNet;
        cfg.hook.mode = HookMode::Gamescope;
        cfg.telemetry.dir = Some("/tmp/sv".into());
        cfg.profiles.insert(
            "Valorant".into(),
            Profile {
                enabled: Some(false),
                max_level: Some(Level::Off),
                ..Profile::default()
            },
        );
        let text = cfg.to_toml_string().unwrap();
        assert!(text.contains("max_level = 2"), "{text}");
        assert!(text.contains("[profiles.Valorant]"), "{text}");
        assert_eq!(Config::from_toml_str(&text).unwrap(), cfg);
    }

    #[test]
    fn partial_toml_fills_in_defaults() {
        let cfg = Config::from_toml_str("[regulator]\nmax_level = 1\n").unwrap();
        assert_eq!(cfg.regulator.max_level, Level::Classic);
        assert_eq!(cfg.regulator.settle_frames, 120);
        assert!(cfg.general.enabled);
    }

    #[test]
    fn empty_toml_is_the_default() {
        assert_eq!(Config::from_toml_str("").unwrap(), Config::default());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        assert!(matches!(
            Config::from_toml_str("[regulator]\nmax_levl = 1\n"),
            Err(ConfigError::Parse(_))
        ));
        assert!(matches!(
            Config::from_toml_str("surprise = true\n"),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn invalid_level_and_ranges_are_rejected() {
        assert!(matches!(
            Config::from_toml_str("[regulator]\nmax_level = 9\n"),
            Err(ConfigError::Parse(_))
        ));
        let err = Config::from_toml_str("[regulator]\nbudget_fraction = 0.9\n").unwrap_err();
        assert!(
            matches!(err, ConfigError::Invalid(ref v) if v[0].path == "regulator.budget_fraction")
        );
        assert!(err.to_string().contains("budget_fraction"));
    }

    #[test]
    fn saving_an_invalid_config_is_refused_and_writes_nothing() {
        let dir = temp_dir("invalid");
        let path = dir.join("config.toml");
        let mut cfg = Config::default();
        cfg.regulator.budget_fraction = 5.0;
        assert!(matches!(cfg.save(&path), Err(ConfigError::Invalid(_))));
        assert!(!path.exists());
    }

    #[test]
    fn save_then_load_roundtrip_leaves_no_temp_file() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("nested").join("config.toml");
        let mut cfg = Config::default();
        cfg.regulator.max_level = Level::Classic;
        cfg.save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap(), cfg);
        assert!(!dir.join("nested").join("config.toml.tmp").exists());

        cfg.regulator.max_level = Level::Off;
        cfg.save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap().regulator.max_level, Level::Off);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_or_default_handles_a_missing_file_but_not_a_broken_one() {
        let dir = temp_dir("missing");
        assert_eq!(
            Config::load_or_default(&dir.join("nothing.toml")).unwrap(),
            Config::default()
        );
        assert!(matches!(
            Config::load(&dir.join("nothing.toml")),
            Err(ConfigError::Io { .. })
        ));

        fs::create_dir_all(&dir).unwrap();
        let broken = dir.join("broken.toml");
        fs::write(&broken, "this = is not toml [").unwrap();
        assert!(matches!(
            Config::load_or_default(&broken),
            Err(ConfigError::Parse(_))
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn config_path_resolution_order() {
        let explicit = env(&[("STARVEKTOR_CONFIG", "/etc/sv.toml"), ("HOME", "/home/u")]);
        assert_eq!(
            default_config_path(&explicit),
            Some(PathBuf::from("/etc/sv.toml"))
        );

        let xdg = env(&[("XDG_CONFIG_HOME", "/cfg"), ("HOME", "/home/u")]);
        assert_eq!(
            default_config_path(&xdg),
            Some(PathBuf::from("/cfg/starvektor/config.toml"))
        );

        let home = env(&[("HOME", "/home/u")]);
        assert_eq!(
            default_config_path(&home),
            Some(PathBuf::from("/home/u/.config/starvektor/config.toml"))
        );

        let relative_xdg = env(&[("XDG_CONFIG_HOME", "relative"), ("HOME", "/home/u")]);
        assert_eq!(
            default_config_path(&relative_xdg),
            Some(PathBuf::from("/home/u/.config/starvektor/config.toml")),
            "relative XDG is ignored, per the specification"
        );

        assert_eq!(default_config_path(&env(&[])), None);
    }

    #[test]
    fn log_dir_resolution() {
        let state = env(&[("XDG_STATE_HOME", "/st")]);
        assert_eq!(
            default_log_dir(&state),
            Some(PathBuf::from("/st/starvektor/logs"))
        );
        let home = env(&[("HOME", "/home/u")]);
        assert_eq!(
            default_log_dir(&home),
            Some(PathBuf::from("/home/u/.local/state/starvektor/logs"))
        );
    }
}
