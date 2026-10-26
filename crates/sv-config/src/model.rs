//! Modelo de la configuración y valores por defecto.
//!
//! Los valores numéricos por defecto son las hipótesis iniciales de la hoja de
//! prototipo; se ajustan con las mediciones de la fase 1.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sv_types::Level;

/// Versión del formato del archivo de configuración.
pub const CONFIG_VERSION: u32 = 1;

/// Cómo se engancha StarVektor al juego (decisión D2, pendiente de medición).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookMode {
    /// Elige la opción que haya ganado la medición (fase 1).
    #[default]
    Auto,
    /// Capa Vulkan implícita.
    Vulkan,
    /// Dentro de gamescope.
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
    /// Ajustes por juego; la clave es el nombre del juego (ver `Config::effective`).
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
    /// Interruptor general: apagado, StarVektor deja pasar todo.
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
    /// Nivel máximo permitido (0 a 3).
    pub max_level: Level,
    /// Fracción del tiempo base que StarVektor puede gastar.
    pub budget_fraction: f64,
    /// VRAM libre mínima, en MB.
    pub vram_floor_mb: u32,
    /// VRAM libre extra sobre el mínimo para poder subir de nivel, en MB.
    pub vram_margin_mb: u32,
    /// Holgura exigida para subir (fracción del presupuesto).
    pub up_margin: f64,
    /// Muestras malas seguidas antes de bajar de nivel.
    pub panic_frames: u32,
    /// Muestras buenas seguidas antes de subir de nivel.
    pub settle_frames: u32,
    /// Multiplicador máximo de espera tras bajadas forzadas.
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
    /// Escribir el log JSONL de cada ejecución.
    pub enabled: bool,
    /// Directorio de logs; si falta, `$XDG_STATE_HOME/starvektor/logs`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// Mostrar el overlay mínimo (frame time, nivel, marcas).
    pub overlay: bool,
    /// Tecla que registra un `user_mark` ("aquí vi un artefacto").
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

/// Sobrescrituras para un juego. Lo que no se indica hereda lo global.
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
