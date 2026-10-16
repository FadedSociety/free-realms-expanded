---
type: index
updated: 2026-10-12
---

# Build Guides

One guide per feature of Free Realms. Each holds, in one place, every mapped client function that belongs to it across game logic, network, UI and physics, plus a written explanation of how the feature works. These are the documents to build the Rust server and the new client from.

Generated 2026-10-12 from `lab/native-map/run/map.sqlite` by `project/scripts/native_map/build_guides.py` (69641 accepted labels of 199405 functions; mapping is still in progress, so re-run the script to refresh the tables). Library and runtime guides are reference only: the rebuild uses Rust crates for those and only needs their data formats.

**Game content** (job quest lines, dungeons, NPCs and their quests) is in [[Content]].

## Gameplay

- [[Kart Racing]] — 319 functions (82 confirmed, 98 likely)
- [[Demo Derby]] — 236 functions (128 confirmed, 59 likely)
- [[Trading Card Game]] — 3412 functions (1463 confirmed, 903 likely)
- [[Chess]] — 42 functions (26 confirmed, 6 likely)
- [[Soccer]] — 206 functions (93 confirmed, 63 likely)
- [[Fishing]] — 216 functions (103 confirmed, 61 likely)
- [[Ship Combat (Pirates)]] — 309 functions (66 confirmed, 73 likely)
- [[Battle Mages]] — 88 functions (36 confirmed, 13 likely)
- [[FRTW Game]] — 282 functions (168 confirmed, 78 likely)
- [[Minigame Framework]] — 499 functions (173 confirmed, 198 likely)
- [[Housing]] — 229 functions (103 confirmed, 65 likely)
- [[Pets]] — 341 functions (150 confirmed, 95 likely)
- [[Mounts]] — 78 functions (28 confirmed, 32 likely)
- [[Quests and Objectives]] — 381 functions (131 confirmed, 171 likely)
- [[Combat and Abilities]] — 430 functions (177 confirmed, 105 likely)
- [[Character and Customization]] — 829 functions (540 confirmed, 181 likely)
- [[Items and Inventory]] — 332 functions (121 confirmed, 104 likely)
- [[Rewards and Collections]] — 478 functions (299 confirmed, 78 likely)
- [[Coin Store and Purchases]] — 1314 functions (685 confirmed, 309 likely)
- [[Trade]] — 520 functions (242 confirmed, 134 likely)
- [[Social (Friends, Guilds, Chat)]] — 1142 functions (593 confirmed, 283 likely)
- [[Matchmaking and Lobby]] — 459 functions (244 confirmed, 95 likely)
- [[Game Rules and State]] — 1607 functions (677 confirmed, 467 likely)

## World

- [[Zones and Terrain]] — 367 functions (177 confirmed, 83 likely)
- [[Actors and NPCs]] — 198 functions (67 confirmed, 63 likely)
- [[Movement and Pathfinding]] — 150 functions (60 confirmed, 50 likely)
- [[Physics]] — 334 functions (106 confirmed, 84 likely)

## Network

- [[Login and Account]] — 651 functions (327 confirmed, 129 likely)
- [[Gateway and Transport]] — 4762 functions (1992 confirmed, 1699 likely)
- [[Command System]] — 2873 functions (1492 confirmed, 812 likely)
- [[User Data and Storage]] — 852 functions (427 confirmed, 242 likely)

## Client Engine

- [[UI Framework (GFC and XML)]] — 5390 functions (2050 confirmed, 1697 likely)
- [[Scaleform and Flash UI]] — 596 functions (142 confirmed, 286 likely)
- [[Text and Localization]] — 820 functions (178 confirmed, 151 likely)
- [[Rendering]] — 2163 functions (871 confirmed, 548 likely)
- [[Animation]] — 1026 functions (228 confirmed, 307 likely)
- [[Audio]] — 421 functions (121 confirmed, 148 likely)
- [[Assets and Pack Files]] — 1828 functions (602 confirmed, 653 likely)
- [[Scripting (Lua)]] — 457 functions (96 confirmed, 180 likely)
- [[Client Core]] — 2448 functions (566 confirmed, 889 likely)

## Libraries and Runtime

- [[Containers and Utilities]] — 17623 functions (5290 confirmed, 6174 likely)
- [[C Runtime]] — 8824 functions (452 confirmed, 4956 likely)
- [[OpenSSL and Crypto]] — 359 functions (10 confirmed, 178 likely)
- [[zlib, XML and HTTP]] — 325 functions (84 confirmed, 126 likely)
- [[Unclassified]] — 3425 functions (28 confirmed, 535 likely)
