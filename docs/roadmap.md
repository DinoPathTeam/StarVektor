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
- [ ] H1.3 Three fixed scenes per evaluation game, played by a person or with the game's own benchmark mode (no input automation), on the unmodified game. Progress: 2 of 10 evaluation games (Skyrim Special Edition and Metro 2033 Redux), with Euro Truck Simulator 2 in progress. Three scenes (Helgen keep combat, Whiterun exteriors, Whiterun to Riverwood), three repetitions each from the same save, gamescope capped at 60 fps. Across repetitions the p99 frame time varied by 0.48 ms (combat), 0.09 ms (Whiterun) and 0.07 ms (Riverwood), and the p99.9 by 0.66, 0.16 and 0.10 ms; mean fps was 60.0 in all nine runs. Skyrim at 60 fps leaves the GPU mostly idle, so these ranges are a floor: they must be re-measured on a game that loads the GPU. Metro 2033 Redux is the chosen GPU-heavy game (store page checked: single-player, no anti-cheat or third-party DRM listed). At 1080p it is not demanding, so the load was raised to 2560x1440 internal (gamescope) with 4A TSR, Shading Rate 2.0x, tessellation Very high, VSync off, output 1920x1080 at 60 Hz and the `performance` CPU governor; the game was relaunched before each repetition and played by hand from the same save. In every run the GPU load averaged 99 to 100 %, the GPU clock stayed between 2666 and 2677 MHz and its temperature between 51 and 64 °C, without throttling. Four scenes were measured. `tower-1440` (end of the prologue, five repetitions, first 180 s of each): mean fps 76.7 to 78.9, median frame time 12.91 to 13.13 ms, p99 spread 1.92 ms and p99.9 spread 1.89 ms, with no trend across runs and no run excluded. Three chapter 1 scenes (three repetitions each), delimited by in-game events and not by the clock, with load screens and cutscenes left out: `cap1-hunter-combate` (from entering Hunter's room until the last enemy falls, 197 to 201 s) had 77.9 to 79.4 fps, a median spread of 0.52 ms, a p99 spread of 1.14 ms and a p99.9 spread of 0.92 ms; `cap1-exposicion` (from the end of the loading, with the camera free, until just before talking to the NPC that leads to the trolley, 130 to 138 s) had 100.0 to 100.8 fps, a median spread of 0.04 ms, a p99 spread of 0.23 ms and a p99.9 spread of 0.12 ms; `cap1-vagon-combate` (from after the cutscene with the Dark Ones until the last enemy falls after the flamethrower, 164 to 168 s) had 81.7 to 83.5 fps, a median spread of 0.33 ms, a p99 spread of 0.09 ms and a p99.9 spread of 0.08 ms. The combat in Hunter's room is not deterministic, and it is the scene with the widest tails. In the second repetition of the three chapter 1 scenes the VRAM in use was about 250 MB higher than in the other two (3.55 GB against 3.25 to 3.33 GB) with no visible effect on the frame times; it is a system-wide figure, so another process may explain it. The routes were played by hand. The repeatability tolerance is set per game, from the spreads that game shows across its own baseline repetitions, and it is fixed before StarVektor is compared against that baseline. Games differ in genre and engine (Skyrim, an RPG on the Creation Engine, against Metro 2033 Redux, a first-person shooter on the 4A Engine), so a single value for all of them would be wrong. A difference between StarVektor and the baseline that is smaller than the tolerance of that game and scene is not reported as an improvement or a regression. For now the tolerance of each scene is the spread observed in its repetitions (the Skyrim and Metro 2033 Redux figures above). These values are provisional: three repetitions tend to understate the real spread, as the `tower-1440` scene showed when its spread doubled from the second to the third run, so they are revisited with more repetitions. H1.3 is not ticked until the other games have their own. Euro Truck Simulator 2 (version 1.61x under Proton, single-player, no mods) is the second game being measured. Its traffic and surroundings are never the same twice, so there are no fixed scenes: the unit is a whole trip, recorded as one file and converted without segments, and also read in windows of 60 s with the context (road type, light, traffic) noted by hand. The tolerance comes from repeating the same trip from the same save, with the same in-game hour, weather, truck and camera. Pilot: three repetitions of a reference trip of about 7 minutes (in-game start at 10:00, clear weather, same truck, interior camera, graphics settings unchanged, FPS limiter off, `gamemoderun` and the `-mm_*` memory options kept fixed). The trips lasted 6.8 to 7.4 minutes and had 104.5 to 107.5 mean fps, a median frame time of 8.81 to 8.95 ms, a p99 of 14.60 to 15.80 ms (spread 1.20 ms) and a p99.9 of 19.01 to 20.94 ms (spread 1.93 ms). The fps profile along the route repeats closely from run to run (about 145 to 165 fps in the first minute, falling to 70 to 90 fps in the last three), and each run had one 57 to 64 ms frame at about the same point of the trip (145 to 163 s). Unlike Metro 2033 Redux, this game is not limited by the GPU: the mean GPU load was 43 to 46 % and the GPU clock about 2010 MHz, so the frame time is decided by the CPU and the engine. This is a different regime from the Metro scenes and is kept as a separate condition. The pilot is not a full baseline: the trip mixes fps above and below the monitor's 100 Hz, so whole-trip figures hide where the regulator would act, and the longer A to B and B to A trips at different hours are still to be recorded. The first long trip is Prague to Ostrava, 323 km, starting at 11:27 in game in all three repetitions and arriving between 15:40 and 15:50, clear weather without changes, with a different truck than the pilot (530 hp, 14 t cargo, new profile without mods) and the same graphics settings and launch parameters; it is a separate condition from the pilot. Three repetitions from the same save lasted 16.7 to 18.0 minutes of real time (about 15 times faster than the game clock). Whole trip: mean fps 106.4 to 112.7, median frame time 8.97 to 9.06 ms (spread 0.09 ms), p99 16.58 to 17.05 ms (spread 0.47 ms) and p99.9 19.16 to 20.53 ms (spread 1.37 ms). Read by stretch, the motorway (without the first minute and the last three) had a p99 of 12.17 to 12.55 ms (spread 0.38 ms) and a median spread of 0.35 ms, while the last three minutes, the arrival in the city, ran at 74.7 to 79.5 fps with a p99 of 18.35 to 19.52 ms (spread 1.17 ms) and a median spread of 1.0 ms; the city is the heaviest part and is not limited by the GPU. Each run had a freeze in the same stretch of the route (around minute 7 to 8, about 13:15 to 13:30 in game, on the motorway, where only the road was visible, with a junction toward a city still a few real minutes away; random events happened around 13:10 and the data cannot tell whether they are related): 13 frames above 30 ms, up to 125 ms, in the first run, 5 frames up to 123 ms in the third and only a 31 ms frame in the second. The p99.99 (25.4 to 47.9 ms) and the maximum (34 to 125 ms) are dominated by these isolated freezes and are not repeatable, so they are not used to set the tolerance; the number of frames above 100 ms is reported as an event count instead (5, 0 and 2). During the freeze of the first run the GPU clock fell to idle while VRAM, RAM and swap did not change, so its cause is unknown but it was not the GPU. The first run averaged 257 fps in its first minute, unlike the other two (101 and 111 fps); the cause has not been confirmed, and that minute is left out when the stretches are compared. The GPU load was 41 to 43 % and the CPU load 26 to 27 % in all three runs. The tolerance of this trip is the spread observed in these repetitions, provisional like the others. Other candidates are listed under `evaluation_candidates` in `data/sources.toml`.
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
