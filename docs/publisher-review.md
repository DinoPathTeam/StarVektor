# Publisher agreement review

A record of what the license agreements (EULAs) of the publishers whose games might be used to **evaluate** StarVektor say. It is the team's reading, not legal advice. Texts can change and vary by region and edition: before relying on one, the current version is re-read.

> **Early stage.** This record grows as publishers are reviewed. Anything not listed here has not been reviewed.

**Last review:** 2026-10-27.

## How to read this

- No agreement read mentions capture or frame generation. Only Epic's mentions AI (training of generative AI, see below). Silence neither allows nor forbids explicitly.
- What each agreement regulates are **acts**; intent does not count. That is why the project's defense is to describe precisely what it does and does not do, see [usage-statement.md](usage-statement.md).
- Statuses: **No clause against** (per our reading), **Broad clauses** (assessed with caution and, ideally, written permission), **Excluded for now** (not used), **Unverified**.

## Platforms

| Platform | Relevant clauses | Status |
|---|---|---|
| Steam (Subscriber Agreement, §2.G, §4, §6) | §2.G: forbids copying, reproducing, modifying, disassembling, decompiling, reverse engineering, creating derivative works and removing proprietary notices from the contents and services, except with permission or as applicable law allows. Personal, non-commercial use. §4.B: forbids creating or helping others create cheats (defined as functions that give an unfair competitive advantage in multiplayer), disabling or interfering with anti-cheat software, altering the operation of Steam or of the contents and services without Valve's express authorization, and using unauthorized third-party software to interact with or control their processes or user interface. §4.C: forbids scripts, bots, macros or other systems not controlled by humans to interact with Steam contents and services (the examples concern accounts, statistics, rewards and moderation). §6: covers user-generated content uploaded to Steam; it does not apply, as we upload nothing. The English and Spanish texts were compared and are identical. | No clause against, with two points to watch. (1) "Altering the operation" of the contents (§4.B): our reading is that observing the presented image does not alter the game's operation; an in-process Vulkan layer is closer to the line than gamescope, which feeds the footprint criterion of D2. (2) Automation (§4.C): test scenes are played by a human or use the game's own benchmark mode, never injected input or macros. |
| Epic Games Store (EULA and licensed products terms) | Reverse engineering, modifying, decompiling or creating derivative works; unauthorized programs to gain an advantage in online modes; using the products beyond what is expressly permitted. **Forbids using code or content extracted from the products as training data for a generative AI program** (or as input used to train one). Defines the term broadly, and excepts programs that operate only on the original content (for example, image upscaling). As observed, the same text appears in the Epic games reviewed. | No clause against as long as we do not train on third-party games (D4 and rule 7 of the [data policy](data-policy.md)). We do not rely on the "original content only" exception: frame generation creates new visual content. |

## Publishers

