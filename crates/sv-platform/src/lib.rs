//! Contrato de la capa de plataforma.
//!
//! Es lo único que cambia entre Linux y Windows: captura del fotograma,
//! presentación y lectura de la entrada. Las implementaciones reales viven en
//! crates aparte (por ejemplo, un futuro `sv-platform-linux` con el enganche
//! a la capa Vulkan o a gamescope, según el resultado de la fase 1), así que
//! portar a otro sistema es añadir un crate, no modificar este.
//!
//! - [`Platform`]: el contrato.
//! - [`synthetic`]: una plataforma sin sistema operativo, determinista, para pruebas.

use std::fmt;

use sv_types::{FrameHandle, FrameKind, Resolution};

pub mod synthetic;

/// Fotograma capturado del juego. Solo metadatos: los píxeles no viajan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapturedFrame {
    pub handle: FrameHandle,
    /// Índice del fotograma renderizado por el juego.
    pub index: u64,
    /// Instante en que el juego terminó de renderizarlo, en ms desde el inicio.
    pub render_ts_ms: f64,
    pub resolution: Resolution,
    /// Si trae vectores de movimiento nuevos (señal para detectar la generación de fotogramas).
    pub fresh_motion: bool,
}

/// Petición de presentar un fotograma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresentRequest {
    /// Fotograma de origen (el real, o el último en el que se basa uno generado).
    pub source: FrameHandle,
    pub kind: FrameKind,
    /// Instante objetivo de presentación, en ms desde el inicio.
    pub target_ts_ms: f64,
}

/// Error de la capa de plataforma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// El juego o el compositor se cerró.
    Disconnected,
    Other(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::Disconnected => write!(f, "el juego o el compositor se desconectó"),
            PlatformError::Other(what) => write!(f, "{what}"),
        }
    }
}

impl std::error::Error for PlatformError {}

/// Lo que StarVektor necesita del sistema donde corre.
pub trait Platform {
    /// Nombre corto para el log (`"synthetic"`, `"linux-vulkan-layer"`...).
    fn name(&self) -> &'static str;

    /// Periodo de refresco del monitor, en ms.
    fn refresh_period_ms(&self) -> f64;

    /// Siguiente fotograma del juego. `Ok(None)` cuando termina la sesión.
    ///
    /// # Errors
    /// Si la captura falla.
    fn next_frame(&mut self) -> Result<Option<CapturedFrame>, PlatformError>;

    /// Presenta un fotograma y devuelve el instante real de presentación, en ms.
    ///
    /// # Errors
    /// Si la presentación falla.
    fn present(&mut self, request: &PresentRequest) -> Result<f64, PlatformError>;

    /// `true` si el usuario pulsó la tecla de marca desde la última consulta.
    fn poll_user_mark(&mut self) -> bool;
}
