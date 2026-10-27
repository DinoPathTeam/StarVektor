//! Configuration model and default values.
//!
//! The default numeric values are the initial hypotheses from the prototype
//! sheet; they are tuned with the Phase 1 measurements.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sv_types::Level;

/// Version of the configuration file format.
pub const CONFIG_VERSION: u32 = 1;

/// How StarVektor hooks into the game (decision D2, pending measurement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookMode {
    /// Picks the option that won the measurement (Phase 1).
    #[default]
    Auto,
    /// Implicit Vulkan layer.
    Vulkan,
    /// Inside gamescope.
    Gamescope,
}

impl HookMode {
    pub const ALL: [HookMode; 3] = [HookMode::Auto, HookMode::Vulkan, HookMode::Gamescope];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            HookMode::Auto => "auto",
            HookMode::Vulkan => "vulkan",
            HookMode::Gamescope => "gamescope",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub config_version: u32,
    pub general: General,
    pub regulator: RegulatorSettings,
    pub hook: HookSettings,
    pub telemetry: TelemetrySettings,
    /// Per-game settings; the key is the game name (see `Config::effective`).
    pub profiles: BTreeMap<String, Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_version: CONFIG_VERSION,
            general: General::default(),
            regulator: RegulatorSettings::default(),
            hook: HookSettings::default(),
            telemetry: TelemetrySettings::default(),
            profiles: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    /// Master switch: when off, StarVektor lets everything through.
    pub enabled: bool,
}

impl Default for General {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RegulatorSettings {
    /// Maximum allowed level (0 to 3).
    pub max_level: Level,
    /// Fraction of the base frame time StarVektor may spend.
    pub budget_fraction: f64,
    /// Minimum free VRAM, in MB.
    pub vram_floor_mb: u32,
    /// Extra free VRAM above the minimum to be able to rise a level, in MB.
    pub vram_margin_mb: u32,
    /// Headroom required to rise (fraction of the budget).
    pub up_margin: f64,
    /// Consecutive bad samples before dropping a level.
    pub panic_frames: u32,
    /// Consecutive good samples before rising a level.
    pub settle_frames: u32,
    /// Maximum wait multiplier after forced drops.
    pub max_backoff: u32,
}

impl Default for RegulatorSettings {
    fn default() -> Self {
        Self {
            max_level: Level::FullNet,
            budget_fraction: 0.12,
            vram_floor_mb: 400,
            vram_margin_mb: 200,
            up_margin: 0.6,
            panic_frames: 3,
            settle_frames: 120,
            max_backoff: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HookSettings {
    pub mode: HookMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TelemetrySettings {
    /// Write the JSONL log of each run.
    pub enabled: bool,
    /// Log directory; if missing, `$XDG_STATE_HOME/starvektor/logs`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// Show the minimal overlay (frame time, level, marks).
    pub overlay: bool,
    /// Key that records a `user_mark` ("I saw an artifact here").
    pub mark_key: String,
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            dir: None,
            overlay: false,
            mark_key: "F9".to_owned(),
        }
    }
}

/// Overrides for one game. Anything not set inherits the global value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_fraction: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook: Option<HookMode>,
}
