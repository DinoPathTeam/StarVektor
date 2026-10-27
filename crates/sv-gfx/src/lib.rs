//! Graphics backend contract.
//!
//! This crate only defines **what** StarVektor needs from the GPU; **how** it is
//! obtained lives in separate crates (for example, a future `sv-gfx-vulkan` with
//! `ash`), so adding or changing a graphics API touches neither the core nor
//! this contract.
//!
//! Pixels do not pass through here: layers exchange identifiers
//! (`FrameHandle`) and each backend resolves the real resource.

use std::fmt;

use sv_types::{MemoryBudget, PassTimings};

/// Graphics backend error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GfxError {
    /// The required API or extension is not available.
    Unavailable(String),
    /// The device failed (context loss, out of memory, etc.).
    Device(String),
}

impl fmt::Display for GfxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GfxError::Unavailable(what) => write!(f, "unavailable: {what}"),
            GfxError::Device(what) => write!(f, "device error: {what}"),
        }
    }
}

impl std::error::Error for GfxError {}

/// What StarVektor needs to know about the GPU on every frame.
pub trait GraphicsBackend {
    /// Short name for the log (`"vulkan"`, `"null"`).
    fn name(&self) -> &'static str;

    /// Used and free video memory (with Vulkan, `VK_EXT_memory_budget`).
    ///
    /// # Errors
    /// If the backend cannot query memory.
    fn memory_budget(&mut self) -> Result<MemoryBudget, GfxError>;

    /// Milliseconds per pass of the last processed frame, measured on the
    /// GPU with timestamp queries (not with the program's clock).
    ///
    /// # Errors
    /// If the backend cannot read the counters.
    fn last_pass_timings(&mut self) -> Result<PassTimings, GfxError>;
}

/// GPU-less backend: returns values set by whoever uses it.
///
/// It is used to test the rest of the system (regulator, telemetry, TUI) without
/// hardware and for the executable's `selftest`.
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
