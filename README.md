# StarVektor

**Cadencia de fotogramas estable en cualquier monitor, sin depender del VRR.**

StarVektor es una dependencia del sistema, sin interfaz gráfica, que intenta imitar por software lo que hace la frecuencia de refresco variable (VRR) en los monitores que la traen: que lo que ves se presente de forma pareja aunque el juego no mantenga un ritmo constante. Para lograrlo genera fotogramas intermedios, predice cuánto tardará el siguiente y decide en cada momento cuánto esfuerzo vale la pena gastar, con una regla por encima de todas: **nunca dejar la imagen peor que no hacer nada**.

> **Estado: Fase 0 completada, Fase 1 por empezar.** Existe la arquitectura, el regulador, el formato de log, la configuración y una TUI, verificados con una plataforma sintética. **Todavía no se engancha a ningún juego ni usa la GPU.** Nada de lo que aparece como cifra de rendimiento en este repositorio está medido: son hipótesis a validar (ver [docs/design.md](docs/design.md)).

## Por qué existe

Las consolas se ven fluidas en pantallas sin VRR porque el sistema controla toda la cadena de presentación. En un PC, el monitor suele ser quien absorbe las variaciones de tiempo de cada fotograma, y sin VRR queda el tearing o la latencia del V-Sync. Las soluciones actuales de generación de fotogramas (FSR 3, AFMF 2, DLSS FG, la de OptiScaler) mejoran la fluidez pero no están pensadas como sustituto del VRR, y a veces dejan artefactos visibles, sobre todo en el HUD y los bordes. StarVektor explora si un sistema que **aprende de resultados** y que **sabe retirarse cuando duda** puede acercarse más.

## Cómo funciona (visión técnica)

1. **Predictor de tiempos.** Estima el tiempo del próximo fotograma a partir del historial (media y varianza exponenciales) para decidir cuándo presentar.
2. **Generación de fotogramas.** Estimación de movimiento, reproyección y mezcla para producir fotogramas intermedios.
3. **Máscara de confianza.** Marca las zonas donde la interpolación probablemente falle (HUD, bordes, partículas); ahí se repite el fotograma en lugar de arriesgar un artefacto.
4. **Percepción aprendida.** Redes pequeñas para la máscara de errores, la protección del HUD y, más adelante, el movimiento y el relleno de zonas descubiertas.
5. **Regulador de niveles (0 a 3).** Elige cuánto cómputo usar según el presupuesto de tiempo y de VRAM.

| Nivel | Qué corre | Cuándo |
|---|---|---|
| 0 | Apagado | El juego ya iguala el refresco, o ni lo clásico cabe en el presupuesto |
| 1 | Todo clásico (flujo óptico, reglas de HUD, máscara heurística) | Línea base y hardware sin aceleración de IA |
| 2 | Clásico + red mínima para máscara de errores y HUD | Margen de GPU medio |
| 3 | Red completa de percepción | Hardware moderno con margen |

El regulador baja de nivel rápido (pocos fotogramas malos seguidos) y sube despacio, con histéresis y espera creciente tras bajadas forzadas, para no oscilar. También baja si la VRAM libre cae por debajo de un mínimo.

Regla de presupuesto: StarVektor debe costar entre el 10 y el 15 % del tiempo base de un fotograma. Con 60 fps base serían unos 1.7 a 2.5 ms. Detalle y estimaciones por pasada en [docs/design.md](docs/design.md).

### Compatibilidad con OptiScaler

Mucha gente ya usa OptiScaler para escalado o generación de fotogramas. StarVektor detecta la configuración (solo escalado, FG o ambos) en lugar de asumirla, no apila dos generadores de fotogramas y siempre mide por separado los fps renderizados y los presentados.

## Alcance y objetivos

- **Primero:** Linux con Vulkan, probado en una AMD RX 9060 XT de 8 GB.
- **Después:** Windows y GPUs antiguas sin aceleración de IA (RX 580, GTX 1650), con degradación escalonada.
- **Un jugador, sin conexión.** No está pensado para juegos online ni con anticheat; ver la [declaración de uso](docs/usage-statement.md).
- **Sin GUI.** Se configura con un archivo TOML y variables de entorno; una TUI (`starvektor tui`) edita ese mismo archivo. Se reevaluará una GUI si se apunta a usuarios no técnicos.

