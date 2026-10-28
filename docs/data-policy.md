# Data policy

StarVektor learns from **results and from frames from permitted sources**; it never stores content, code or binaries from games or SDKs. Not storing is not enough: terms of use may forbid the act of capturing and training, so **the source is decided before capturing**.

## Categories

| Category | What goes in | Use |
|---|---|---|
| Permitted | Our own scenes rendered in an open engine (Godot, Blender); public datasets with an open license; games with explicit permission | Training and evaluation |
| Evaluation only | Commercial games (Control, Cyberpunk 2077, The Witcher 3 2026 version, Crimson Desert, Skyrim, The Medium, Hell is Us, Metro: Last Light Redux) | Compare quality and cost; never train |
| Comparison reference | Output of FSR 3, AFMF 2, DLSS FG and OptiScaler FG | Measure whether StarVektor does better or worse; never as a training target |
| Forbidden | SDK code or binaries, assets extracted from games, games whose EULA forbids capturing or training, captures made with FG active as input | Never |

A commercial game's EULA is reviewed before capturing it.

## Pipeline rules

1. Training ground truth is the real frame at 120-240 fps (intermediate frames are discarded), never the output of an existing generator.
2. Every sequence carries a provenance record: origin, version, license or permission, date, who captured it and the file hash.
3. The list of sources is versioned in this repository (`data/sources.toml`); a sequence without an entry does not enter the dataset.
4. Before publishing a model, a memorization test is run: given similar inputs, it must not reproduce training scenes.
5. Telemetry logs contain no images or game data; the capture dataset is separate and optional.
6. The first prototype is trained only on our own scenes; commercial games are reserved for evaluation.
7. No model is trained or fine-tuned on images from third-party games, not even on the user's machine (there is no online learning on what is being played).
8. No frame from commercial games is stored: the image is processed in memory, discarded, and only numerical metrics are kept.
9. Evaluation is limited to single-player, offline games. Status of each publisher in [publisher-review.md](publisher-review.md).

## Automatic verification

`scripts/check_sources.py` (also in CI) checks that each source declares id, kind, use, license, origin, date and `frame_generation = "off"`; that games with permission state it; and that no evaluation-only game or reference generator appears as training.

**Limit:** the script cannot detect a commercial game declared as something else. That is covered by human review when each source is added.

## Contributing data

If you want to contribute scenes, open a proposal stating origin and license. Only sources that comply with this policy are accepted.
