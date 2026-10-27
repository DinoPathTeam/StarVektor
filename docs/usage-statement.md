# Usage statement

> **Early stage.** StarVektor is under development and its code is incomplete. What follows describes the design the project commits to, not behavior that is already implemented: today the program does not hook into any game. Any copy of the code is partial.

This document describes, in terms of **concrete acts**, what StarVektor does to games and what it does not do. It is not legal advice: it is the technical description the project commits to respect. If the program's behavior changes, this document is updated before the code.

## What it is

Free software (MIT license planned) that improves presentation smoothness on monitors without VRR by generating its own frames. It imitates a function of the **display** (variable refresh rate), not a function of any game.

## Scope

- **Single-player, offline** games.
- It is not used in online games or in games with anti-cheat. In them StarVektor is off by default (exclusion list, milestone H1.7 of the [roadmap](roadmap.md)) and is never tested on online accounts.
- Comparison tests are run only with single-player games, under the conditions in [publisher-review.md](publisher-review.md).

## What it does

1. Observes the image the system already presents, through the system's public interfaces (Vulkan, Wayland/X11 on Linux).
2. Generates its own frames from that image.
3. Measures its own timings (frame time, cost per pass, video memory) and numerically compares the result against a reference.
4. Processes the image in memory and discards it. From commercial games only **numerical metrics** are kept; no frame.

## What it does not do

- It does not read or write the memory of the game process.
- It does not modify or patch the game's binaries, libraries or files, and does not read its data files.
- It does not extract or copy assets, models, textures, audio or text.
- It does not intercept or emulate the game's network traffic or protocols.
- It does not bypass, disable or interfere with protections (DRM, anti-cheat, access control).
- It does not automate the game or give an advantage of any kind in matches.
- It does not use commercial games to train models (see [data-policy.md](data-policy.md)).
- It does not use the output of other frame generators as a training target.
- It does not train or fine-tune any model on images from third-party games, not even on the user's machine. What is kept are numerical metrics and the marks the user makes by hand, without images.
- It has no commercial purpose: the software is free.

## Integration levels

| Level (`integration_level`) | What it uses | On third-party games |
|---|---|---|
| `image_only` | Only the presented image | Allowed; this is the black-box mode |
| `upscaler` | Data the game hands to its upscaler (depth, motion vectors), by intercepting the public API | Not used without the rights holder's permission. Reserved for integration done by the game's own developer |
| `engine` | Engine data, through the SDK | Only when the game's developer integrates it into their own engine |

When a developer integrates the SDK into their game, StarVektor stops being third-party software for that game and almost all restrictions on external programs no longer apply.

## Footprint of each hook

The decision between a Vulkan layer and gamescope (D2 in [design.md](design.md)) includes measuring the **footprint**: whether the code runs inside the game process and what data it accesses. The results will be recorded here. Note: gamescope also installs a Vulkan layer inside the game, so "outside the process" describes only our code.

| Hook | Runs in the game process | Data it accesses | Status |
|---|---|---|---|
| Own Vulkan layer (observation-only mode) | Yes | Presented image and presentation timings | To be measured (H1.4) |
| Gamescope | Our code, no | Final image from the compositor | To be measured (H1.5) |

## For studios and publishers

- StarVektor is free and open source. A studio can integrate it into its own engine or use it internally during development, at no cost.
- If a rights holder asks us to stop using their game in comparisons or to remove a mention, we will comply.
- To propose a collaboration or a comparison permission, open an issue in the repository.
