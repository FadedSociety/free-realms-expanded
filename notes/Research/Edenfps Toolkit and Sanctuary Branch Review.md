# Edenfps Asset Toolkit and Sanctuary Branch Review

Reviewed October 9, 2026 Pacific. Both requested repositories were downloaded into the private lab, inventoried and hashed before analysis. This is a source review with a selected asset-reader probe; Sanctuary was not built, deployed or tested with a native client.

## Pinned downloads

| Reference | Branch / commit | Local directory | Tracked files inventoried |
| --- | --- | --- | --- |
| [FR ADR Toolkit](https://github.com/edenfps/fr-adr-toolkit/tree/5acfdf8d6006fa6f43d1d3605f076a6be7393e72) | main / `5acfdf8d6006fa6f43d1d3605f076a6be7393e72` | `../lab/references/fr-adr-toolkit-20261009` | 100 |
| [Sanctuary PS3](https://github.com/edenfps/Sanctuary-PS3/tree/d1c747f64653264cd52b854efd6c17de6d418a78) | ps3 / `d1c747f64653264cd52b854efd6c17de6d418a78` | `../lab/references/Sanctuary-PS3-20261009` | 766 |
| [Sanctuary PC 2009](https://github.com/edenfps/Sanctuary-PS3/tree/b1182d671d8fcc1e44efa1909e6b3cc5056df8e6) | pc-2009 / `b1182d671d8fcc1e44efa1909e6b3cc5056df8e6` | `../lab/references/Sanctuary-PC2009-20261009` | 763 |

Download commands, from `lab/references`: `git clone --depth 1 https://github.com/edenfps/fr-adr-toolkit.git fr-adr-toolkit-20261009`; `git clone --depth 1 --no-single-branch https://github.com/edenfps/Sanctuary-PS3.git Sanctuary-PS3-20261009`; then `git -C Sanctuary-PS3-20261009 worktree add --detach ../Sanctuary-PC2009-20261009 origin/pc-2009`. LFS smudge was disabled. Existing references were preserved.

## What helps this build

**Asset restoration and later custom content:** the toolkit reads ADR actor references/animation slots, DME geometry, DMA materials and DSK bone names/bind matrices. The full download also includes OBJ/glTF importers, mesh/material/actor writers, templates and a GUI. `fr_formats/writers.py::write_adr_preserve_template` retains actor sections beyond mesh/skeleton references; its comment identifies player FX/skintone sections that must survive replacement. These are useful community tools to try before authoring a Rust format reader or replacement cosmetic.

This is the same commit already used in [[Community Asset Validation]], rather than new format research. All 21 prior pinned source/documentation Git blobs match the old SOURCE.json hashes. The fresh checkout uses CRLF, so raw checkout hashes differ; canonical Git hashes and normalized text match. The earlier reference contained only selected files; the new clone retains the full converter source.

**Executed read-only probe:** hash-verified six supplied-client input files, copied them into the report, then loaded five ADR/DME/DMA/DSK files using unmodified community readers. Chatdy: DMOD v3, 4,256 vertices, 6,317 triangles, 92 bones/bind poses and 32 animation slots. Warpstone: two chunks, 998 vertices and 978 triangles. Positions are finite, indices are in bounds, and original/copy hashes remain unchanged. DDS was hash-preserved, not decoded in this probe. No new preview, conversion, animation playback or native override was tested.

Toolkit limits remain: its ordinary DMOD reader accepts 36/52-byte layouts, not our cave's 44-byte layout; use the already validated ForgeLightToolkit reader for that case. DSK hierarchy is retained as raw bytes. GR2 animation authoring is an open research item. Unweighted OBJ input binds to ROOT; converter availability does not establish working rigged cosmetics. Toolkit LICENSE is AGPL-3.0. Downloaded templates and script bundles stay lab-only.

**Version-specific compatibility:** Sanctuary's PC-2009 branch contains alternate movement, zone, POI, welcome and purchase serializers. The PS3 branch adds XML-over-HTTP account/creation/login routes, PS3 zone and profile serialization, and world initialization. Our supplied Windows client version is `1.910.1.530630`; the fork explicitly separates that version from PC-2009 and PS3. Do not transplant older/platform packet layouts into our accepted Windows path without capture parity.

**Loading diagnostics:** `analysis/world-loading-checkpoints.md` traces PS3 world-ready gates, opcode 14, preload subopcode 26, actor readiness and opcode 10 before loading-overlay dismissal. This is useful diagnostic methodology and corroborating cross-platform evidence. Its function addresses belong to a different executable, and its rendering-thread stall/patch remains an upstream hypothesis. The repository's reported logs and IDA databases were not supplied as verified local evidence.

**Fishing protocol clues:** both branches contain registration/casting/bobber/school/reel packets and handlers under `Sanctuary.Packet/BaseFishingPacket` and `Sanctuary.Gateway/Handlers/BaseFishingPacket`. Activity startup uses character-state flag `0x400000` and comments that zoning can disable fishing. This is a candidate trace to compare against our actual client. Reel-in always returns a caught Trout (`FishId=418`); registration has numerous Unknown fields and fixed sample schools. It does not establish historical fishing rules, reward authority or persistence.

**Kart and teleport clues:** `MiniGameStartGamePacketHandler` sends race state 0, a 630-byte largely zero vehicle blob, then state 5; registration and vehicle-data handlers mainly log. This is an experiment to cross-check, not a functioning race. Zone teleport resolves a POI, derives heading, updates server position and sends location update. World teleport sends a target player's location without equivalent server-state update in that handler, so it is not a complete authority model to copy.

## Housing and completeness

`ClientHousingPacketEnterRequestHandler` has its zone transfer entirely commented out. SetEditMode echoes an update. Instance data, fixture lists, permissions and build-area serializers are valuable comparison material, but contain Unknown fields and a TODO FactoryInstance. No working house-entry/editor/save loop was found in the reviewed handlers. Likewise, Mining payload handling only answers OnConnectMsg in the reviewed fork. These downloads do not replace our already richer Rust gameplay state machines or complete housing integration.

Compared with our pinned Sanctuary `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, PC-2009 has 561 common C# paths (444 identical normalized text, 117 changed) and 119 additional C# paths; PS3 has 561 common (440 identical, 121 changed) and 121 additional. These counts establish substantial overlap, not behavior equivalence.

Neither downloaded Sanctuary branch contains a tracked LICENSE/COPYING file. The README's old MIT badge/comment is insufficient: our pinned upstream Sanctuary includes AGPL-3.0. Preserve ancestry/provenance and clarify fork licensing before copying implementation into a distributable build.

## Evidence and next use

Evidence: `%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/edenfps-reference-review-20261009T182429Z/receipt.json`; three inventories, `source-overlap.json`, `probe-inputs.json`, `asset-reader-probe.json` and private input copies in the same directory.

Priority: use asset tools for bounded restoration/custom-content experiments; compare fishing wire definitions with native captures; retain loading/version research as corroboration. Housing still needs original-client enter/place/move/remove/save/reload acceptance. No production source, dependency, balance, saves, release or native client state was changed by this review.

The separate [[SOE Protocol Rust Crate Assessment]] is also retained. At the owner's explicit request its qualified findings were submitted as a Codex memory extension note.
