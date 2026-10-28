# Design and prototype sheet

Technical document for the prototype. **All figures are design estimates, not measurements**: they will be validated with GPU timestamp queries on real hardware, with the game running, starting in Phase 1.

## Current decisions

If a decision changes, it is updated here with the date and the reason. A milestone is not considered met without its exit criterion.

| | Decision |
|---|---|
| D1. Language | The core (predictor, regulator, confidence mask, telemetry, TUI) is Rust. C++ only where the ecosystem requires it (a thin shim for the Vulkan layer or the vendor's inference SDK), with a narrow boundary: a small C interface, coarse calls per frame, clear buffer ownership rules and one-way dependency. Shaders are compiled separately. |
| D2. Linux hook | Decided by measurement, not preference: implicit Vulkan layer versus gamescope, with the same protocol. The one with the lowest overhead and best presentation pacing wins; the other is kept only if it offers something the first does not. A **footprint** criterion is added: what each option touches (whether it runs inside the game process and what data it accesses), because it feeds the [usage statement](usage-statement.md). Note: gamescope also installs a Vulkan layer inside the game, so "outside the process" only describes our code, not the whole path. Hypothesis to verify: a hybrid (the layer extracts engine data and gamescope presents on time), which would also double the footprint. A **protected games mode** is planned for titles with anti-cheat or anti-tamper: gamescope only, no component of ours in the game process, measurements taken from the compositor side; whether gamescope can host frame generation is part of H1.5. **Pending (Phase 1).** |
| D3. Configuration | A TUI over a TOML file and environment variables, per-game profiles and a minimal overlay (frame time, active level, hotkey to mark artifacts). No GUI for now. |
| D4. Data | Training only on our own scenes in an open engine and sources with explicit permission. Commercial games for evaluation only. Output of other generators as reference only. Provenance mandatory. See [data-policy.md](data-policy.md). |
| D5. Cross-cutting rule | Never leave the image worse than doing nothing. When in doubt, repeat the frame or switch off (level 0). |
| D6. Order of work | Linux first; Vulkan as the base so that the platform layer is the only thing that changes when porting. Older hardware after it works on modern hardware. |
| D7. OptiScaler chain | Do not stack two frame generators. Detect the configuration instead of assuming it; always measure rendered and presented fps. |
| D8. Minimal footprint (black box) | StarVektor only observes the image the system already presents (Vulkan, Wayland/X11) and measures its own timings. It does not read or write game memory, does not modify binaries or files, does not extract assets, does not intercept the network and does not interfere with protections. It also does not train or fine-tune models on images from third-party games, not even on the user's machine. The level that uses data from the game's upscaler (depth, vectors) is reserved for integration done by the game's own developer, and is not used on third-party games without permission. Details in [usage-statement.md](usage-statement.md). |
| D9. Scope | Single-player, offline games. Games with anti-cheat are out of scope: StarVektor is off by default in them (exclusion list, milestone H1.7) and is never tested on online accounts. |
| D10. License and distribution | Planned: MIT, so that distros can include it as a dependency and engines as an SDK. Model weights and datasets will be licensed separately. There is no `LICENSE` file yet. Copyright holder: DinoPathTeam. |

## Layers

1. **Agnostic core:** time prediction, regulator (with hysteresis), confidence mask, configuration detection, telemetry. No operating system calls.
2. **Graphics backend:** Vulkan compute behind a small interface. Inference behind an abstract interface (AI units today, generic shaders later).
3. **Platform layer:** capture and presentation (Wayland/gamescope on Linux), input reading, hooking with OptiScaler and the game's upscaler. It is the only thing that changes between Linux and Windows.

## Configuration: TUI yes, GUI not for now

A TUI suits a dependency: it runs over SSH or in a terminal, needs no graphical server of its own and is maintained with little code. So that it is not the only path, every option lives in the TOML file and in environment variables (useful in a Steam launch option or a script), there are per-game profiles and a minimal overlay for testing. A thin GUI over the same file would be added if non-technical users are targeted, especially on Windows.

## Time budget (1440p, estimated)

Base rule: the per-frame cost should be 10 to 15 % of the base time T. At a 60 fps base (T ≈ 16.7 ms), about 1.7 to 2.5 ms.

| Pass | target ms |
|---|---|
| Zero-copy capture | 0.3 |
| Motion estimation | 0.8 to 1.0 |
| Reprojection and blending | 0.4 to 0.5 |
| Confidence mask and HUD protection | 0.3 to 0.5 |
| Presentation | 0.2 |
| **Total** | **≈ 2.0 to 2.5** |

## VRAM budget (8 GB, estimated)

| Item | MB |
|---|---|
| History of 2 to 3 frames | 30 to 45 |
| Vectors and approximate depth (half resolution) | 10 to 20 |
| Masks | < 5 |
| Model weights | 2 to 10 |
| Activations (half resolution) | ≈ 30 |
| **Target / hard cap** | **150 to 250 / 400** |

If free VRAM drops below about 400 MB (`VK_EXT_memory_budget`), the regulator drops a level. This is why VRAM per model matters: the model size is chosen according to the available budget.

## Test cases

Three fixed scenes per game (exploration, combat with particles, changing lighting) in four configurations: native, with upscaling, with upscaling + OptiScaler FG, and with StarVektor. Commercial games are used for evaluation only.

| Game | What it stresses |
|---|---|
| Cyberpunk 2077 | Dense HUD, neon, transparencies |
| Control | Reflections, particles, destructible geometry |
| The Witcher 3 (2026 version) | Dense vegetation, skies, shimmering |
| Crimson Desert | Huge map, vegetation, varied weather, destruction, dynamic lighting |
| The Elder Scrolls V: Skyrim Special Edition | Old engine with physics tied to 60 fps, open world, first and third person; unmodified base game |
| The Medium | Two worlds shown at once, with a moving split between two different images: stresses the confidence mask and edges |
| Hell is Us | Free VRAM headroom, to measure generation quality |
| Metro: Last Light Redux | Free VRAM headroom, dark scenes, particles |

Control is expected to run with very little free VRAM on an 8 GB card, so it is used with high textures (to check the safe switch-off) and with reduced textures (to leave room for levels 1 to 3); this is an expectation to be confirmed by measurement. For the older-hardware phase, more single-player games will be added. Competitive online games are out of scope (D9); StarVektor stays off in them.

## Open decisions

- Implicit Vulkan layer or inside gamescope (Phase 1, with measurements).
- Order of the chain when the user enables OptiScaler's frame generation: measured in Phase 1; the starting recommendation is not to stack two generators.
