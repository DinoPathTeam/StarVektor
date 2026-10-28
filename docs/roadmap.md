# Roadmap

Seven phases, in order. We do not move to the next one without meeting the exit criterion. The rule that cuts across all of them: **StarVektor must never leave the image worse than doing nothing.** The underlying decisions are in [design.md](design.md).

Current status: **Phase 0 complete**; Phase 1 is next.

## Phase 0. Foundations (complete)

Goal: repository, layers, log schema, data policy and a skeleton TUI backed by a TOML file.

- [x] H0.1 Repository with separate layers, compiling.
- [x] H0.2 Versioned log schema (`run`, `frame`, `event`, `summary`) and validator.
- [x] H0.3 Data policy and versioned list of sources, with automatic verification.
- [x] H0.4 Skeleton TUI that edits and saves the TOML file.

**Exit:** a valid `run` is written and checked against the schema; the TUI edits and saves the configuration. Checked with `starvektor selftest`, which uses synthetic data.

## Phase 1. Baseline and hook test

Goal: measure without generating anything and decide how to hook in on Linux.

- [ ] H1.1 Measurement without generating anything: frame times, rendered vs. presented fps, VRAM.
- [ ] H1.2 Detection of the OptiScaler configuration (upscaling only, FG, both).
- [ ] H1.3 Three fixed scenes per evaluation game, played by a person or with the game's own benchmark mode (no input automation), on the unmodified game.
- [ ] H1.4 Test A: implicit Vulkan layer. Overhead, 1 % and 0.1 % percentiles, latency, available engine data, compatibility and footprint (whether it runs inside the game process and what data it accesses).
- [ ] H1.5 Test B: gamescope, with the same metrics, including footprint.
- [ ] H1.6 Decision D2 recorded with the figures that support it.
- [ ] H1.7 Exclusion list of games with anti-cheat: StarVektor off by default in them (D9).

**Exit:** repeating the same scene gives comparable results (tolerance to be defined from the data) and D2 is decided.

## Phase 2. Classic core (level 1)

Goal: frame generation without a neural network.

- [ ] H2.1 Frame-time predictor.
- [ ] H2.2 Classic motion estimation and reprojection.
- [ ] H2.3 Rule-based HUD protection and heuristic confidence mask.
- [ ] H2.4 Regulator with hysteresis and VRAM monitoring.
- [ ] H2.5 "Do not intervene" mode (level 0) tested.

**Exit:** cost within budget, 1 % percentile no worse than native and at most one frame of added latency.

_Note: the predictor (H2.1) and the regulator (H2.4) already exist as code and unit tests in `sv-core`, but they are not considered met until measured with a real game._

## Phase 3. Data and first model (level 2)

Goal: dataset with provenance and a v0 model for error masking and HUD.

- [ ] H3.1 Our own scenes in an open engine captured at 120-240 fps, with a provenance record.
- [ ] H3.2 Dataset using the discarded frames as training ground truth.
- [ ] H3.3 Minimal model for error masking and HUD protection (model v0).
- [ ] H3.4 Memorization test passed.
- [ ] H3.5 Evaluation on held-out scenes (SSIM/PSNR plus visual review of HUD and edges).

**Exit:** model v0 improves on level 1 and does not memorize training scenes.

## Phase 4. Comparison

Goal: measure against the state of the art and decide whether to continue.

- [ ] H4.1 Measurement of FSR 3, AFMF 2 and OptiScaler FG on the evaluation games.
- [ ] H4.2 Comparison table of quality, latency and ms, in the four configurations per scene.
- [ ] H4.3 Explicit decision: continue, change approach or cut scope.

**Exit:** decision recorded in this repository, with the data.

## Phase 5. Full network (level 3)

Goal: learned motion estimator, filling of disoccluded regions, per-game profiles and a model family.

- [ ] H5.1 Learned motion estimator.
- [ ] H5.2 Filling of disoccluded regions (inpainting).
- [ ] H5.3 Per-game profiles.
- [ ] H5.4 Model family (large, medium, minimal) by distillation.
- [ ] H5.5 Automatic model selection by ms and VRAM budget.

**Exit:** automatic selection works and respects the VRAM cap on an 8 GB GPU.

## Phase 6. Portability and older hardware

Goal: platform layer for Windows and stepwise degradation on GPUs without AI acceleration.

- [ ] H6.1 Platform layer for Windows.
- [ ] H6.2 Tests on older GPUs (RX 580, GTX 1650) with stepwise degradation.
- [ ] H6.3 Review of the GUI decision (D3).

**Exit:** the same core runs on both systems; on hardware without AI acceleration, level 1 or 2 without making the image worse.
