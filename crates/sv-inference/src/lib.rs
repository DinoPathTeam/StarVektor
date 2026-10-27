//! Inference contract and model selection.
//!
//! As with the graphics side, only the interface lives here. Concrete
//! backends (AMD or NVIDIA AI units, generic shaders for older GPUs,
//! ONNX) will go in separate crates, and all models are described with
//! [`ModelSpec`]. Selection is a pure function: it does not touch the GPU.

use std::fmt;

use sv_types::Level;

/// Family of model sizes (obtained by distillation from the large one).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelTier {
    Minimal,
    Medium,
    Large,
}

impl ModelTier {
    /// Regulator level at which this size is used.
    #[must_use]
    pub fn level(self) -> Level {
        match self {
            ModelTier::Minimal => Level::MiniNet,
            ModelTier::Medium | ModelTier::Large => Level::FullNet,
        }
    }
}

/// Description of an available model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSpec {
    pub id: String,
    /// Content hash (recorded in the log: `run.model_hash`).
    pub hash: String,
    pub tier: ModelTier,
    /// Inference time measured on the current hardware, in ms.
    pub est_ms: f32,
    /// VRAM it occupies with its activations, in MB.
    pub est_vram_mb: u32,
}

/// Picks the largest model that fits the budget.
///
/// `budget_ms` is the time left for inference and `vram_available_mb` is the
/// memory that can be dedicated to it (the caller has already subtracted the
/// safety threshold). At equal size the faster one wins. `None` if none fits:
/// the regulator must stay at level 1 (classic).
#[must_use]
pub fn select_model(
    models: &[ModelSpec],
    budget_ms: f32,
    vram_available_mb: u32,
) -> Option<&ModelSpec> {
    models
        .iter()
        .filter(|m| m.est_ms <= budget_ms && m.est_vram_mb <= vram_available_mb)
        .max_by(|a, b| a.tier.cmp(&b.tier).then(b.est_ms.total_cmp(&a.est_ms)))
}

/// Inference backend error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceError {
    Unsupported(String),
    LoadFailed(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferenceError::Unsupported(what) => write!(f, "unsupported: {what}"),
            InferenceError::LoadFailed(what) => write!(f, "could not load the model: {what}"),
        }
    }
}

impl std::error::Error for InferenceError {}

/// What is asked of an inference backend.
pub trait InferenceBackend {
    fn name(&self) -> &'static str;

    /// Whether the hardware has accelerated AI units (tensor cores, WMMA, NPU).
    fn has_ai_acceleration(&self) -> bool;

    /// Loads a model, unloading the previous one.
    ///
    /// # Errors
    /// If the model is not compatible or loading fails.
    fn load(&mut self, model: &ModelSpec) -> Result<(), InferenceError>;

    /// Releases the loaded model.
    fn unload(&mut self);

    /// Currently loaded model, if any.
    fn loaded(&self) -> Option<&ModelSpec>;
}

/// Backend with no real inference: it only remembers which model was loaded.
#[derive(Debug, Default)]
pub struct NullInference {
    loaded: Option<ModelSpec>,
}

impl InferenceBackend for NullInference {
    fn name(&self) -> &'static str {
        "null"
    }

    fn has_ai_acceleration(&self) -> bool {
        false
    }

    fn load(&mut self, model: &ModelSpec) -> Result<(), InferenceError> {
        self.loaded = Some(model.clone());
        Ok(())
    }

    fn unload(&mut self) {
        self.loaded = None;
    }

    fn loaded(&self) -> Option<&ModelSpec> {
        self.loaded.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str, tier: ModelTier, est_ms: f32, est_vram_mb: u32) -> ModelSpec {
        ModelSpec {
            id: id.into(),
            hash: format!("hash-{id}"),
            tier,
            est_ms,
            est_vram_mb,
        }
    }

    fn family() -> Vec<ModelSpec> {
        vec![
            spec("min", ModelTier::Minimal, 0.4, 20),
            spec("med", ModelTier::Medium, 1.2, 80),
            spec("big", ModelTier::Large, 2.6, 200),
        ]
    }

    #[test]
    fn picks_the_largest_that_fits_both_budgets() {
        let models = family();
        assert_eq!(select_model(&models, 3.0, 300).unwrap().id, "big");
        assert_eq!(
            select_model(&models, 2.0, 300).unwrap().id,
            "med",
            "the large one does not fit in time"
        );
        assert_eq!(
            select_model(&models, 3.0, 100).unwrap().id,
            "med",
            "the large one does not fit in memory"
        );
        assert_eq!(select_model(&models, 0.5, 300).unwrap().id, "min");
    }

    #[test]
    fn returns_none_when_nothing_fits() {
        let models = family();
        assert!(select_model(&models, 0.1, 300).is_none());
        assert!(select_model(&models, 3.0, 10).is_none());
        assert!(select_model(&[], 3.0, 300).is_none());
    }

    #[test]
    fn ties_go_to_the_faster_model() {
        let models = vec![
            spec("lento", ModelTier::Medium, 1.5, 80),
            spec("fast", ModelTier::Medium, 1.0, 80),
        ];
        assert_eq!(select_model(&models, 3.0, 300).unwrap().id, "fast");
    }

    #[test]
    fn tiers_map_to_regulator_levels() {
        assert_eq!(ModelTier::Minimal.level(), Level::MiniNet);
        assert_eq!(ModelTier::Large.level(), Level::FullNet);
    }

    #[test]
    fn null_backend_tracks_the_loaded_model() {
        let mut backend = NullInference::default();
        assert!(backend.loaded().is_none());
        let m = spec("min", ModelTier::Minimal, 0.4, 20);
        backend.load(&m).unwrap();
        assert_eq!(backend.loaded().unwrap().id, "min");
        assert!(!backend.has_ai_acceleration());
        backend.unload();
        assert!(backend.loaded().is_none());
    }
}