| Publisher / game | Relevant clauses | Status |
|---|---|---|
| Bethesda (Fallout 4, Skyrim Special Edition and similar) | Forbids commercial use, distribution, copying, network/online use, removing proprietary notices, and (clause g) reverse engineering, decompiling, disassembling or modifying the software, in whole or in part. Nothing about third-party software, cheats, overlays, recording, data mining or AI. Mods are governed by a separate Editor EULA. Skyrim Special Edition text (North America and Latin America version) read in full on 2026-10-27. | No clause against, under the usage statement (we do not modify the software; evaluated offline and without mods) |
| Square Enix (FFVII Remake) | §3: commercial use, copying or storing the product, reverse engineering, circumventing protections, and creating programs that imitate the product's data or functionality. | No clause against, under the usage statement (no frames stored; we imitate a function of the display, not of the game) |
| 505 Games (Death Stranding) | Forbids extracting information and reverse engineering. Nothing about overlays or AI. | No clause against |
| Ubisoft | Forbids trainers, macros and cheat programs; the game may monitor memory for interacting third-party programs. | Offline games only |
| CD PROJEKT RED (Cyberpunk 2077, The Witcher 3) | Forbids reverse engineering and "cheats", defined as software that interacts with or affects the game "in any way", including programs that gather information by reading memory. Also using similar software alongside theirs and extracting the game for text and data mining. | Broad clauses: evaluate only under the usage statement and, in the future, request permission |
| Pearl Abyss (Crimson Desert) | Art. 5 of the Black Desert EULA: reverse engineering, unauthorized programs, interfering with technical measures, data mining and creating programs that imitate the game's data or functionality. The Crimson Desert text was read and is the same as Black Desert's (and Black Desert Mobile's). Breach can lead to a ban from all of the publisher's games. Crimson Desert is single-player, has no anti-cheat and uses Denuvo Anti-Tamper, which protects the integrity of the executable; we do not touch the executable. | Broad clauses: evaluate later, only under the usage statement and, ideally, with written permission. |
| Warner Bros. Games | §2.1(g): forbids unauthorized software "designed to interact with the software in any way", including to gather information. | Excluded for now, until we have arguments and permission |
| Sony (PlayStation PC) | Text written for consoles; application to PC unclear. | Excluded by project decision |
| Remedy (Control Ultimate Edition; EULA on Steam, id 870780) | Licensor: Remedy Entertainment Plc; the text does not name the game. §2.3(iv): do not "harvest information" from the software or its content. §2.3(v): do not disassemble, decompile, decrypt, hack, emulate, exploit, reverse engineer or modify. §2.3(vi): do not circumvent technical protection measures. Personal, non-commercial use. Nothing about overlays, recording or AI. Its website has no separate EULA: the privacy policy and terms and conditions cover only the services and the site. | No clause against, under the usage statement. Point to watch: §2.3(iv); we do not harvest game information, we measure our own output. The id corresponds to Control Ultimate Edition on Steam. |
| SCS Software (Euro Truck Simulator 2) | No EULA of its own was located on its website, blog or project page. The installation files contain only license notices for third-party libraries (Bullet, DirectStorage, HIDAPI, Dear ImGui, MoltenVK, OpenXR, NVAPI, PhysX, wxWidgets, Xiph.Org). | No own EULA located: the Steam agreement would apply. This is not the same as "does not exist"; pending confirmation with the publisher |
| Bloober Team (The Medium) | The Steam page links no game-specific EULA, only the Steam Subscriber Agreement. Single-player. | No own EULA located: the Steam agreement applies. This is not the same as "does not exist" |
| Hell is Us | The Steam page links no game-specific EULA, only the Steam Subscriber Agreement. | No own EULA located: the Steam agreement applies. Same caveat as above |
| Deep Silver, a division of Plaion (Metro: Last Light Redux) | The Steam page links no game-specific EULA, only the Steam Subscriber Agreement. Single-player. The Epic copy falls under Epic's general terms. | No own EULA located: the Steam and Epic agreements apply. Same caveat as above |
| Tripwire Interactive (Maneater) | EULA on Steam. Forbids decompiling, modifying and reverse engineering the game, and "cheats, hacks, mods, mutators, exploits or anything else not intended in the game" (conduct clause, enforced with online co-op/multiplayer bans). Mods must not modify executables. Nothing about overlays, recording, data mining or AI. | No clause against. Not selected for evaluation for now |
| CI Games (Sniper Ghost Warrior Contracts 2) | EULA on Steam. Forbids using "any other similar software together with" theirs, cheats defined as software that "interact with or affect" the game, and unauthorised third-party programs that collect information about the game. States the game has no DRM. The first Contracts game shows no game-specific EULA on Steam. | Broad clauses. Excluded by project decision |
| Koei Tecmo (Nioh: Complete Edition, Epic text) | Forbids reverse engineering and otherwise analyzing the product, combining it with any other software, interrupting Koei Tecmo's smooth operation, and "any other acts that Koei Tecmo deems to be inappropriate". | Very broad clauses. Excluded by project decision |
| 2K (Tiny Tina's Wonderlands) | Steam links a third-party EULA that was not read. | Excluded by project decision |
| Playstack (Mortal Shell) | The Steam page links no game-specific EULA. | Discarded by project decision |
| All other publishers | Not reviewed. | Unverified |

## Anti-cheat

The main risk in online games is operational rather than contractual: an anti-cheat can treat an injected layer as an intrusion. Valve's documentation warns that VAC can produce false positives with overlays and recording software. That is why the scope is single-player and offline (D9). The compatibility status of each anti-cheat on Linux changes, and is checked per title before testing.

## Next steps

- Review the remaining publishers whose games enter the evaluation set (for example Assassin's Creed Odyssey, Ghostrunner, Ghostwire: Tokyo, Guardians of the Galaxy, Shadow of the Tomb Raider, Wolfenstein: The New Order), reading the text of each edition used.
- Wait for the reply of SCS Software before using Euro Truck Simulator 2; without a reply, it is dropped.
- With the Phase 1 results, request permission for numerical comparison from publishers with broad clauses, or offer the software for their internal development use.
