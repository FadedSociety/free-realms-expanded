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
   - Each item closes with a verdict: **PROVEN** (all gates ran and passed, including native where required), **PARTLY PROVEN** (say which gates are missing), or **NOT PROVEN**.
   - New test suites include one **canary**: a deliberate break that must make a test fail, which proves the tests can catch regressions.

   Both ideas come from Anthropic's code-modernization plugin.
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
| 1.0 | ✅ **Done 2026-10-15 (W75, lead-reviewed): [[Security Model]].** Twelve ordered must-haves: L1 to L9 block a LAN playtest, I10 to I12 block internet play. **Bare-internet UDP stays closed; remote friends join over an encrypted VPN.** **Security model first:** a threat model for exposing login and gateway beyond loopback (packet fuzzing and malformed input, login brute force, ticket replay, impersonating another session, flooding, save tampering, Studio exposure). Uses `security-reviewer` plus `secure-code-guardian`. | Sol → lead | `notes/Design/Security Model.md` reviewed; 1.1 to 1.5 cite it |
| 1.1 | Remote connections: a `--bind` option for non-loopback addresses on login and gateway, plus the matching launcher option | Lead | A second PC on the LAN reaches character select |
| 1.2 | **Design done 2026-10-15 (W76, lead-reviewed): [[Research/Multi Login Design]].** Native finding: the client's login packet carries only *Session* and *Fingerprint*, never a username or password. So authentication happens in the **launcher**: a credential broker checks Argon2id-hashed passwords and returns a 60-second one-use token, which becomes the client's Session. The design also has a peer session table (32 cap), one-use gateway grants and separate account/character id maps. **Changed 2026-10-15 (the owner): key pairs instead of passwords.** The launcher keeps a per-server Ed25519 private key; the server stores only the public key and verifies a signed challenge, so a malicious private-server host never receives a reusable secret. Crates approved: ed25519-dalek, getrandom, subtle. Build order: players.rs maps → account schema 3 → `login_auth` + `osfr-account` CLI → login and gateway wiring (lead). Login for many players: several concurrent sessions and accounts, simple per-account credentials, a real native test of the HMAC ticket path | Sol design → Luna build → lead | Two clients log in at once with different accounts |
| 1.3 | **Built 2026-10-15 (W80 + lead wiring), PARTLY: tests pass, native not run.** Other players now get the real name, look, active job and outfit in AddPc, the facing fix, and the movement flags (a flags-only change is relayed). Canary: `tests/multiplayer.rs` checks the names and the flags-only relay. Still (R): name ids, speed 10, wield type 0, guild/title/VIP. **Research done 2026-10-15 (W77, lead-reviewed): [[Research/AddPc Layout]].** 78 fields with native provenance. It found that our AddPc facing is sent as a quaternion where the client reads a direction (the same bug already fixed for NPCs), and that state-only movement relays are suppressed. Real AddPc: appearance, name, equipment and job from self data; keep the position relay's state bytes | Sol research (AddPc layout from the native map) → Luna | Native: the other player renders correctly and animates |
| 1.4 | Fix bugs the 10-15 audit found: social packets dropped without `--trace`; account-id/GUID collision; stale save on duplicate login | Lead | Regression tests plus a bot chat test with content loaded |
| 1.5 | Roles: player, builder and admin per account; `realm` cheats only for builders and admins | Luna (to a spec) → lead | A player account can't run `realm god`; the admin can |
| 1.8 | **Admission and packet hardening** (Security Model L6, L7): session caps (16 total, 4 per IP), handshake and credential rate buckets, per-family packet caps, a reassembly deadline, and removal of reachable panics; runs fuzz tests T1 to T3 and T9. | Luna (to spec) → lead | T1, T2, T3 and T9 pass; second bot unaffected |
| 1.9 | **Safe administration and audit** (L5, L8, L9): refuse `--dev-tools` and Studio on exposed servers; per-account action budget for chat, social and commands; a bounded security log without names, chat or tickets. | Luna → lead | T7, T8 and T10 pass |
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

- **2026-10-16, job outfits integrated:** a job unlocked by any path now gets its starter outfit, and activating a job whose outfit is completely empty equips its starter pieces (reusing owned pieces, no duplicates; pieces the catalogue lacks are skipped) (R, the owner's rule). New active play package round70 = round69 + W82 starter lists (14 jobs) + W85 Ninja/Warrior/Wizard attacks. Canary `tests/job_outfits.rs` (private, OSFR_PLAY_PACKAGE; fails on the old code). Kart Driver's jersey 13419 is missing from the catalogue until W88 loads W84's definitions. **W86b:** launcher now opens and draws (causes: missing libunwind.dll; a 2172-px banner over egui's 2048 texture limit). The UI below the banner still needs a Flash-portal skin pass. Suite 2112 passed.

