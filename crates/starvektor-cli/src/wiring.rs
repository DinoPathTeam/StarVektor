//! Translation between layers.
//!
//! `sv-config` and `sv-core` do not know each other. This is the only seam: if one
//! of the two changes, only this file needs review.

use sv_config::{Config, Effective};
use sv_core::regulator::RegulatorConfig;
use sv_types::Level;

/// Regulator parameters for a game, from the configuration.
///
/// If StarVektor is disabled for that game, the cap is level 0.
#[must_use]
pub fn regulator_config(config: &Config, effective: &Effective) -> RegulatorConfig {
    let r = &config.regulator;
    RegulatorConfig {
        max_level: if effective.enabled {
            effective.max_level
        } else {
            Level::Off
        },
        budget_fraction: effective.budget_fraction,
        vram_floor_mb: r.vram_floor_mb,
        vram_margin_mb: r.vram_margin_mb,
        up_margin: r.up_margin,
        panic_frames: r.panic_frames,
        settle_frames: r.settle_frames,
        max_backoff: r.max_backoff,
        ..RegulatorConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use sv_config::model::Profile;

    use super::*;

    #[test]
    fn maps_every_regulator_setting() {
        let mut config = Config::default();
        config.regulator.vram_floor_mb = 512;
        config.regulator.settle_frames = 30;
        config.regulator.panic_frames = 5;
        let effective = config.effective("game");
        let rc = regulator_config(&config, &effective);
        assert_eq!(rc.vram_floor_mb, 512);
        assert_eq!(rc.settle_frames, 30);
        assert_eq!(rc.panic_frames, 5);
        assert_eq!(rc.max_level, Level::FullNet);
        assert!((rc.budget_fraction - 0.12).abs() < f64::EPSILON);
    }

    #[test]
    fn a_profile_changes_the_cap_and_budget() {
        let mut config = Config::default();
        config.profiles.insert(
            "Control".into(),
            Profile {
                max_level: Some(Level::Classic),
                budget_fraction: Some(0.1),
                ..Profile::default()
            },
        );
        let rc = regulator_config(&config, &config.effective("Control"));
        assert_eq!(rc.max_level, Level::Classic);
        assert!((rc.budget_fraction - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn disabled_means_level_zero_cap() {
        let mut config = Config::default();
        config.profiles.insert(
            "Valorant".into(),
            Profile {
                enabled: Some(false),
                ..Profile::default()
            },
        );
        let rc = regulator_config(&config, &config.effective("Valorant"));
        assert_eq!(rc.max_level, Level::Off);
    }
}
