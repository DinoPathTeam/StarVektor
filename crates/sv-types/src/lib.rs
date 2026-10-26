//! Contratos compartidos entre las capas de StarVektor.
//!
//! Este crate es lo único que todos los demás conocen, así que se mantiene
//! pequeño y estable: solo tipos de datos sin lógica de negocio. Si un tipo lo
//! usa un solo crate, vive en ese crate y no aquí.

use core::fmt;

use serde::{Deserialize, Serialize};

/// Nivel del regulador (ver D5 y la tabla de niveles del documento maestro).
///
/// En el log y en la configuración se escribe como entero (0 a 3).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(try_from = "u8", into = "u8")]
pub enum Level {
    /// Apagado: el fotograma pasa tal cual.
    #[default]
    Off = 0,
    /// Todo clásico: flujo óptico, reglas de HUD, máscara heurística.
    Classic = 1,
    /// Clásico más una red mínima para máscara de errores y HUD.
    MiniNet = 2,
    /// Red completa de percepción.
    FullNet = 3,
}

impl Level {
    pub const ALL: [Level; 4] = [Level::Off, Level::Classic, Level::MiniNet, Level::FullNet];

    /// Un nivel por encima, o el mismo si ya es el máximo.
    #[must_use]
    pub fn up(self) -> Level {
        match self {
            Level::Off => Level::Classic,
            Level::Classic => Level::MiniNet,
            Level::MiniNet | Level::FullNet => Level::FullNet,
        }
    }

    /// Un nivel por debajo, o el mismo si ya es `Off`.
    #[must_use]
    pub fn down(self) -> Level {
        match self {
            Level::Off | Level::Classic => Level::Off,
            Level::MiniNet => Level::Classic,
            Level::FullNet => Level::MiniNet,
        }
    }
}

/// Error al convertir un entero que no es un nivel válido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidLevel(pub u8);

impl fmt::Display for InvalidLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nivel inválido: {} (se esperaba 0 a 3)", self.0)
    }
}

impl std::error::Error for InvalidLevel {}

impl TryFrom<u8> for Level {
    type Error = InvalidLevel;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Level::Off),
            1 => Ok(Level::Classic),
            2 => Ok(Level::MiniNet),
            3 => Ok(Level::FullNet),
            other => Err(InvalidLevel(other)),
        }
    }
}

impl From<Level> for u8 {
    fn from(level: Level) -> u8 {
        level as u8
    }
}

/// Cómo se obtuvo el fotograma que se presenta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameKind {
    /// Lo renderizó el juego.
    Rendered,
    /// Lo generó StarVektor.
    Generated,
    /// Se repitió el fotograma anterior (la máscara de confianza no se fió).
    Repeated,
}

/// Configuración de OptiScaler detectada en la cadena (D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptiScalerMode {
    #[default]
    None,
    /// Solo upscaling.
    Upscale,
    /// Solo generación de fotogramas.
    Fg,
    /// Upscaling y generación de fotogramas.
    Both,
}

/// Qué datos del juego están disponibles para StarVektor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationLevel {
    /// El motor entrega vectores, profundidad y capas.
    Engine,
    /// Los datos llegan a través del upscaler del juego (con o sin OptiScaler).
    Upscaler,
    /// Solo se ve la imagen final.
    ImageOnly,
}

/// Resolución en píxeles. Se escribe como `[ancho, alto]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution(pub u32, pub u32);

impl Resolution {
    #[must_use]
    pub fn width(self) -> u32 {
        self.0
    }

    #[must_use]
    pub fn height(self) -> u32 {
        self.1
    }
}

/// Milisegundos por pasada de StarVektor en un fotograma.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct PassTimings {
    pub capture: f32,
    pub motion: f32,
    pub reproject: f32,
    pub mask: f32,
    pub present: f32,
}

impl PassTimings {
    /// Costo total de StarVektor en el fotograma, en ms.
    #[must_use]
    pub fn total(&self) -> f32 {
        self.capture + self.motion + self.reproject + self.mask + self.present
    }
}

/// Memoria de vídeo disponible, en MB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MemoryBudget {
    pub used_mb: u32,
    pub free_mb: u32,
}

/// Identificador opaco de un fotograma capturado.
///
/// Los píxeles nunca viajan entre capas: se pasan identificadores y cada
/// backend resuelve el recurso real (captura sin copia).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameHandle(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_roundtrips_as_integer() {
        for level in Level::ALL {
            let n: u8 = level.into();
            assert_eq!(Level::try_from(n), Ok(level));
        }
        assert_eq!(Level::try_from(4), Err(InvalidLevel(4)));
    }

    #[test]
    fn level_up_down_saturate() {
        assert_eq!(Level::FullNet.up(), Level::FullNet);
        assert_eq!(Level::Off.down(), Level::Off);
        assert_eq!(Level::Off.up(), Level::Classic);
        assert_eq!(Level::Classic.down(), Level::Off);
        assert_eq!(Level::FullNet.down(), Level::MiniNet);
    }

    #[test]
    fn level_orders_by_capability() {
        assert!(Level::Off < Level::Classic);
        assert!(Level::MiniNet < Level::FullNet);
    }

    #[test]
    fn pass_timings_total_sums_passes() {
        let t = PassTimings {
            capture: 0.3,
            motion: 1.0,
            reproject: 0.5,
            mask: 0.4,
            present: 0.2,
        };
        assert!((t.total() - 2.4).abs() < 1e-6);
    }
}
