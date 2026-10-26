//! Tipos de los registros del log. Fuente de verdad del esquema.
//!
//! Reglas de evolución (ver `docs/log-schema.md`):
//! - Añadir un campo opcional es compatible: sube la versión menor (`1.1`).
//! - Quitar, renombrar o cambiar el tipo de un campo es incompatible: sube la mayor.
//! - Los lectores aceptan cualquier versión menor de la mayor que conocen.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sv_types::{FrameKind, IntegrationLevel, Level, OptiScalerMode, PassTimings, Resolution};

/// Versión del esquema que escribe esta versión de StarVektor.
pub const SCHEMA_VERSION: &str = "1.0";

/// Versión mayor que sabe leer el validador.
pub const SUPPORTED_MAJOR: &str = "1";

/// Una línea del archivo: la versión del esquema más el registro.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub schema: String,
    #[serde(flatten)]
    pub record: Record,
}

impl Line {
    #[must_use]
    pub fn new(record: Record) -> Self {
        Self {
            schema: SCHEMA_VERSION.to_owned(),
            record,
        }
    }
}

/// Registro del log. El campo `type` lo distingue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Record {
    /// Encuadrado: aparece una vez por archivo y es mucho mayor que `Frame`,
    /// que es el que viaja por la cola de escritura en cada fotograma.
    Run(Box<RunRecord>),
    Frame(FrameRecord),
    Event(EventRecord),
    Summary(SummaryRecord),
}

/// Una vez, al inicio. Debe ser la primera línea.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    /// ISO 8601 en UTC, por ejemplo `2026-10-26T17:00:00Z`.
    pub started_at: String,
    pub game: String,
    pub game_version: String,
    /// Escena fija de prueba (exploración, combate, luz cambiante).
    pub scene: String,
    pub gpu: String,
    pub driver: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesa: Option<String>,
    pub resolution: Resolution,
    pub optiscaler: OptiScalerMode,
    pub integration_level: IntegrationLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_hash: Option<String>,
    /// Semilla para repetir la prueba exacta.
    pub seed: u64,
    /// Configuración completa vigente (copia de la TOML resuelta).
    pub config: serde_json::Value,
}

/// Uno por fotograma presentado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameRecord {
    /// Índice del fotograma presentado, estrictamente creciente.
    pub n: u64,
    pub t_render_ms: f64,
    pub t_present_ms: f64,
    pub kind: FrameKind,
    pub pass_ms: PassTimings,
    pub vram_used_mb: u32,
    pub vram_free_mb: u32,
    pub level: Level,
    /// Porcentaje de pantalla donde la máscara cayó a repetir (0 a 100).
    pub repeat_pct: f32,
    /// Solo si hay verdad disponible (0 a 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssim: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psnr: Option<f32>,
}

/// Tipo de evento puntual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    LevelChange,
    SceneCut,
    FrameSpike,
    VramLow,
    /// Marca manual del usuario: "aquí vi un artefacto".
    UserMark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    /// Fotograma asociado.
    pub n: u64,
    pub event: EventKind,
    pub detail: serde_json::Value,
}

/// Uno por escena, al cerrar. Debe ser la última línea.
///
/// `p1_ms` y `p01_ms` son los tiempos de fotograma en el percentil 99 y 99.9
/// (el 1 % y el 0.1 % más lentos). Se calculan con el método del rango más
/// cercano sobre los intervalos entre fotogramas presentados.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryRecord {
    pub fps_rendered_avg: f64,
    pub fps_presented_avg: f64,
    pub p1_ms: f64,
    pub p01_ms: f64,
    /// Latencia añadida estimada, en ms. Ausente si no se pudo estimar
    /// (por ejemplo, en el `selftest` sintético): nunca se rellena con un 0 falso.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_est_ms: Option<f64>,
    pub overhead_ms_avg: f64,
    /// Porcentaje de fotogramas presentados en cada nivel; claves `"0"` a `"3"`.
    pub level_time_pct: BTreeMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn line_roundtrips_for_every_record_type() {
        let records = [
            Record::Run(Box::new(fixtures::run())),
            Record::Frame(fixtures::frame(7, 100.0)),
            Record::Event(EventRecord {
                n: 7,
                event: EventKind::UserMark,
                detail: serde_json::json!({"note": "halo en el HUD"}),
            }),
            Record::Summary(fixtures::summary()),
        ];
        for record in records {
            let line = Line::new(record);
            let text = serde_json::to_string(&line).unwrap();
            let back: Line = serde_json::from_str(&text).unwrap();
            assert_eq!(back, line, "{text}");
        }
    }

    #[test]
    fn wire_format_matches_the_documented_shape() {
        let line = Line::new(Record::Frame(fixtures::frame(3, 50.0)));
        let value: serde_json::Value = serde_json::to_value(&line).unwrap();
        assert_eq!(value["schema"], "1.0");
        assert_eq!(value["type"], "frame");
        assert_eq!(value["n"], 3);
        assert_eq!(value["kind"], "rendered");
        assert_eq!(value["level"], 1, "el nivel se escribe como entero");
        assert!(value["pass_ms"]["motion"].is_number());
        assert!(
            value.get("ssim").is_none(),
            "los opcionales vacíos no se escriben"
        );
    }

    #[test]
    fn run_resolution_is_an_array() {
        let line = Line::new(Record::Run(Box::new(fixtures::run())));
        let value = serde_json::to_value(&line).unwrap();
        assert_eq!(value["resolution"], serde_json::json!([2560, 1440]));
        assert_eq!(value["optiscaler"], "upscale");
        assert_eq!(value["integration_level"], "image_only");
    }
}
