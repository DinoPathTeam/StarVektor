//! Núcleo agnóstico de StarVektor.
//!
//! Aquí no hay llamadas al sistema operativo ni a la API gráfica: todo lo que
//! entra son números y todo lo que sale son decisiones. Eso lo hace probable
//! sin hardware y portable a cualquier sistema (ver `docs/architecture.md`).
//!
//! Cada módulo es independiente de los demás:
//! - [`timing`]: predicción del tiempo de fotograma.
//! - [`regulator`]: sube y baja el nivel según costo y memoria (con histéresis).
//! - [`chain`]: detecta si la cadena trae upscaling o generación de fotogramas.
//!
//! La máscara de confianza y la estimación de movimiento (fase 2) tendrán su
//! propio módulo; no se tocará ninguno de los anteriores para añadirlos.

pub mod chain;
pub mod regulator;
pub mod timing;
