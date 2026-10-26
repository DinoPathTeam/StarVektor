# Esquema del log (versión 1.0)

Fuente de verdad: `crates/sv-telemetry/src/schema.rs`. Este documento lo resume.

Formato JSONL: una línea por registro, cada una con `schema` y `type`. Orden: un `run` primero, luego `frame` y `event`, un `summary` al final. No se guardan imágenes ni datos del juego.

| `type` | Cuándo | Campos principales |
|---|---|---|
| `run` | una vez, al inicio | `run_id`, `started_at` (UTC), `game`, `game_version`, `scene`, `gpu`, `driver`, `mesa?`, `resolution`, `optiscaler`, `integration_level`, `model_id?`, `model_hash?`, `seed`, `config` |
| `frame` | uno por fotograma presentado | `n` (creciente), `t_render_ms`, `t_present_ms`, `kind`, `pass_ms`, `vram_used_mb`, `vram_free_mb`, `level`, `repeat_pct` (0–100), `ssim?`, `psnr?` |
| `event` | puntual | `n`, `event` (`level_change`, `scene_cut`, `frame_spike`, `vram_low`, `user_mark`), `detail` |
| `summary` | una vez, al cerrar | `fps_rendered_avg`, `fps_presented_avg`, `p1_ms`, `p01_ms`, `latency_est_ms?`, `overhead_ms_avg`, `level_time_pct` |

`?` = opcional. `ssim` y `psnr` solo existen si hay verdad de referencia. `latency_est_ms` falta si no se pudo estimar (nunca se rellena con 0).

## Definiciones
- `p1_ms` y `p01_ms`: tiempo de fotograma en el percentil 99 y 99.9 (el 1 % y 0.1 % más lentos), por rango más cercano, sobre los intervalos entre fotogramas presentados.
- `level_time_pct`: porcentaje de fotogramas presentados por nivel; claves `"0"` a `"3"`.

## Versionado
- Campo opcional nuevo: compatible, sube la versión menor (`1.1`).
- Quitar, renombrar o cambiar el tipo de un campo: incompatible, sube la mayor.
- Los lectores aceptan cualquier menor de la mayor que conocen.
- `sv-telemetry/tests/compat.rs` lee `tests/fixtures/v1_minimal.jsonl`: si esa prueba falla, se rompió la compatibilidad.

## Validar
`starvektor log validate ARCHIVO`
