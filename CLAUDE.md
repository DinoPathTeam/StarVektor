# StarVektor: guía para sesiones de Claude

Dependencia del sistema (sin GUI, con TUI de configuración) que imita VRR. Regla suprema: **nunca empeorar la imagen**; ante duda, el regulador baja de nivel.

## Idioma
Documentación y comentarios en español; identificadores de código en inglés.

## Arquitectura (ver docs/architecture.md)
- Un crate por responsabilidad. Capas: `sv-types` → núcleo (`sv-core`, `sv-telemetry`, `sv-config`) → backends (`sv-gfx`, `sv-inference`) → `sv-platform` → `sv-tui` → `starvektor-cli`.
- Las dependencias permitidas están en `architecture/layers.toml`; `scripts/check_layers.py` las verifica. Si necesitas una dependencia nueva, edita ese archivo a propósito, no para silenciar el error.
- `sv-core` no depende de nada externo ni de `sv-config`. La traducción config → núcleo vive solo en `starvektor-cli/src/wiring.rs`.
- Un cambio debe tocar un crate. Si toca varios, revisa si falta una abstracción.
- `unsafe_code = "forbid"` en todo el workspace. FFI (shim C++, capa Vulkan) irá en crates aparte que lo permitan explícitamente.
- Rust es el núcleo; C++ solo como shim acotado si hace falta. Decisión Vulkan vs gamescope pendiente de la medición de la Fase 1.

## Datos
- Política D4: entrenar solo con escenas propias; juegos comerciales solo para evaluar; FSR/AFMF/DLSS FG solo como referencia de comparación. Toda fuente se declara en `data/sources.toml`.
- El log es JSONL versionado (`docs/log-schema.md`). Cambiar el esquema exige actualizar `sv-telemetry/tests/fixtures` y respetar las reglas de versión.

## Antes de dar algo por terminado
Ejecutar los seis comandos de verificación del README. No afirmar rendimiento a partir de `selftest`: es sintético.

## Cambios frecuentes
- Ajuste nuevo: `sv-config` (modelo, validación, env) + `config/example.toml` (una prueba lo exige) + `wiring.rs` + campo en `sv-tui` si se edita ahí.
- Campo de log nuevo: `sv-telemetry/src/schema.rs`, validador, `docs/log-schema.md`, versión menor.
- Backend nuevo: implementar el trait en su crate; registrar en `wiring.rs`.
- Comando nuevo: archivo en `starvektor-cli/src/commands/` + variante en `cli.rs`.
