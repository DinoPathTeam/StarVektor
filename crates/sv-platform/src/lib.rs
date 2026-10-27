//! Platform layer contract.
//!
//! It is the only thing that changes between Linux and Windows: frame capture,
//! presentation and input reading. Real implementations live in separate
//! crates (for example, a future `sv-platform-linux` with the hook
//! into the Vulkan layer or gamescope, depending on the Phase 1 result), so
//! porting to another system means adding a crate, not modifying this one.
//!
//! - [`Platform`]: the contract.
//! - [`synthetic`]: a deterministic platform with no operating system, for tests.

use std::fmt;

use sv_types::{FrameHandle, FrameKind, Resolution};

pub mod synthetic;

/// Frame captured from the game. Metadata only: pixels do not travel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapturedFrame {
    pub handle: FrameHandle,
    /// Index of the frame rendered by the game.
    pub index: u64,
    /// Moment the game finished rendering it, in ms since the start.
    pub render_ts_ms: f64,
    pub resolution: Resolution,
    /// Whether it carries fresh motion vectors (signal to detect frame generation).
    pub fresh_motion: bool,
}

/// Request to present a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresentRequest {
    /// Source frame (the real one, or the last one a generated frame is based on).
    pub source: FrameHandle,
    pub kind: FrameKind,
    /// Target presentation time, in ms since the start.
    pub target_ts_ms: f64,
}

/// Platform layer error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// The game or the compositor closed.
    Disconnected,
    Other(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::Disconnected => write!(f, "the game or the compositor disconnected"),
            PlatformError::Other(what) => write!(f, "{what}"),
        }
    }
}

impl std::error::Error for PlatformError {}

/// What StarVektor needs from the system it runs on.
pub trait Platform {
    /// Short name for the log (`"synthetic"`, `"linux-vulkan-layer"`...).
    fn name(&self) -> &'static str;

    /// Monitor refresh period, in ms.
    fn refresh_period_ms(&self) -> f64;

    /// Next frame of the game. `Ok(None)` when the session ends.
    ///
    /// # Errors
    /// If capture fails.
    fn next_frame(&mut self) -> Result<Option<CapturedFrame>, PlatformError>;

    /// Presents a frame and returns the actual presentation time, in ms.
    ///
    /// # Errors
    /// If presentation fails.
    fn present(&mut self, request: &PresentRequest) -> Result<f64, PlatformError>;

    /// `true` if the user pressed the mark key since the last query.
    fn poll_user_mark(&mut self) -> bool;
}
