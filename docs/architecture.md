# Architecture

## Principle
Each responsibility lives in one crate. Layers only depend downwards, and the rule is checked automatically (`architecture/layers.toml` + `scripts/check_layers.py`, also in CI). This way a change stays in its crate and does not force changes in the others.

```
starvektor-cli  (composition root: wiring.rs)
   ├── sv-tui
   ├── sv-platform ─ sv-gfx ─ sv-inference     (backends: traits + implementations)
   ├── sv-core   sv-telemetry   sv-config      (unaware of each other)
   └── sv-types                                (shared contracts)
```

- `sv-core` has no external dependencies. `sv-config` and `sv-core` do not know each other: `wiring.rs` converts `Config` into `RegulatorConfig`.
- Backends are defined as traits; there is a null or synthetic implementation to test without hardware.

## Where each change goes

| I want to… | I touch |
|---|---|
| Change when the level goes up or down | `sv-core/src/regulator.rs` |
| Change the frame-time prediction | `sv-core/src/timing.rs` |
| Add a user setting | `sv-config` (`model`, `validate`, `env`), `config/example.toml`, `wiring.rs`, `sv-tui` if editable |
| Add a log field | `sv-telemetry/src/schema.rs`, `validate.rs`, `docs/log-schema.md`, bump the minor version |
| Add a graphics backend | new crate or module in `sv-gfx` implementing `GraphicsBackend`; register in `wiring.rs` |
| Add an inference engine or model | `sv-inference`; the budget-based choice is in `select_model` |
| Port to another OS or hook (Vulkan layer, gamescope) | implement `Platform` in `sv-platform` |
| New measurement source on Linux | add a reader in `sv-platform-linux`; map it in `starvektor-cli/src/commands/measure.rs` |
| Add a command | `starvektor-cli/src/commands/` and `cli.rs` |
| Change the TUI | `sv-tui` (`app.rs` is pure state; `ui.rs` only draws) |

## How to add a backend (example: Vulkan)
1. Create the module/crate; if it needs `unsafe`, make it a separate crate with the lint explicitly relaxed.
2. Implement `GraphicsBackend` (`name`, `memory_budget`, `last_pass_timings`).
3. Add its allowed external dependencies in `architecture/layers.toml`.
4. Select it in `wiring.rs`. No other crate changes.

## Regulator (summary)
Levels 0 to 3 (off, classic, minimal network, full network). It drops quickly (`panic_frames`), rises slowly (`settle_frames`) with hysteresis and exponential backoff (`max_backoff`). It switches off if the game already matches the refresh rate and drops a level if free VRAM falls below `vram_floor_mb`.

## Open decisions
- Vulkan layer vs gamescope: decided with the Phase 1 measurements.
- C++ shim: only if the measurements justify it.
