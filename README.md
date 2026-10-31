# StarVektor

**Steady frame cadence on any monitor, without depending on VRR.**

StarVektor is a system dependency, with no graphical interface, that tries to imitate in software what variable refresh rate (VRR) does on monitors that support it: what you see is presented evenly even when the game does not hold a constant pace. To do so it generates intermediate frames, predicts how long the next one will take, and decides at every moment how much effort is worth spending, under one rule above all others: **never leave the image worse than doing nothing**.

Developed by [DinoPathTeam](https://github.com/DinoPathTeam).

> **Early stage.** This repository is work in progress: **the code is incomplete and changes without notice**. Any copy, fork or download is a partial version: it cannot be used with games, comes with no warranty, and must not be used as the basis of anything in production.

> **Status: Phase 0 complete, Phase 1 about to start.** The architecture, regulator, log format, configuration and a TUI exist, verified with a synthetic platform. **It does not yet hook into any game or use the GPU.** None of the performance figures in this repository has been measured: they are hypotheses to be validated (see [docs/design.md](docs/design.md)).

## Why it exists

Consoles look smooth on screens without VRR because the system controls the whole presentation chain. On a PC the monitor usually absorbs the variation in each frame's timing, and without VRR you are left with tearing or V-Sync latency. Current frame generation solutions (FSR 3, AFMF 2, DLSS FG, OptiScaler's) improve smoothness but are not designed as a VRR substitute, and sometimes leave visible artifacts, especially on the HUD and at edges. StarVektor explores whether a system that **learns from results** and **knows when to step back** can get closer.

## How it works (technical overview)

1. **Frame-time predictor.** Estimates the time of the next frame from history (exponential mean and variance) to decide when to present.
2. **Frame generation.** Motion estimation, reprojection and blending to produce intermediate frames.
3. **Confidence mask.** Marks the areas where interpolation is likely to fail (HUD, edges, particles); there the frame is repeated instead of risking an artifact.
4. **Learned perception.** Small networks for the error mask, HUD protection and, later, motion and filling of disoccluded areas.
5. **Level regulator (0 to 3).** Chooses how much compute to use according to the time and VRAM budget.

| Level | What runs | When |
|---|---|---|
| 0 | Off | The game already matches the refresh rate, or even the classic path does not fit the budget |
| 1 | All classic (optical flow, HUD rules, heuristic mask) | Baseline and hardware without AI acceleration |
| 2 | Classic + minimal network for the error mask and HUD | Medium GPU headroom |
| 3 | Full perception network | Modern hardware with headroom |

The regulator drops a level quickly (a few consecutive bad frames) and rises slowly, with hysteresis and growing wait after forced drops, to avoid oscillation. It also drops if free VRAM falls below a minimum.

Budget rule: StarVektor should cost 10 to 15 % of a frame's base time. At a 60 fps base that is about 1.7 to 2.5 ms. Details and per-pass estimates in [docs/design.md](docs/design.md).

### OptiScaler compatibility

Many people already use OptiScaler for upscaling or frame generation. StarVektor detects the configuration (upscaling only, FG, or both) instead of assuming it, never stacks two frame generators, and always measures rendered and presented fps separately.

## Scope and goals

- **First:** Linux with Vulkan, tested on an AMD RX 9060 XT with 8 GB.
- **Later:** Windows and older GPUs without AI acceleration (RX 580, GTX 1650), with stepwise degradation.
- **Single-player, offline.** It is not intended for online games or games with anti-cheat; see the [usage statement](docs/usage-statement.md).
- **No GUI.** It is configured with a TOML file and environment variables; a TUI (`starvektor tui`) edits that same file. A GUI will be reconsidered if non-technical users are targeted.

## Data and privacy

StarVektor learns from **results**; it does not copy or store game or SDK content.

- It is trained only on our own scenes in open engines and on sources with an explicit license or permission.
- Commercial games are used **for evaluation only**, never for training.
- The output of other frame generators is a comparison reference only.
- Telemetry logs contain no images or game data.

Full policy: [docs/data-policy.md](docs/data-policy.md). Log format: [docs/log-schema.md](docs/log-schema.md). What the program does and does not do to games: [docs/usage-statement.md](docs/usage-statement.md). Review of publisher agreements: [docs/publisher-review.md](docs/publisher-review.md).

## Roadmap

| Phase | Goal | Status |
|---|---|---|
| 0. Foundations | Repository, layers, log, data policy, skeleton TUI | Complete |
| 1. Baseline and hook | Measure without generating; decide Vulkan layer vs gamescope | Next |
| 2. Classic core (level 1) | Frame generation without a network | Pending |
| 3. Data and first model (level 2) | Dataset with provenance and model v0 | Pending |
| 4. Comparison | Measure against FSR 3, AFMF 2 and OptiScaler FG; decide whether to continue | Pending |
| 5. Full network (level 3) | Learned motion, profiles, model family | Pending |
| 6. Portability | Windows and older hardware | Pending |

Each phase has milestones and an exit criterion; the next phase is not started until it is met. Details in [docs/roadmap.md](docs/roadmap.md).

## Architecture

A Rust workspace with one crate per responsibility, so that a change stays in one place. The dependencies allowed between crates are declared in `architecture/layers.toml` and checked automatically.

| Crate | Responsibility |
|---|---|
| `sv-types` | Shared types (levels, timings, memory) |
| `sv-core` | Predictor, regulator, chain detector (no external dependencies) |
| `sv-telemetry` | Log schema, writer and validator |
| `sv-config` | TOML file, per-game profiles, environment variables |
| `sv-gfx` | Graphics backend interface (Vulkan, in the future) |
| `sv-inference` | Model selection and inference interface |
| `sv-platform` | Capture and presentation; synthetic platform for tests |
| `sv-platform-linux` | Linux measurement sources read from outside the game (mangoapp CSV, amdgpu sysfs) |
| `sv-tui` | Terminal configuration interface |
| `starvektor-cli` | Binary; the only place where everything is wired together |

The core is Rust. C++ would enter only as a narrow shim if the Vulkan layer or an inference SDK requires it. More in [docs/architecture.md](docs/architecture.md).

## Try it

Requires stable Rust.

```sh
cargo run -p starvektor-cli -- selftest        # wiring test with synthetic data
cargo run -p starvektor-cli -- measure FILE.csv --game NAME --scene LABEL   # baseline log from a mangoapp CSV (no generation)
# add --segment-seconds 180 to cut a long recording into 3-minute logs
cargo run -p starvektor-cli -- config init     # creates the configuration file
cargo run -p starvektor-cli -- tui             # edits the configuration
cargo run -p starvektor-cli -- log validate F  # validates a JSONL log
```

`selftest` only checks that the pieces connect; its costs are made up and it **does not measure performance**. Commented configuration example: [config/example.toml](config/example.toml).

## Contributing

The project is at an early stage and the structure will change. Before proposing changes, run the checks:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/check_layers.py
python3 scripts/check_sources.py
python3 -m unittest discover -s scripts -p "test_*.py"
```

Any training data must be declared in `data/sources.toml` in accordance with the data policy. All documentation and code comments in this repository are written in English.

## License

Planned: MIT. Until a `LICENSE` file exists, no license to use the code is granted. Remember that the code is at an early stage and is incomplete.

Copyright holder: DinoPathTeam.
