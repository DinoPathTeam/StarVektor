//! Telemetría de StarVektor: log JSONL versionado.
//!
//! Un archivo por ejecución, una línea por registro. Nunca contiene imágenes ni
//! datos del juego (ver `docs/data-policy.md`). El esquema se describe en
//! `docs/log-schema.md`; la fuente de verdad es [`schema`].
//!
//! Módulos independientes entre sí, salvo que todos usan [`schema`]:
//! - [`schema`]: tipos de los registros (`run`, `frame`, `event`, `summary`).
//! - [`writer`]: escritura asíncrona, sin sumar milisegundos al fotograma.
//! - [`summary`]: calcula el registro `summary` a partir de los `frame`.
//! - [`validate`]: comprueba un log contra el esquema.
//! - [`time`]: marcas de tiempo ISO 8601 sin dependencias externas.

pub mod schema;
pub mod summary;
pub mod time;
pub mod validate;
pub mod writer;

#[cfg(test)]
mod fixtures;
