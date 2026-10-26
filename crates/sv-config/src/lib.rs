//! Configuración de StarVektor (decisión D3).
//!
//! Todo lo configurable vive en un archivo TOML y en variables de entorno; la
//! TUI (`sv-tui`) solo lo edita. Así funciona igual desde una opción de
//! lanzamiento de Steam o un script, sin abrir nada.
//!
//! Este crate no conoce `sv-core`: el modelo es datos planos y la conversión a
//! los tipos del núcleo se hace en el crate de composición. Por eso se puede
//! cambiar el regulador sin tocar la configuración, y al revés.
//!
//! - [`model`]: estructuras y valores por defecto.
//! - [`effective`]: resolución de perfil por juego.
//! - [`validate`]: reglas de rango.
//! - [`env`]: sobrescrituras desde variables de entorno.
//! - [`io`]: carga, guardado atómico y rutas por defecto.

pub mod effective;
pub mod env;
pub mod error;
pub mod io;
pub mod model;
pub mod validate;

pub use effective::Effective;
pub use error::{ConfigError, ConfigIssue};
pub use model::{Config, HookMode};
