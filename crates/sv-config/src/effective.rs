//! Resolution of the effective configuration for a game.

use sv_types::Level;

use crate::model::{Config, HookMode};

/// What is actually applied to a game: global plus the profile, if one exists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effective {
    pub enabled: bool,
    pub max_level: Level,
    pub budget_fraction: f64,
    pub hook: HookMode,
}

impl Config {
    /// Combines the global settings with the profile of `game`.
    ///
    /// Lookup is by exact name. Without a profile, the global values are used.
    #[must_use]
    pub fn effective(&self, game: &str) -> Effective {
        let profile = self.profiles.get(game);
        Effective {
            enabled: self.general.enabled && profile.and_then(|p| p.enabled).unwrap_or(true),
            max_level: profile
                .and_then(|p| p.max_level)
                .unwrap_or(self.regulator.max_level),
            budget_fraction: profile
                .and_then(|p| p.budget_fraction)
                .unwrap_or(self.regulator.budget_fraction),
            hook: profile.and_then(|p| p.hook).unwrap_or(self.hook.mode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Profile;

    #[test]
    fn without_profile_uses_globals() {
        let cfg = Config::default();
        let e = cfg.effective("anything");
        assert!(e.enabled);
        assert_eq!(e.max_level, Level::FullNet);
        assert!((e.budget_fraction - 0.12).abs() < f64::EPSILON);
        assert_eq!(e.hook, HookMode::Auto);
    }

    #[test]
    fn profile_overrides_only_what_it_sets() {
        let mut cfg = Config::default();
        cfg.profiles.insert(
            "Valorant".into(),
            Profile {
                enabled: Some(false),
                max_level: Some(Level::Off),
                ..Profile::default()
            },
        );
        cfg.profiles.insert(
            "Control".into(),
            Profile {
                budget_fraction: Some(0.1),
                ..Profile::default()
            },
        );

        let valorant = cfg.effective("Valorant");
        assert!(!valorant.enabled);
        assert_eq!(valorant.max_level, Level::Off);

        let control = cfg.effective("Control");
        assert!(control.enabled);
        assert_eq!(control.max_level, Level::FullNet, "hereda lo global");
        assert!((control.budget_fraction - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn global_switch_wins_over_a_profile() {
        let mut cfg = Config::default();
        cfg.general.enabled = false;
        cfg.profiles.insert(
            "Control".into(),
            Profile {
                enabled: Some(true),
                ..Profile::default()
            },
        );
        assert!(!cfg.effective("Control").enabled);
    }

    #[test]
    fn game_names_are_matched_exactly() {
        let mut cfg = Config::default();
        cfg.profiles.insert(
            "Control".into(),
            Profile {
                max_level: Some(Level::Classic),
                ..Profile::default()
            },
        );
        assert_eq!(cfg.effective("control").max_level, Level::FullNet);
    }
}