- **2026-10-16 (early), worker batch W82-W87 reviewed and committed:**
  - **1.8 admission (lead), PARTLY:** caps 16 total / 4 per IP, 5 handshakes/IP/s, 5 s login deadline, per-session packet and byte budgets from the measured native peak (41 datagrams/s). Tests: 17th connection refused, never-logged-in peer dropped (both fail on the old gateway). Open: credential-failure limits (needs 1.2 wiring), T3/T9 fuzz and flood runs, refusal logging is a bounded summary only.
  - **1.2 key-pair login core (W83), PARTLY:** `login_auth` (strict Ed25519 over a server-bound one-use challenge; hashed one-use 60 s tokens; constant-time compare), account schema 3 with up to 4 public keys and a writer lock, `osfr-account` CLI. Lead-reviewed. Next: lead wires login and gateway; launcher key storage.
  - **2.1 job outfits (W82):** native job equipment rows are inventory *instance* ids; starter lists for 14 jobs reconstructed from Legacy (R), Adventurer/Card Duelist/Pet Trainer unresolved. **2.1 attacks (W85):** Ninja/Warrior/Wizard basic attacks staged and bot-proven on scratch gateways; not yet merged into the play campaign; attacks do not check the equipped weapon (Medium integrity gap).
  - **2.3 item catalogue (W84):** generator covers all 16,551 missing references and reproduces the 1,703 captured rows byte-exact; only 419/1,703 match when regenerated independently from Legacy/client data, so generated rows stay (R). Loading and on-demand 35/34 replies are not built.
  - **3.5 Rust launcher (W86):** egui/eframe + gilrs skeleton committed; builds only on the gnullvm target with llvm-mingw (installed in lab/tools, the owner-approved); the window does not draw yet (W86b investigating).
  - **Native map (W87):** 6 stuck shards unstuck, 23 labels changed; 57 corrections blocked by older evidence failing validation. Found the 35/34 item-id word (`008d7980`, `00a3da40`) and same-slot ability cancellation (`00a36a70`).

- **2026-10-15 (late night), Kart Driver job and workers:** the owner's missing clothes were traced to job 48 (Kart Driver): entering a race switched the job and leaving never switched back, and Kart Driver's native default outfit (4 items) is missing from our starter content. Fixed (the owner's rule, (R)): leaving a race, or reconnecting out of one, restores the previous job; Kart Driver stays only when chosen or required. Canary `tests/kart_gameplay.rs::private_leaving_a_race_restores_the_previous_job` (private package v17; fails on the old code). The older private kart test still fails on HEAD at its bare 41/108 check (pre-existing). Dispatched Sol workers W82 job outfits, W83 key-pair login core, W84 item catalogue, W85 Ninja/Warrior/Wizard attacks, W86 Rust launcher skeleton, W87 native map upkeep.

- **2026-10-15 (late night), native two-player run (1.6), PARTLY:** the owner saw the bot (Buddy) spawn and walk beside him: first native run with two characters in our world. Frida hooks (`native_hook.py --set add_pc`) confirmed the client parses our AddPc and creates the actor. Open: the bot's first spawn and its later disappearance were not seen; the owner had moved about 37 m away, and the cause is not yet known. **Server crash found and fixed:** a player whose game exits abruptly made Windows report ConnectionReset (10054) on the next receive, and the gateway exited, taking every player down. Receive now treats it as harmless (gateway and login). Canary: `tests/multiplayer.rs::a_vanished_client_does_not_stop_the_server` (fails with 10054 without the fix); suite 2089 passed.

- **2026-10-15 (late night), first native two-player attempt (1.6):** the bot could not enter the world. Two single-character assumptions found and fixed: the encounter system only accepted a character with GUID 1 (`EncounterEngine::supports_location`), and a basic attack only counted when it targeted GUID 1 (now the attacker's own GUID). Canary: `tests/encounter.rs::any_character_guid_may_load_and_basic_attack`; suite 2088 passed. the owner's controller feedback: left stick now strafes (Q/E) instead of turning; camera drags keep the pointer inside the game window.

- **2026-10-15 (late night):** Controller support for testing. The PC client has no gamepad driver (no XInput or DirectInput import; its gamepad and rumble types are shared PS3 code with nothing feeding them on PC). Added `scripts/launcher/gamepad_bridge.py`: XInput in, the client's own default keys and right-mouse camera out, only while the client window is in front; layout in `gamepad.json`, re-read live; `run-local-baseline.py --gamepad`. 6 mapping tests; not yet tried in game. For the Rust launcher (3.5), port it to `gilrs` (MIT/Apache, the owner's suggestion).

- **2026-10-15 (late night):** Item 1.3 wired: the gateway builds each player's AddPc look from the same identity, profile and inventory as their own self data, and relays the two movement bytes (flags-only changes too). Next is the native two-player run (1.6).

- **2026-10-15 (night):**
  - **Fixed, tested and pushed:** chat and social packets dropped without `--trace`; the coin store overwriting the live inventory; a duplicate login saving a stale copy.
  - **W75 security model and W77 AddPc research** were reviewed and accepted.
  - **Added items** 1.8 (admission and packet hardening) and 1.9 (safe administration and audit).
  - **Decision pending (the owner):** the `argon2` crate for password hashing.

- **2026-10-15 (later):** Installed 17 MIT skills for Claude and Codex (security, Rust, review, testing, operations, design; provenance in `lab/downloads/skills-20261015/PROVENANCE.md`). Added the security gates rule and item 1.0 (security model before any non-loopback exposure).

- **2026-10-15 (later):** Confirmed there is no player-facing launcher. Added 1.7 (minimal launcher in M1) and expanded 3.5 (full launcher). README rewritten; the banner art is being redone in real Free Realms promo style using client art as references.

- **2026-10-15 (later):** Git baseline committed. codebase-memory-mcp installed and indexed; its first query confirmed that nothing calls `FishingService`, matching the audit.

- **2026-10-15:** Plan created from a five-part audit (network, world/combat, quests/economy, minigames, worker history). The ledger was rewritten. Found and recorded: loopback-only servers, a one-session login, the AddPc stub, social packets dropped without trace, the coin-store overwrite bug, hollow quests, missing starter weapons and attacks, and no git history. Earlier today, built and verified: World Builder text and literal NPC names (run 20261016T011257Z), live world layer, enemy reach made tunable, suite at 2065 passed.
