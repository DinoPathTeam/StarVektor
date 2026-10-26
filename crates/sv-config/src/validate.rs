//! Reglas de rango de la configuración.

use crate::error::ConfigIssue;
use crate::model::{CONFIG_VERSION, Config};

/// Máximo razonable de VRAM libre exigida, en MB (evita errores de unidad).
const MAX_VRAM_MB: u32 = 1 << 20;

impl Config {
    /// Lista de problemas; vacía si la configuración es válida.
    #[must_use]
    pub fn validate(&self) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();
        let mut check = |ok: bool, path: &str, message: &str| {
            if !ok {
                issues.push(ConfigIssue::new(path, message));
            }
        };

        check(
            self.config_version == CONFIG_VERSION,
            "config_version",
            &format!(
                "versión no soportada: {} (se espera {CONFIG_VERSION})",
                self.config_version
            ),
        );

        let r = &self.regulator;
        check(
            is_fraction(r.budget_fraction),
            "regulator.budget_fraction",
            "debe estar en (0, 0.5]",
        );
        check(
            r.up_margin > 0.0 && r.up_margin <= 1.0,
            "regulator.up_margin",
            "debe estar en (0, 1]",
        );
        check(
            r.vram_floor_mb <= MAX_VRAM_MB,
            "regulator.vram_floor_mb",
            "valor absurdo (¿MB?)",
        );
        check(
            r.vram_margin_mb <= MAX_VRAM_MB,
            "regulator.vram_margin_mb",
            "valor absurdo (¿MB?)",
        );
        check(
            r.panic_frames >= 1,
            "regulator.panic_frames",
            "debe ser al menos 1",
        );
        check(
            r.settle_frames >= 1,
            "regulator.settle_frames",
            "debe ser al menos 1",
        );
        check(
            r.max_backoff >= 1,
            "regulator.max_backoff",
            "debe ser al menos 1",
        );

        check(
            !self.telemetry.mark_key.trim().is_empty(),
            "telemetry.mark_key",
            "no puede estar vacía",
        );
        check(
            self.telemetry
                .dir
                .as_deref()
                .is_none_or(|d| !d.trim().is_empty()),
            "telemetry.dir",
            "si se indica, no puede estar vacío",
        );

        for (name, profile) in &self.profiles {
            let base = format!("profiles.{name}");
            check(
                !name.trim().is_empty(),
                &base,
                "el nombre del juego no puede estar vacío",
            );
            check(
                profile.budget_fraction.is_none_or(is_fraction),
                &format!("{base}.budget_fraction"),
                "debe estar en (0, 0.5]",
            );
        }
        issues
    }
}

fn is_fraction(value: f64) -> bool {
    value.is_finite() && value > 0.0 && value <= 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Profile;

    fn paths(cfg: &Config) -> Vec<String> {
        cfg.validate().into_iter().map(|i| i.path).collect()
    }

    #[test]
    fn default_config_is_valid() {
        assert!(Config::default().validate().is_empty());
    }

    #[test]
    fn rejects_out_of_range_regulator_values() {
        let mut cfg = Config::default();
        cfg.regulator.budget_fraction = 0.9;
        cfg.regulator.up_margin = 0.0;
        cfg.regulator.panic_frames = 0;
        cfg.regulator.settle_frames = 0;
        cfg.regulator.max_backoff = 0;
        cfg.regulator.vram_floor_mb = u32::MAX;
        let found = paths(&cfg);
        for expected in [
            "regulator.budget_fraction",
            "regulator.up_margin",
            "regulator.panic_frames",
            "regulator.settle_frames",
            "regulator.max_backoff",
            "regulator.vram_floor_mb",
        ] {
            assert!(
                found.iter().any(|p| p == expected),
                "falta {expected} en {found:?}"
            );
        }
    }

    #[test]
    fn rejects_nan_and_negative_fractions() {
        let mut cfg = Config::default();
        cfg.regulator.budget_fraction = f64::NAN;
        assert_eq!(paths(&cfg), ["regulator.budget_fraction"]);
        cfg.regulator.budget_fraction = -0.1;
        assert_eq!(paths(&cfg), ["regulator.budget_fraction"]);
    }

    #[test]
    fn rejects_bad_profiles_and_telemetry() {
        let mut cfg = Config::default();
        cfg.profiles.insert(
            "Control".into(),
            Profile {
                budget_fraction: Some(2.0),
                ..Profile::default()
            },
        );
        cfg.profiles.insert("  ".into(), Profile::default());
        cfg.telemetry.mark_key = " ".into();
        cfg.telemetry.dir = Some(String::new());
        let found = paths(&cfg);
        assert!(found.contains(&"profiles.Control.budget_fraction".to_owned()));
        assert!(found.contains(&"profiles.  ".to_owned()));
        assert!(found.contains(&"telemetry.mark_key".to_owned()));
        assert!(found.contains(&"telemetry.dir".to_owned()));
    }

    #[test]
    fn rejects_an_unknown_config_version() {
        let cfg = Config {
            config_version: 99,
            ..Config::default()
        };
        assert_eq!(paths(&cfg), ["config_version"]);
    }
}
