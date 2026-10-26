# Arquitectura

## Principio
Cada responsabilidad vive en un crate. Las capas solo dependen hacia abajo, y la regla se verifica automáticamente (`architecture/layers.toml` + `scripts/check_layers.py`, también en CI). Así un cambio queda en su crate y no obliga a tocar los demás.

```
starvektor-cli  (raíz de composición: wiring.rs)
   ├── sv-tui
   ├── sv-platform ─ sv-gfx ─ sv-inference     (backends: traits + implementaciones)
   ├── sv-core   sv-telemetry   sv-config      (sin conocerse entre sí)
   └── sv-types                                (contratos compartidos)
```

- `sv-core` no tiene dependencias externas. `sv-config` y `sv-core` no se conocen: `wiring.rs` convierte `Config` en `RegulatorConfig`.
- Los backends se definen como traits; hay una implementación nula o sintética para probar sin hardware.

## Dónde va cada cambio

| Quiero… | Toco |
|---|---|
| Cambiar cuándo sube o baja de nivel | `sv-core/src/regulator.rs` |
| Cambiar la predicción de tiempos | `sv-core/src/timing.rs` |
| Añadir un ajuste de usuario | `sv-config` (`model`, `validate`, `env`), `config/example.toml`, `wiring.rs`, `sv-tui` si es editable |
| Añadir un campo al log | `sv-telemetry/src/schema.rs`, `validate.rs`, `docs/log-schema.md`, subir versión menor |
| Añadir un backend gráfico | nuevo crate o módulo en `sv-gfx` que implemente `GraphicsBackend`; registrar en `wiring.rs` |
| Añadir un motor de inferencia o modelo | `sv-inference`; la elección por presupuesto está en `select_model` |
| Portar a otro SO o enganche (capa Vulkan, gamescope) | implementar `Platform` en `sv-platform` |
| Añadir un comando | `starvektor-cli/src/commands/` y `cli.rs` |
| Cambiar la TUI | `sv-tui` (`app.rs` es estado puro; `ui.rs` solo dibuja) |

## Cómo añadir un backend (ejemplo: Vulkan)
1. Crear el módulo/crate; si necesita `unsafe`, que sea un crate aparte con el lint relajado explícitamente.
2. Implementar `GraphicsBackend` (`name`, `memory_budget`, `last_pass_timings`).
3. Añadir sus dependencias externas permitidas en `architecture/layers.toml`.
4. Elegirlo en `wiring.rs`. Ningún otro crate cambia.

## Regulador (resumen)
Niveles 0 a 3 (apagado, clásico, red mínima, red completa). Baja rápido (`panic_frames`), sube despacio (`settle_frames`) con histéresis y espera exponencial (`max_backoff`). Se apaga si el juego ya iguala el refresco y baja si la VRAM libre cae bajo `vram_floor_mb`.

## Decisiones abiertas
- Capa Vulkan vs gamescope: se decide con la medición de la Fase 1.
- Shim C++: solo si la medición lo justifica.
