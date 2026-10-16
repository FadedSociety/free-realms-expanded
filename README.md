<p align="center">
  <img src="docs/images/banner-collage.png" alt="Free Realms Expanded" width="100%">
</p>

<h3 align="center">An open-source revival of Free Realms, with tools to build new content in the running game.</h3>

<p align="center">
  <a href="#what-works-today">What works</a> ·
  <a href="#the-toolkit">Toolkit</a> ·
  <a href="#roadmap">Roadmap</a> ·
  <a href="#getting-started">Getting started</a> ·
  <a href="#project-principles">Principles</a>
</p>

---

**Free Realms Expanded** brings back Sony Online Entertainment's 2009 family MMO, which shut down in 2014. It has two parts:

- **A new server written in Rust.** The original game client connects to it unmodified and plays as if the official servers were back.
- **A toolkit for building new content inside the running game.** You can walk the world, place NPCs, write quests and run live events. Everything is saved and survives restarts.

The goal is not only to restore Free Realms but to let the community **expand** it. The gaps left by the original server can be filled with new quests, dungeons, NPCs and events, built by walking around in the world.

> **Status:** active development, local and private for now. The original client plays real content against our server today. Multiplayer, content breadth and public hosting are being built (see [Roadmap](#roadmap)).

## What works today

Everything below has been checked in the **original game client**, not just in tests. Run ids and evidence are in the [Build Ledger](notes/Design/Build%20Ledger.md).

| Area | Working in the real client |
|---|---|
| **Getting in** | Account login, character select, and world entry (all 46 startup messages built by the server) |
| **The world** | About 580 NPCs across the starting regions, with dialogue; ground-height-aware movement and enemies |
| **Quests** | Offer, accept, track and turn in; NPC dialogue camera; quest chains with talk and kill objectives |
| **Combat** | Field enemies; job scaling to level 20; Medic abilities with level-locked skills |
| **Battles** | *Troll Summoner Madness!*: a wandering battle marker, start screen with prizes, an instanced arena, a summoner that calls minions and runs away to rest, the loot wheel, and back to the world |
| **The opening** | The full 17-stage tutorial, including Cooking, Smelting and Forging and the first cave dungeon |
| **Kart racing** | Full three-lap races with boosts and rewards (started with a dev command for now) |
| **Building** | Placing NPCs in the running world, with names from text we author or typed in directly (the client shows both) |

The automated test suite has more than 2,000 tests.

## The toolkit

Building an MMO's worth of content takes good tools. The server includes a developer **Studio**: a local web page plus in-game chat commands.

- **Live:** your character, nearby actors, and one-click commands (`level`, `heal`, `god`, `battle`, `win`).
- **Map:** a shaded relief map of the world built from the client's terrain. Click to teleport onto the ground, or click to place an NPC.
- **Content:** edit battles, enemies, abilities, quests and NPCs as tables, then hot-reload them without restarting.
- **Inspector:** a graph of how the game's files connect (items, models, effects, animations and abilities). "Ice Axe" shows its frost effect, swing trail and both special attacks.
- **Fight:** a balance simulator per job and level, plus a fight log.
- **Packets, Saves, Report, Client log:** a live message inspector with send, save states, a content checker, and the client's own errors.
- **Bot client:** a scripted player for automated end-to-end tests, such as logging in, entering a battle arena and winning.
- **Behaviour presets:** enemy AI (speed, reach, attack rate, run-and-rest) is data you can tune while the game runs.

**World Builder** (in progress) lets trusted builders place NPCs, write quests and text, link quests together, and send an event such as a scavenger hunt to everyone online. Builds persist in a live-world layer, and finished content goes through review before it becomes part of the main world. Design: [World Builder](notes/Design/World%20Builder.md).

<p align="center"><img src="docs/images/banner-clean.png" alt="" width="100%"></p>

## Roadmap

The living plan is [Build Plan](notes/Design/Build%20Plan.md). The first big target is a **playtest night**: about ten players on one server together.

1. **M0, foundations:** a verified state ledger, version control and code memory. ✅
2. **M1, two players, one world:** remote connections, many logins at once, seeing and chatting with other players, roles and permissions.
3. **M2, an hour of real play:** every combat job fights, a full item catalogue, real quest chains, roaming enemies, five starter dungeons, and shops.
4. **M3, playtest readiness:** shared encounters and groups, minigames from their normal entrances, World Builder events, a one-click launcher for friends, and a 10-bot stress test.
5. **M4, playtest night.**

## Getting started

> Running the server currently needs a private workspace: a legally obtained copy of the original client plus derived reference data. A one-command setup for contributors is planned (Build Plan item 3.5).

The repository holds the server, tools, scripts and research notes. No game assets are included.

```text
prototype/osfr-tools/   Rust server: gateway, login, encounter engine, quest engine, Studio, bot, terrain
  src/bin/              osfr-gateway, osfr-login, osfr-bot, osfr-terrain
  data/content/         Game content as JSON: quests, NPCs, battles, jobs, abilities, behaviours
  studio/               Studio web page (served by the gateway with --dev-tools)
scripts/                Launcher, content checker, terrain and locale builders, research tools
notes/                  The design and research vault (open in Obsidian)
tests/                  Python checks for scripts and coverage
```

Build and test (Windows, with the project's private Rust toolchain):

```powershell
.\scripts\cargo.ps1 test --release --manifest-path .\prototype\osfr-tools\Cargo.toml
```

## Project principles

- **The client is the source of truth.** Behaviour comes from what the original client sends, expects and displays. A function map of the native client covers about 99,000 labelled functions. Fan wiki and community emulator data are used only where the client is silent, and the client wins when they disagree.
- **Reconstructions are marked.** No original server code survives, so every server rule we had to invent (balance, drop tables, AI) is tagged **(R)** in code and notes.
- **Native acceptance.** A feature is done only when it works in the original client: "tests pass" is not enough.
- **Clean-room code.** Community projects are used as references only. Their code is never copied, which matters for AGPL-licensed projects in particular.

## Documentation

- [Build Plan](notes/Design/Build%20Plan.md): the living plan
- [Build Ledger](notes/Design/Build%20Ledger.md): the verified state of every system
- [Studio Toolkit](notes/Design/Studio%20Toolkit.md) and [World Builder](notes/Design/World%20Builder.md)
- [Build Guides](notes/Build%20Guides/Build%20Guides.md): one guide per game system, built from the native client map
- [Project journal](MODLOG.md)

## Legal

Free Realms Expanded is a **non-commercial fan project**. It is not affiliated with, endorsed by or sponsored by Daybreak Game Company or Sony. Free Realms and related names are trademarks of their respective owners. This repository contains no original game assets or client files. It holds original server code, tools and research notes.
