---
type: plan
started: 2026-10-15
updated: 2026-10-15
---

# Build Plan

The single living plan from **2026-10-15** onward. The verified current state of every system is in [[Build Ledger]], and this plan only schedules work against it.

**Rules:**

1. **Verify before you plan.** Before scheduling or claiming a gap, check the code (what is routed) and the newest native review. Notes go stale; the 10-13 ledger did.
2. **Update after every milestone, integration or native run.** Tick the item, add a line to the [[#Update log]], and update the [[Build Ledger]] row. The same goes for [[Current Progress]].
3. **Every task has gates:** runnable checks that prove it is done, such as a test name, a bot script, or a native check with a run id. "Tests pass" alone is not a gate. A module that isn't routed isn't done.
4. **Security gates** (the owner, 2026-10-15: "make sure it's actually secure"). Every task that touches the network, logins, passwords, roles, saves or file paths:
   - is built with the `secure-code-guardian` skill;
   - passes a `security-reviewer` audit before integration, with findings fixed or logged;
   - starts from a threat model.

   Studio and dev tools stay loopback-only and admin-only. Nothing is exposed beyond this PC until M1's threat model ([[Security Model]]) is written and reviewed.
5. **Look and feel:** project visuals follow the 2008–09 Flash-portal art direction (see the README banner). The `taste-skill` house style bans gradients, shadows and ornaments, which conflicts with that look. Use it only for plain pages such as docs and Studio; use `redesign-skill` audits for launcher readability.
6. **Workers:** Sol (`gpt-6.1-sol`, about Astra level) does research, design and review. Luna (`gpt-6-luna`) does tightly scoped builds. Run 6–10 at a time, sized to what the lead can review and integrate. Workers own only the paths named in their task. The lead integrates, wires routes and runs native checks.

## Goal: playtest night (M4)

You and about 10 friends log into our server from your own PCs and play together for an evening:

- create a character and walk the starting regions with real NPCs;
- pick any combat job and level it through free-roaming enemies, field battles and about 5 dungeons;
- do the first chapters of the job quest lines and a few minigames;
- keep the coins and items you earn across logins;
- see each other, chat and group up;
- an admin runs a live scavenger hunt.

## M0: Working foundations (now)

- [x] Audit the true state and rewrite the [[Build Ledger]] (2026-10-15).
- [x] Write this plan.
- [x] **Version control** (2026-10-15). First commit `ffef740` (5,458 files). Commit at every milestone and integration. It is pushed to the **private** GitHub repo https://github.com/FadedSociety/free-realms-expanded. Push after every commit. Going public is the owner's call, because the content carries client-derived text and Legacy-derived data.
- [x] **Code memory** (2026-10-15). `codebase-memory-mcp` 0.12.0 is installed for Claude Code, Codex and VS Code, with project `cartoonmmo` indexed (57,693 nodes, 168,877 edges). Use `trace_path` (inbound callers) to check whether a module is wired before planning work. Re-index after big changes with `codebase-memory-mcp cli index_repository --repo-path <project> --name cartoonmmo`.
- [x] **Worker preamble** (2026-10-15, `lab/runs/luna/prompts/preamble.md`). Add the current toolkit (Studio API, bot client, terrain, behaviours, content checker, asset graph, World Builder, `realm` commands), the gate rule, and "read Build Plan and Build Ledger first, not older notes".
- [x] **Brought stale notes up to date** (2026-10-15; originals in `lab/snapshots/notes-20261015/`): AGENTS.md (the captured-init and search-crash lines), Tasks.md, Current Progress top section, and MODLOG entries for 10-13 and 10-14.

## M1: Two players, one world

Gate: two real clients on two PCs see each other move with real appearances, chat, and group. A bot alongside a native client passes first.

| # | Work | Owner | Gate |
|---|---|---|---|
| 1.0 | **Security model first:** a threat model for exposing login and gateway beyond loopback (packet fuzzing and malformed input, login brute force, ticket replay, impersonating another session, flooding, save tampering, Studio exposure). Uses `security-reviewer` plus `secure-code-guardian`. | Sol → lead | `notes/Design/Security Model.md` reviewed; 1.1 to 1.5 cite it |
| 1.1 | Remote connections: a `--bind` option for non-loopback addresses on login and gateway, plus the matching launcher option | Lead | A second PC on the LAN reaches character select |
| 1.2 | Login for many players: several concurrent sessions and accounts, simple per-account credentials, a real native test of the HMAC ticket path | Sol design → Luna build → lead | Two clients log in at once with different accounts |
| 1.3 | Real AddPc: appearance, name, equipment and job from self data; keep the position relay's state bytes | Sol research (AddPc layout from the native map) → Luna | Native: the other player renders correctly and animates |
| 1.4 | Fix bugs the 10-15 audit found: social packets dropped without `--trace`; account-id/GUID collision; stale save on duplicate login | Lead | Regression tests plus a bot chat test with content loaded |
| 1.5 | Roles: player, builder and admin per account; `realm` cheats only for builders and admins | Luna (to a spec) → lead | A player account can't run `realm god`; the admin can |
| 1.6 | Native two-player run: client plus bot, then two clients | Lead | Review note with run ids |
| 1.7 | **Minimal friend launcher** (moved up from M3, 2026-10-15). Set up a client copy, apply our overlay (world text locale, settings, server address), then launch. There is no launcher today: `run-local-baseline.py` is a dev harness that needs the private `lab/`. | Luna → lead | A clean second PC goes from launcher to character select |

## M2: A real hour of play

| # | Work | Owner | Gate |
|---|---|---|---|
| 2.1 | All six combat jobs fight: basic attacks for Ninja, Warrior and Wizard; starter weapons for those three | Sol (ids from client evidence) → Luna | Native: each job kills an enemy |
| 2.2 | Job abilities per level for every combat job (Medic: Target Vitals 5, Vitamins 10, Shock Paddles 15; others from the guides) | Sol → Luna | Unit tests plus a native ability bar per job |
| 2.3 | Item catalogue: definition layout from the 1,649 captured samples, generated definitions (Legacy item data is allowed as a starting point per the 10-13 decision; client evidence wins), on-demand 35/34 replies | Sol → Luna → lead | Byte-exact round trip of the captured rows; native: a non-captured item granted and shown |
| 2.4 | Quests that are real: hide the ~639 quests with no objectives; write objectives and rewards for the starting-region chains; enforce level and job gates (`player_level` is hard-coded to 1) | Sol design → Luna content → lead | Every offered quest has at least one objective; native run of one chain |
| 2.5 | Objective events reach the quest engine: collect, gather, craft, minigame, area and reach | Lead | Unit tests per kind plus one native collect quest |
| 2.6 | Free-roaming enemies in the starting regions using behaviour presets, with respawns | Sol (placement from guides and Legacy spawns) → Luna | Native: enemies roam and respawn |
| 2.7 | Dungeon template: one new battle or dungeon end to end, with the checklist written down | Lead | Native full loop, like battle 96 |
| 2.8 | Five starting-region dungeons and battles built from the template | Sol designs → Luna builds → Sol review → lead | Native loop per dungeon (the bot can pre-check) |
| 2.9 | Vendors and coin store: fix the stale-inventory overwrite, act on 66/43 buys, reply on refusals | Lead / Luna | Native: buy and sell at a vendor, coins correct after a quest |

## M3: Playtest readiness

| # | Work | Owner | Gate |
|---|---|---|---|
| 3.1 | Shared world: one set of field enemies for everyone; per-group arena instances (wire the `world::World` coordinator; drop the fixed instance 0) | Sol design → lead | Two clients fight the same summoner |
| 3.2 | Minigames without dev commands: kart from its NPC or menu; mining and harvesting win and pay out (wire the lifecycle adapters); world crafting stations | Luna → lead | Native win and payout per activity |
| 3.3 | World Builder for admins: quest authoring, broadcast, scavenger-hunt event ([[World Builder]] steps 4–6) | Lead | Native: an admin's hunt quest offered to a second player |
| 3.4 | Shippable content: freeze activity content from `lab` packages into a playtest bundle | Lead | A clean checkout plus the bundle starts the full game |
| 3.5 | **Full launcher (Rust).** Lessons from Free Realms Legacy's launcher, whose patching visibly fails with "Failed to download 22 files":
- **Patching:** a SHA-256 manifest, resumable downloads with retries and a mirror, and a "verify and repair" button.
- **Client:** fetch the community client release (licence check first) rather than hosting it ourselves, then apply our small overlay.
- **In the window:** live server status and player count from the gateway; a "What's New" panel fed by World Builder events; login handoff; window-size settings (today's client opened larger than the screen).
- **Art:** generated in Free Realms promo style, matching the client's own green UI buttons. | Sol design → Luna → lead | A fresh PC installs, patches, survives a killed download, and plays |
| 3.6 | Stress test: 10 bots playing scripted loops for an hour | Lead | No crashes, saves intact, tick time logged |

## M4: Playtest night

Run it, collect bug reports and field notes, and feed them back into this plan.

## Later (not scheduled)

The remaining dungeons, housing (missing scene assets), pets, mounts (including the community "Mounts and Pets" update, licence check first), fishing, soccer, chess, derby, the card game, the guild, trade and matchmaking routes, a native Rust 3D editor (whose renderer becomes the Rust client), a database for saves, public hosting, and restarting the native map (6 stuck shards to reset).

## Update log

- **2026-10-15 (later):** Installed 17 MIT skills for Claude and Codex (security, Rust, review, testing, operations, design; provenance in `lab/downloads/skills-20261015/PROVENANCE.md`). Added the security gates rule and item 1.0 (security model before any non-loopback exposure).

- **2026-10-15 (later):** Confirmed there is no player-facing launcher. Added 1.7 (minimal launcher in M1) and expanded 3.5 (full launcher). README rewritten; the banner art is being redone in real Free Realms promo style using client art as references.

- **2026-10-15 (later):** Git baseline committed. codebase-memory-mcp installed and indexed; its first query confirmed that nothing calls `FishingService`, matching the audit.

- **2026-10-15:** Plan created from a five-part audit (network, world/combat, quests/economy, minigames, worker history). The ledger was rewritten. Found and recorded: loopback-only servers, a one-session login, the AddPc stub, social packets dropped without trace, the coin-store overwrite bug, hollow quests, missing starter weapons and attacks, and no git history. Earlier today, built and verified: World Builder text and literal NPC names (run 20261016T011257Z), live world layer, enemy reach made tunable, suite at 2065 passed.
