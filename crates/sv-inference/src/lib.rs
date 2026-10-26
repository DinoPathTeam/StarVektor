//! Contrato de inferencia y selección de modelo.
//!
//! Igual que con la parte gráfica, aquí solo vive la interfaz. Los backends
//! concretos (unidades de IA de AMD o NVIDIA, shaders genéricos para GPUs
//! antiguas, ONNX) irán en crates aparte, y todos los modelos se describen con
//! [`ModelSpec`]. La selección es una función pura: no toca la GPU.

use std::fmt;

use sv_types::Level;

/// Familia de tamaños de modelo (se obtienen por destilación del grande).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelTier {
    Minimal,
    Medium,
    Large,
}

impl ModelTier {
    /// Nivel del regulador en el que se usa este tamaño.
    #[must_use]
    pub fn level(self) -> Level {
        match self {
            ModelTier::Minimal => Level::MiniNet,
            ModelTier::Medium | ModelTier::Large => Level::FullNet,
        }
    }
}

/// Descripción de un modelo disponible.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSpec {
    pub id: String,
    /// Hash del contenido (queda en el log: `run.model_hash`).
    pub hash: String,
    pub tier: ModelTier,
    /// Tiempo de inferencia medido en el hardware actual, en ms.
    pub est_ms: f32,
    /// VRAM que ocupa con sus activaciones, en MB.
    pub est_vram_mb: u32,
}

/// Elige el modelo más grande que cabe en el presupuesto.
///
/// `budget_ms` es el tiempo que queda para la inferencia y `vram_available_mb`
/// la memoria que se le puede dedicar (quien llama ya restó el umbral de
/// seguridad). A igual tamaño gana el más rápido. `None` si ninguno cabe: el
/// regulador debe quedarse en nivel 1 (clásico).
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

/// Error de un backend de inferencia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceError {
    Unsupported(String),
    LoadFailed(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferenceError::Unsupported(what) => write!(f, "no soportado: {what}"),
            InferenceError::LoadFailed(what) => write!(f, "no se pudo cargar el modelo: {what}"),
        }
    }
}

impl std::error::Error for InferenceError {}

/// Lo que se le pide a un backend de inferencia.
pub trait InferenceBackend {
    fn name(&self) -> &'static str;

    /// Si el hardware tiene unidades de IA aceleradas (tensor cores, WMMA, NPU).
    fn has_ai_acceleration(&self) -> bool;

    /// Carga un modelo, descargando el anterior.
    ///
    /// # Errors
    /// Si el modelo no es compatible o falla la carga.
    fn load(&mut self, model: &ModelSpec) -> Result<(), InferenceError>;

    /// Libera el modelo cargado.
    fn unload(&mut self);

    /// Modelo cargado ahora mismo, si lo hay.
    fn loaded(&self) -> Option<&ModelSpec>;
}

/// Backend sin inferencia real: solo recuerda qué modelo se cargó.
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
            "no cabe el grande en tiempo"
        );
        assert_eq!(
            select_model(&models, 3.0, 100).unwrap().id,
            "med",
            "no cabe el grande en memoria"
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
            spec("rapido", ModelTier::Medium, 1.0, 80),
        ];
        assert_eq!(select_model(&models, 3.0, 300).unwrap().id, "rapido");
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
