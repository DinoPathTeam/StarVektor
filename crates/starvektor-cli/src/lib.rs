//! Raíz de composición de StarVektor.
//!
//! Es el único crate que conoce a todos los demás. Aquí no hay lógica propia:
//! se convierten los tipos de una capa a los de otra y se conectan las piezas.
//! Cada comando vive en su archivo bajo [`commands`], así añadir o cambiar uno
//! no toca a los demás.
//!
//! - [`wiring`]: traducción entre capas (configuración -> parámetros del núcleo).
//! - [`ids`]: identificadores de ejecución.
//! - [`commands`]: `tui`, `config`, `log` y `selftest`.

pub mod cli;
pub mod commands;
pub mod ids;
pub mod wiring;
