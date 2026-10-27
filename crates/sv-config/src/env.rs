//! Overrides from environment variables.
//!
//! They allow tuning without touching the file, for example in Steam's
//! launch option: `STARVEKTOR_MAX_LEVEL=1 %command%`.
//!
//! | Variable | Values |
//! | --- | --- |
//! | `STARVEKTOR_ENABLED` | `1`/`0`, `true`/`false`, `on`/`off` |
//! | `STARVEKTOR_MAX_LEVEL` | `0` to `3` |
//! | `STARVEKTOR_HOOK` | `auto`, `vulkan`, `gamescope` |
//! | `STARVEKTOR_BUDGET_FRACTION` | number in (0, 0.5] |

use sv_types::Level;

use crate::error::ConfigError;
use crate::io::EnvGet;
use crate::model::{Config, HookMode};

impl Config {
    /// Applies the variables present and revalidates the result.
    ///
    /// # Errors
    /// If a variable has a value that is not understood, or the result is out of range.
    pub fn apply_env(&mut self, get: EnvGet<'_>) -> Result<(), ConfigError> {
        if let Some(value) = get("STARVEKTOR_ENABLED") {
            self.general.enabled = parse_bool("STARVEKTOR_ENABLED", &value)?;
        }
        if let Some(value) = get("STARVEKTOR_MAX_LEVEL") {
            let number: u8 = value
                .trim()
                .parse()
                .map_err(|_| env_err("STARVEKTOR_MAX_LEVEL", "expected 0 to 3"))?;
            self.regulator.max_level = Level::try_from(number)
                .map_err(|e| env_err("STARVEKTOR_MAX_LEVEL", &e.to_string()))?;
        }
        if let Some(value) = get("STARVEKTOR_HOOK") {
            self.hook.mode = HookMode::ALL
                .into_iter()
                .find(|mode| mode.as_str() == value.trim().to_lowercase())
                .ok_or_else(|| env_err("STARVEKTOR_HOOK", "expected auto, vulkan or gamescope"))?;
        }
        if let Some(value) = get("STARVEKTOR_BUDGET_FRACTION") {
            self.regulator.budget_fraction = value.trim().parse().map_err(|_| {
                env_err("STARVEKTOR_BUDGET_FRACTION", "expected a number, e.g. 0.12")
            })?;
        }

        let issues = self.validate();
        if issues.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Invalid(issues))
        }
    }
}

fn env_err(var: &'static str, message: &str) -> ConfigError {
    ConfigError::Env {
        var,
        message: message.to_owned(),
    }
}

fn parse_bool(var: &'static str, value: &str) -> Result<bool, ConfigError> {
    match value.trim().to_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Ok(true),
        "0" | "false" | "off" | "no" => Ok(false),
        _ => Err(env_err(var, "expected 1/0, true/false or on/off")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(pairs: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let get = |name: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_owned())
        };
        let mut cfg = Config::default();
        cfg.apply_env(&get)?;
        Ok(cfg)
    }

    #[test]
    fn no_variables_changes_nothing() {
        assert_eq!(apply(&[]).unwrap(), Config::default());
    }

    #[test]
    fn variables_override_the_file_values() {
        let cfg = apply(&[
            ("STARVEKTOR_ENABLED", "off"),
            ("STARVEKTOR_MAX_LEVEL", "1"),
            ("STARVEKTOR_HOOK", "Gamescope"),
            ("STARVEKTOR_BUDGET_FRACTION", "0.1"),
        ])
        .unwrap();
        assert!(!cfg.general.enabled);
        assert_eq!(cfg.regulator.max_level, Level::Classic);
        assert_eq!(cfg.hook.mode, HookMode::Gamescope);
        assert!((cfg.regulator.budget_fraction - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn bad_values_name_the_variable() {
        for (var, value) in [
            ("STARVEKTOR_ENABLED", "maybe"),
            ("STARVEKTOR_MAX_LEVEL", "7"),
            ("STARVEKTOR_MAX_LEVEL", "high"),
            ("STARVEKTOR_HOOK", "directx"),
            ("STARVEKTOR_BUDGET_FRACTION", "lots"),
        ] {
            let err = apply(&[(var, value)]).unwrap_err();
            assert!(err.to_string().contains(var), "{var}={value}: {err}");
        }
    }

    #[test]
    fn out_of_range_results_are_rejected() {
        let err = apply(&[("STARVEKTOR_BUDGET_FRACTION", "0.9")]).unwrap_err();
        assert!(matches!(err, ConfigError::Invalid(_)));
    }
}