## Datos y privacidad

StarVektor aprende de **resultados**, no copia ni guarda contenido de juegos o SDK.

- Se entrena solo con escenas propias en motores abiertos y fuentes con licencia o permiso explícito.
- Los juegos comerciales se usan **solo para evaluar**, nunca para entrenar.
- La salida de otros generadores de fotogramas es solo referencia de comparación.
- Los logs de telemetría no contienen imágenes ni datos del juego.

Política completa: [docs/data-policy.md](docs/data-policy.md). Formato del log: [docs/log-schema.md](docs/log-schema.md). Qué hace y qué no hace el programa sobre los juegos: [docs/usage-statement.md](docs/usage-statement.md). Lectura de los acuerdos de los editores: [docs/publisher-review.md](docs/publisher-review.md).

## Hoja de ruta

| Fase | Objetivo | Estado |
|---|---|---|
| 0. Cimientos | Repositorio, capas, log, política de datos, TUI esqueleto | Completada |
| 1. Línea base y enganche | Medir sin generar; decidir capa Vulkan frente a gamescope | Siguiente |
| 2. Núcleo clásico (nivel 1) | Generación de fotogramas sin red | Pendiente |
| 3. Datos y primer modelo (nivel 2) | Dataset con procedencia y modelo v0 | Pendiente |
| 4. Comparación | Medir contra FSR 3, AFMF 2 y FG de OptiScaler; decidir si seguir | Pendiente |
| 5. Red completa (nivel 3) | Movimiento aprendido, perfiles, familia de modelos | Pendiente |
| 6. Portabilidad | Windows y hardware antiguo | Pendiente |

Cada fase tiene hitos y un criterio de salida; no se pasa a la siguiente sin cumplirlo. Detalle en [docs/roadmap.md](docs/roadmap.md).

## Arquitectura

Workspace de Rust con un crate por responsabilidad, para que un cambio quede en un solo lugar. Las dependencias permitidas entre crates se declaran en `architecture/layers.toml` y se verifican automáticamente.

| Crate | Responsabilidad |
|---|---|
| `sv-types` | Tipos compartidos (niveles, tiempos, memoria) |
| `sv-core` | Predictor, regulador, detector de cadena (sin dependencias externas) |
| `sv-telemetry` | Esquema, escritor y validador del log |
| `sv-config` | Archivo TOML, perfiles por juego, variables de entorno |
| `sv-gfx` | Interfaz del backend gráfico (Vulkan, a futuro) |
| `sv-inference` | Selección de modelo e interfaz de inferencia |
| `sv-platform` | Captura y presentación; plataforma sintética para pruebas |
| `sv-tui` | Interfaz de configuración en terminal |
| `starvektor-cli` | Binario; único lugar donde se conecta todo |

El núcleo es Rust. C++ entraría solo como un shim estrecho si la capa Vulkan o un SDK de inferencia lo exigen. Más en [docs/architecture.md](docs/architecture.md).

## Probarlo

Requiere Rust estable.

```sh
cargo run -p starvektor-cli -- selftest        # prueba de cableado con datos sintéticos
cargo run -p starvektor-cli -- config init     # crea el archivo de configuración
cargo run -p starvektor-cli -- tui             # edita la configuración
cargo run -p starvektor-cli -- log validate F  # valida un log JSONL
```

`selftest` solo comprueba que las piezas se conectan; sus costos son inventados y **no miden rendimiento**. Ejemplo de configuración comentado: [config/example.toml](config/example.toml).

## Contribuir

El proyecto está en una etapa temprana y la estructura cambiará. Antes de proponer cambios, ejecuta las verificaciones:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/check_layers.py
python3 scripts/check_sources.py
python3 -m unittest discover -s scripts -p "test_*.py"
```

Cualquier dato de entrenamiento debe declararse en `data/sources.toml` conforme a la política de datos.

## Licencia

Prevista: MIT. Todavía no hay archivo `LICENSE`.
