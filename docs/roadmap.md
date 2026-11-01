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

- [x] H1.1 Measurement without generating anything: frame times, rendered vs. presented fps, VRAM. Evidence: `starvektor measure` turned mangoapp recordings of The Elder Scrolls V: Skyrim Special Edition (unmodified, played by a person, gamescope capped at 60 fps) into schema-valid logs; MangoHud VRAM matched the amdgpu `sysfs` counter within 0.5 % in a steady scene (3,348 MB mean vs. 3,340 to 3,352 MB read). Rendered and presented fps are identical by construction, since nothing is generated yet.
- [ ] H1.2 Detection of the OptiScaler configuration (upscaling only, FG, both).
- [ ] H1.3 Three fixed scenes per evaluation game, played by a person or with the game's own benchmark mode (no input automation), on the unmodified game. Progress: 2 of 9 evaluation games (Skyrim Special Edition and Metro 2033 Redux). Three scenes (Helgen keep combat, Whiterun exteriors, Whiterun to Riverwood), three repetitions each from the same save, gamescope capped at 60 fps. Across repetitions the p99 frame time varied by 0.48 ms (combat), 0.09 ms (Whiterun) and 0.07 ms (Riverwood), and the p99.9 by 0.66, 0.16 and 0.10 ms; mean fps was 60.0 in all nine runs. Skyrim at 60 fps leaves the GPU mostly idle, so these ranges are a floor: they must be re-measured on a game that loads the GPU. Metro 2033 Redux is the chosen GPU-heavy game (store page checked: single-player, no anti-cheat or third-party DRM listed). At 1080p it is not demanding, so the load was raised to 2560x1440 internal (gamescope) with 4A TSR, Shading Rate 2.0x, tessellation Very high, VSync off, output 1920x1080 at 60 Hz and the `performance` CPU governor; the game was relaunched before each repetition and played by hand from the same save. In every run the GPU load averaged 99 to 100 %, the GPU clock stayed between 2666 and 2677 MHz and its temperature between 51 and 64 °C, without throttling. Four scenes were measured. `tower-1440` (end of the prologue, five repetitions, first 180 s of each): mean fps 76.7 to 78.9, median frame time 12.91 to 13.13 ms, p99 spread 1.92 ms and p99.9 spread 1.89 ms, with no trend across runs and no run excluded. Three chapter 1 scenes (three repetitions each), delimited by in-game events and not by the clock, with load screens and cutscenes left out: `cap1-hunter-combate` (from entering Hunter's room until the last enemy falls, 197 to 201 s) had 77.9 to 79.4 fps, a median spread of 0.52 ms, a p99 spread of 1.14 ms and a p99.9 spread of 0.92 ms; `cap1-exposicion` (from the end of the loading, with the camera free, until just before talking to the NPC that leads to the trolley, 130 to 138 s) had 100.0 to 100.8 fps, a median spread of 0.04 ms, a p99 spread of 0.23 ms and a p99.9 spread of 0.12 ms; `cap1-vagon-combate` (from after the cutscene with the Dark Ones until the last enemy falls after the flamethrower, 164 to 168 s) had 81.7 to 83.5 fps, a median spread of 0.33 ms, a p99 spread of 0.09 ms and a p99.9 spread of 0.08 ms. The combat in Hunter's room is not deterministic, and it is the scene with the widest tails. In the second repetition of the three chapter 1 scenes the VRAM in use was about 250 MB higher than in the other two (3.55 GB against 3.25 to 3.33 GB) with no visible effect on the frame times; it is a system-wide figure, so another process may explain it. The routes were played by hand. The tolerance still has to be set from these spreads, and the Skyrim ones, before H1.3 is ticked. Euro Truck Simulator 2 is the next game to be measured, with continuous routes of at least 30 minutes, because three minutes say little in a driving game. Other candidates are listed under `evaluation_candidates` in `data/sources.toml`.
- [ ] H1.4 Test A: implicit Vulkan layer. Overhead, 1 % and 0.1 % percentiles, latency, available engine data, compatibility and footprint (whether it runs inside the game process and what data it accesses).
- [ ] H1.5 Test B: gamescope, with the same metrics, including footprint.
- [ ] H1.6 Decision D2 recorded with the figures that support it.
- [ ] H1.7 Exclusion list of games with anti-cheat: StarVektor off by default in them (D9).

_Note: H1.1 covers one game on one machine (RX 9060 XT, 8 GB), with the idle-probe VRAM discrepancy between MangoHud and sysfs explained only for a loaded game. The fixed scenes (H1.3) and the hook tests (H1.4, H1.5) are still open._

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
