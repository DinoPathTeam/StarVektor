//! Contrato del backend gráfico.
//!
//! Este crate solo define **qué** necesita StarVektor de la GPU; **cómo** se
//! obtiene vive en crates aparte (por ejemplo, un futuro `sv-gfx-vulkan` con
//! `ash`), de modo que añadir o cambiar una API gráfica no toca el núcleo ni
//! este contrato.
//!
//! Los píxeles no pasan por aquí: las capas se intercambian identificadores
//! (`FrameHandle`) y cada backend resuelve el recurso real.

use std::fmt;

use sv_types::{MemoryBudget, PassTimings};

/// Error de un backend gráfico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GfxError {
    /// La API o la extensión necesaria no está disponible.
    Unavailable(String),
    /// Falló el dispositivo (pérdida de contexto, memoria agotada, etc.).
    Device(String),
}

impl fmt::Display for GfxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GfxError::Unavailable(what) => write!(f, "no disponible: {what}"),
            GfxError::Device(what) => write!(f, "error del dispositivo: {what}"),
        }
    }
}

impl std::error::Error for GfxError {}

/// Lo que StarVektor necesita saber de la GPU en cada fotograma.
pub trait GraphicsBackend {
    /// Nombre corto para el log (`"vulkan"`, `"null"`).
    fn name(&self) -> &'static str;

    /// Memoria de vídeo usada y libre (con Vulkan, `VK_EXT_memory_budget`).
    ///
    /// # Errors
    /// Si el backend no puede consultar la memoria.
    fn memory_budget(&mut self) -> Result<MemoryBudget, GfxError>;

    /// Milisegundos por pasada del último fotograma procesado, medidos en la
    /// GPU con timestamp queries (no con el reloj del programa).
    ///
    /// # Errors
    /// Si el backend no puede leer los contadores.
    fn last_pass_timings(&mut self) -> Result<PassTimings, GfxError>;
}

/// Backend sin GPU: devuelve valores fijados por quien lo usa.
///
/// Sirve para probar el resto del sistema (regulador, telemetría, TUI) sin
/// hardware y para el `selftest` del ejecutable.
#[derive(Debug, Clone, Default)]
pub struct NullBackend {
    memory: MemoryBudget,
    timings: PassTimings,
}

impl NullBackend {
    #[must_use]
    pub fn new(memory: MemoryBudget, timings: PassTimings) -> Self {
        Self { memory, timings }
    }

    pub fn set_memory(&mut self, memory: MemoryBudget) {
        self.memory = memory;
    }

    pub fn set_timings(&mut self, timings: PassTimings) {
        self.timings = timings;
    }
}

impl GraphicsBackend for NullBackend {
    fn name(&self) -> &'static str {
        "null"
    }

    fn memory_budget(&mut self) -> Result<MemoryBudget, GfxError> {
        Ok(self.memory)
    }

    fn last_pass_timings(&mut self) -> Result<PassTimings, GfxError> {
        Ok(self.timings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_backend_returns_what_it_was_given() {
        let mem = MemoryBudget {
            used_mb: 3000,
            free_mb: 5000,
        };
        let t = PassTimings {
            motion: 1.0,
            ..PassTimings::default()
        };
        let mut gfx = NullBackend::new(mem, t);
        assert_eq!(gfx.name(), "null");
        assert_eq!(gfx.memory_budget().unwrap(), mem);
        assert_eq!(gfx.last_pass_timings().unwrap(), t);

        gfx.set_memory(MemoryBudget {
            used_mb: 7800,
            free_mb: 200,
        });
        assert_eq!(gfx.memory_budget().unwrap().free_mb, 200);
    }

    #[test]
    fn works_through_a_trait_object() {
        let mut gfx: Box<dyn GraphicsBackend> = Box::new(NullBackend::default());
        assert!(gfx.last_pass_timings().is_ok());
    }

    #[test]
    fn errors_describe_themselves() {
        assert!(
            GfxError::Unavailable("VK_EXT_memory_budget".into())
                .to_string()
                .contains("VK_EXT_memory_budget")
        );
    }
}
