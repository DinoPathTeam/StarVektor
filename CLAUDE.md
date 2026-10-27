# StarVektor: guide for Claude sessions

System dependency (no GUI, with a configuration TUI) that imitates VRR. Supreme rule: **never make the image worse**; when in doubt, the regulator drops a level.

## Language
All documentation, code comments, user-facing strings and commit messages are written in **English**, so that anyone who opens the repository can read it. Code identifiers are in English too. (Conversation with the maintainers may happen in another language; the repository does not.) New documents and edits follow this from now on.

## Identity
The project is published and credited as **DinoPathTeam** (GitHub organization). Authorship, package metadata and the copyright holder refer to DinoPathTeam.

## Architecture (see docs/architecture.md)
- One crate per responsibility. Layers: `sv-types` → core (`sv-core`, `sv-telemetry`, `sv-config`) → backends (`sv-gfx`, `sv-inference`) → `sv-platform` → `sv-tui` → `starvektor-cli`.
- Allowed dependencies are in `architecture/layers.toml`; `scripts/check_layers.py` verifies them. If you need a new dependency, edit that file deliberately, not to silence the error.
- `sv-core` depends on nothing external and not on `sv-config`. The config → core translation lives only in `starvektor-cli/src/wiring.rs`.
- A change should touch one crate. If it touches several, check whether an abstraction is missing.
- `unsafe_code = "forbid"` across the workspace. FFI (C++ shim, Vulkan layer) will go in separate crates that explicitly allow it.
- Rust is the core; C++ only as a narrow shim if needed. Vulkan vs gamescope is pending the Phase 1 measurement.

## Data
- Policy D4: train only on our own scenes; commercial games for evaluation only; FSR/AFMF/DLSS FG only as comparison references. Every source is declared in `data/sources.toml`.
- The log is versioned JSONL (`docs/log-schema.md`). Changing the schema requires updating `sv-telemetry/tests/fixtures` and following the versioning rules.

## Public documentation
README, `docs/design.md`, `docs/roadmap.md` and `docs/data-policy.md` are for public reading. When closing a milestone, tick it in `docs/roadmap.md` only with evidence (measurement or test), and do not present estimates as results. The repository states that it is at an early stage and that any copy of the code is incomplete; keep that notice.

## Legal limits (docs/usage-statement.md)
Black box: do not read or write game memory, do not touch its files or network, do not extract assets, do not store frames from commercial games, do not interfere with DRM or anti-cheat, and do not train or fine-tune models on images from third-party games (not even on the user's machine: no online learning on what is being played). Single-player, offline games only; off by default in games with anti-cheat. Any change that touches this requires updating the statement and `docs/publisher-review.md` first.

## Before declaring something done
Run the six verification commands from the README. Do not claim performance from `selftest`: it is synthetic.

## Frequent changes
- New setting: `sv-config` (model, validation, env) + `config/example.toml` (a test enforces it) + `wiring.rs` + a field in `sv-tui` if it is edited there.
- New log field: `sv-telemetry/src/schema.rs`, validator, `docs/log-schema.md`, minor version.
- New backend: implement the trait in its crate; register in `wiring.rs`.
- New command: file in `starvektor-cli/src/commands/` + variant in `cli.rs`.
