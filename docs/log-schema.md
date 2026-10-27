# Log schema (version 1.0)

Source of truth: `crates/sv-telemetry/src/schema.rs`. This document summarizes it.

JSONL format: one line per record, each with `schema` and `type`. Order: a `run` first, then `frame` and `event`, a `summary` at the end. No images or game data are stored.

| `type` | When | Main fields |
|---|---|---|
| `run` | once, at the start | `run_id`, `started_at` (UTC), `game`, `game_version`, `scene`, `gpu`, `driver`, `mesa?`, `resolution`, `optiscaler`, `integration_level`, `model_id?`, `model_hash?`, `seed`, `config` |
| `frame` | one per presented frame | `n` (increasing), `t_render_ms`, `t_present_ms`, `kind`, `pass_ms`, `vram_used_mb`, `vram_free_mb`, `level`, `repeat_pct` (0–100), `ssim?`, `psnr?` |
| `event` | point-in-time | `n`, `event` (`level_change`, `scene_cut`, `frame_spike`, `vram_low`, `user_mark`), `detail` |
| `summary` | once, at the end | `fps_rendered_avg`, `fps_presented_avg`, `p1_ms`, `p01_ms`, `latency_est_ms?`, `overhead_ms_avg`, `level_time_pct` |

`?` = optional. `ssim` and `psnr` only exist when there is reference ground truth. `latency_est_ms` is absent if it could not be estimated (it is never filled with 0).

## Definitions
- `p1_ms` and `p01_ms`: frame time at the 99th and 99.9th percentile (the slowest 1 % and 0.1 %), by nearest rank, over the intervals between presented frames.
- `level_time_pct`: percentage of presented frames per level; keys `"0"` to `"3"`.

## Versioning
- New optional field: compatible, bumps the minor version (`1.1`).
- Removing, renaming or changing the type of a field: incompatible, bumps the major version.
- Readers accept any minor version of the major they know.
- `sv-telemetry/tests/compat.rs` reads `tests/fixtures/v1_minimal.jsonl`: if that test fails, compatibility was broken.

## Validate
`starvektor log validate FILE`
