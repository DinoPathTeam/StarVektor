# Publisher agreement review

A record of what the license agreements (EULAs) of the publishers whose games might be used to **evaluate** StarVektor say. It is the team's reading, not legal advice. Texts can change and vary by region and edition: before relying on one, the current version is re-read.

> **Early stage.** This record grows as publishers are reviewed. Anything not listed here has not been reviewed.

**Last review:** 2026-10-26.

## How to read this

- No agreement read mentions capture or frame generation. Only Epic's mentions AI (training of generative AI, see below). Silence neither allows nor forbids explicitly.
- What each agreement regulates are **acts**; intent does not count. That is why the project's defense is to describe precisely what it does and does not do, see [usage-statement.md](usage-statement.md).
- Statuses: **No clause against** (per our reading), **Broad clauses** (assessed with caution and, ideally, written permission), **Excluded for now** (not used), **Unverified**.

## Platforms

| Platform | Relevant clauses | Status |
|---|---|---|
| Steam (Subscriber Agreement, §2.G) | Forbids copying, reproducing, modifying, disassembling, decompiling, reverse engineering, creating derivative works and removing proprietary notices from the contents and services, except with permission or as applicable law allows. Personal, non-commercial use. | No clause against image observation. §4.B (unauthorized third-party software) still to be read in full. |
| Epic Games Store (EULA and licensed products terms) | Reverse engineering, modifying, decompiling or creating derivative works; unauthorized programs to gain an advantage in online modes; using the products beyond what is expressly permitted. **Forbids using code or content extracted from the products as training data for a generative AI program** (or as input used to train one). Defines the term broadly, and excepts programs that operate only on the original content (for example, image upscaling). As observed, the same text appears in the Epic games reviewed. | No clause against as long as we do not train on third-party games (D4 and rule 7 of the [data policy](data-policy.md)). We do not rely on the "original content only" exception: frame generation creates new visual content. |

## Publishers

| Publisher / game | Relevant clauses | Status |
|---|---|---|
| Bethesda (Fallout 4 and similar) | Forbids commercial use, distribution, copying, network/online use, reverse engineering and modifying the software. Nothing about third-party software, data mining or recording. | No clause against |
| Square Enix (FFVII Remake) | §3: commercial use, copying or storing the product, reverse engineering, circumventing protections, and creating programs that imitate the product's data or functionality. | No clause against, under the usage statement (no frames stored; we imitate a function of the display, not of the game) |
| 505 Games (Death Stranding) | Forbids extracting information and reverse engineering. Nothing about overlays or AI. | No clause against |
| Ubisoft | Forbids trainers, macros and cheat programs; the game may monitor memory for interacting third-party programs. | Offline games only |
| CD PROJEKT RED (Cyberpunk 2077, The Witcher 3) | Forbids reverse engineering and "cheats", defined as software that interacts with or affects the game "in any way", including programs that gather information by reading memory. Also using similar software alongside theirs and extracting the game for text and data mining. | Broad clauses: evaluate only under the usage statement and, in the future, request permission |
| Pearl Abyss (Crimson Desert) | Art. 5 of the Black Desert EULA: reverse engineering, unauthorized programs, interfering with technical measures, data mining and creating programs that imitate the game's data or functionality. | Broad clauses. **The Crimson Desert EULA is unverified**: confirm that the text read is its own |
| Warner Bros. Games | §2.1(g): forbids unauthorized software "designed to interact with the software in any way", including to gather information. | Excluded for now, until we have arguments and permission |
| Sony (PlayStation PC) | Text written for consoles; application to PC unclear. | Excluded by project decision |
| Remedy (EULA on Steam, id 870780) | Licensor: Remedy Entertainment Plc; the text does not name the game. §2.3(iv): do not "harvest information" from the software or its content. §2.3(v): do not disassemble, decompile, decrypt, hack, emulate, exploit, reverse engineer or modify. §2.3(vi): do not circumvent technical protection measures. Personal, non-commercial use. Nothing about overlays, recording or AI. Its website has no separate EULA: the privacy policy and terms and conditions cover only the services and the site. | No clause against, under the usage statement. Point to watch: §2.3(iv); we do not harvest game information, we measure our own output. Confirm on Steam which game the id corresponds to. |
| SCS Software (Euro Truck Simulator 2) | No EULA of its own was located on its website, blog or project page. The installation files contain only license notices for third-party libraries (Bullet, DirectStorage, HIDAPI, Dear ImGui, MoltenVK, OpenXR, NVAPI, PhysX, wxWidgets, Xiph.Org). | No own EULA located: the Steam agreement would apply. This is not the same as "does not exist"; pending confirmation with the publisher |
| All other publishers | Not reviewed. | Unverified |

## Anti-cheat

The main risk in online games is operational rather than contractual: an anti-cheat can treat an injected layer as an intrusion. Valve's documentation warns that VAC can produce false positives with overlays and recording software. That is why the scope is single-player and offline (D9). The compatibility status of each anti-cheat on Linux changes, and is checked per title before testing.

## Next steps

- Read §4.B of the Steam agreement in full.
- Confirm the Crimson Desert EULA and locate the Control one.
- Review the remaining publishers that enter the evaluation set.
- With the Phase 1 results, request permission for numerical comparison from publishers with broad clauses, or offer the software for their internal development use.
