# StarVektor

Dependencia del sistema que imita la función del VRR en monitores que no lo tienen: genera fotogramas, predice el tiempo de cada uno y entrega al monitor una cadencia fija, sin empeorar la imagen. Sin GUI; la configuración se hace por archivo TOML o con una TUI.

**Estado: Fase 0 (andamiaje).** Hay arquitectura, regulador, log, configuración y TUI, probados con una plataforma sintética. Todavía no toca ninguna GPU ni ningún juego.

Primer objetivo: Linux + Vulkan, probado en RX 9060 XT 8 GB. Después: Windows y GPUs antiguas (RX 580, GTX 1650).

## Inicio rápido

```sh
cargo run -p starvektor-cli -- selftest          # prueba de cableado con datos sintéticos
cargo run -p starvektor-cli -- config init       # crea el archivo de configuración
cargo run -p starvektor-cli -- tui               # edita la configuración
cargo run -p starvektor-cli -- log validate F    # valida un log JSONL
```

`selftest` solo comprueba que las piezas se conectan; sus costos por nivel son inventados y **no miden rendimiento**.

## Crates: dónde tocar según el cambio

| Crate | Responsabilidad | Toca esto cuando… |
|---|---|---|
| `sv-types` | Tipos compartidos (niveles, tiempos, memoria) | un concepto lo usan varios crates |
| `sv-core` | Predictor, regulador, detector de cadena | cambias la lógica de decisión |
| `sv-telemetry` | Esquema, escritor y validador del log | cambias qué se registra |
| `sv-config` | Archivo TOML, perfiles, variables de entorno | añades un ajuste |
| `sv-gfx` | Trait `GraphicsBackend` | integras Vulkan/otra API |
| `sv-inference` | Selección de modelo y trait `InferenceBackend` | integras un motor de inferencia |
| `sv-platform` | Trait `Platform` (captura/presentación) y plataforma sintética | portas a otro SO o hook |
| `sv-tui` | Interfaz de configuración | cambias la TUI |
| `starvektor-cli` | Binario; único lugar que conecta todo (`wiring.rs`) | añades un comando |

Las dependencias permitidas entre crates están en `architecture/layers.toml` y se verifican con `scripts/check_layers.py`. Detalle en [docs/architecture.md](docs/architecture.md).

## Comandos de desarrollo

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/check_layers.py
python3 scripts/check_sources.py
python3 -m unittest discover -s scripts -p "test_*.py"
```

## Documentos

- [docs/architecture.md](docs/architecture.md): capas y dónde va cada cambio
- [docs/log-schema.md](docs/log-schema.md): esquema del log y reglas de versión
- [docs/data-policy.md](docs/data-policy.md): qué datos pueden entrenar al modelo
- El roadmap y las decisiones están en el documento maestro, fuera del repositorio.

Licencia: aún sin definir.
