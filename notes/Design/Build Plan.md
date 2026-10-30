---
type: plan
started: 2026-10-15
updated: 2026-10-27
---

# Build Plan

**Last refreshed 2026-10-27.** The single living plan from **2026-10-15** onward. The verified current state of every system is in [[Build Ledger]], and this plan only schedules work against it. *How* work gets built (the enforced rules, briefs, where new code goes, build order and dormancy) is in [[Build Playbook]]. Where the code lives is in [[Project Map]]. The public version of "what is unfinished" is `ROADMAP.md`; this plan keeps the same order of areas.

Task ids (T, F, L, D) refer to integrated tasks: the id is in the commit subject (`Autopilot <id>:`), and most merged tasks also have a report in `notes/Autopilot/<id>.md`. An id that appears in neither is still in the maintainers' private queue.

**Rules:**

1. **Verify before you plan.** Before scheduling or claiming a gap, check the code (what is routed), the newest native review or run note, and the Ledger. Notes go stale; the 10-13 ledger did, and the 10-17 one did too.
2. **Update after every milestone, integration or native run.** Tick the item, update the [[Build Ledger]] row, and keep this file a plan rather than a diary: per-task detail belongs in `notes/Autopilot/`, and the journal is `MODLOG.md`. The same goes for [[Current Progress]], which is older than both.
3. **Every task has gates:** runnable checks that prove it is done, such as a test name, a bot script, or a native check with a run id. "Tests pass" alone is not a gate. A module that isn't routed isn't done.
   - Each item closes with a verdict: **PROVEN** (all gates ran and passed, including native where required), **PARTLY PROVEN** (say which gates are missing), or **NOT PROVEN**.
   - New test suites include one **canary**: a deliberate break that must make a test fail, which proves the tests can catch regressions.

   Both ideas come from Anthropic's code-modernization plugin.
4. **Security gates** (the owner, 2026-10-15: "make sure it's actually secure"). Every task that touches the network, logins, passwords, roles, saves or file paths:
   - is built with the `secure-code-guardian` skill;
   - passes a `security-reviewer` audit before integration, with findings fixed or logged;
   - starts from a threat model.

   Security files are lead-only (the Playbook names the enforcement). Studio and dev tools stay loopback-only and admin-only. Nothing is exposed beyond this PC except a private or VPN address under the rules in [[Security Model]]; bare-internet UDP stays closed.
5. **Look and feel:** project visuals follow the 2008–09 Flash-portal art direction (see the README banner). The `taste-skill` house style bans gradients, shadows and ornaments, which conflicts with that look. Use it only for plain pages such as docs and Studio; use `redesign-skill` audits for launcher readability.
6. **Workers:** builders run one task at a time through the autopilot pipeline in [[Build Playbook]] (brief, builder, integrator with the full suite, independent review, lead review, native check). Workers own only the paths named in their task and never commit to a remote. The lead integrates, wires routes, does all security work and runs native checks. Native checks come first in each working session, so fixes are not stacked on unchecked fixes.
7. **Publishing:** this repository is public (rules in `AGENTS.md`). Nothing from `lab/`, no client files, no decompiler output, no secrets, no personal names or home paths in notes.

## Goal: playtest night (M4)

You and about 10 friends log into our server from your own PCs and play together for an evening:

- create a character and walk the starting regions with real NPCs;
- pick any combat job and level it through free-roaming enemies, field battles and about 5 dungeons;
- do the first chapters of the job quest lines and a few minigames;
- keep the coins and items you earn across logins;
- see each other, chat and group up;
- an admin runs a live scavenger hunt.

**Where it stands (2026-10-27):** every item above has code behind it and most have tests. The real client has been run only one at a time (plus a scripted second player), never two real clients on two PCs. A realm can be started only from the maintainers' private startup package. Those two gaps, not features, are what stand between today and M4.

## M0: Working foundations (done)

- [x] Audit the true state and write the [[Build Ledger]] (2026-10-15), refreshed 2026-10-17 and 2026-10-27.
- [x] This plan, and the [[Build Playbook]] (2026-10-18): the enforced rules, the spine ratchet, the deletion phase (D001 to D010, merged 2026-10-18 to 10-20) and dormancy slice 1.
- [x] **Version control** (2026-10-15), and the move to a **public** repository (2026-10-26: publishing rules L064, README and ROADMAP L065, MIT or Apache-2.0 plus `NOTICE` L066, AGPL-3.0-or-later since 2026-10-29 L069, launcher split out L067).
- [x] **Code memory** (codebase-memory graph, re-indexed after every integration) and the worker preamble.
- [x] **Stale notes brought up to date** (2026-10-15 to 10-27): `AGENTS.md`, [[Project Map]], the Ledger and this plan.

## M1: Two players, one world

Gate: two real clients on two PCs see each other move with real appearances, chat, and group. A bot alongside a native client passes first.

| # | Work | Status (2026-10-27) | Gate |
|---|---|---|---|
| 1.0 | [[Security Model]]: twelve ordered must-haves (L1 to L9 block a LAN playtest, I10 to I12 block internet play) | Done 2026-10-15 | Reviewed; later items cite it |
| 1.1 | Remote connections: `--allow-lan` on login and gateway (private or VPN addresses, account-store mode, no dev tools) | Built. PARTLY PROVEN: a bot passed broker login, handoff and world entry over a VPN-range address (2026-10-22) | A second PC reaches character select: NOT PROVEN |
| 1.2 | Multi login: launcher-held per-server key pairs, signed challenge, one-use 60 s token ([[Research/Multi Login Design]]) | Built and wired. PARTLY PROVEN: broker tests and the bot; the real client was run through native login in dev mode and through the HMAC handoff (2026-10-22), never with the launcher's key | Two clients log in at once with different accounts: NOT PROVEN |
| 1.3 | Other players' real look, name, job and outfit in AddPc | PARTLY PROVEN: a real client drew a bot from our AddPc (2026-10-21) | The other player renders and animates in two real clients: NOT PROVEN |
| 1.4 | Fix bugs the 10-15 audit found (social drops, id collision, stale save on duplicate login) | Done 2026-10-16, tests | PROVEN |
| 1.5 | Roles: player, builder, moderator, admin; `realm` commands by role | Done. Moderation (mute, kick, ban) and admin move added 2026-10-22 | PROVEN for the role lock (2026-10-21 19:30Z); moderation PARTLY PROVEN (tests only) |
| 1.6 | Native two-player run | PARTLY PROVEN: client plus bot (2026-10-15, 2026-10-21) | Two real clients: NOT RUN |
| 1.7 | Friend launcher | Replaced by the Rust launcher in its own repository; `scripts/launcher/friend_launcher.py` stays as a LAN helper | A clean second PC goes from launcher to character select: NOT PROVEN |
| 1.8 | Admission and packet hardening (caps, rate buckets, budgets, fuzz tests) | Built, tests only: `app_fuzz`, `transport_fuzz`, `login_broker`. No outside review yet | PARTLY PROVEN |
| 1.9 | Safe administration and audit (action budgets, bounded security log, dev tools refused on exposed servers) | Built, tests only | PARTLY PROVEN |
| 1.10 | **Reproducible setup for outside testers:** the gateway needs a startup snapshot and reference packages built from the original client and captured traffic; make a setup that builds them from the player's own client, and decide what the launcher bundles | Not started. Decision pending (see "Waiting on the owner") | A clean machine builds the packages and starts a realm: NOT PROVEN |

## M2: A real hour of play

| # | Work | Status (2026-10-27) | Gate |
|---|---|---|---|
| 2.1 | All six combat jobs fight | Built; Brawler and Medic fought natively, with Brawler level-ups (2026-10-21). Archer, Wizard, Warrior and Ninja: tests only | Native: each job kills an enemy: PARTLY PROVEN. Native six-job pass queued (L026) |
| 2.2 | Job abilities per level | 24 skills for six jobs, bars fit the client's four buttons, toolbar label follows the weapon; unlock levels for some jobs are (R) | PARTLY PROVEN |
| 2.3 | Item catalogue (generated definitions, 35/34 on demand) | Loaded (58,835 ids when last counted); a dragon, a boombox and a potion displayed natively; pets did not | PARTLY PROVEN |
| 2.4 | Quests that are real: hide hollow ones, real objectives and gates | 93 of 678 records enabled; zone quest imports and enable rounds landed in part through 2026-10-26 (for example T661, T663, T723 to T732); the rest of the zones are queued | PARTLY PROVEN: one chain was walked natively (2026-10-21) |
| 2.5 | Objective events reach the quest engine | Done for talk, kill, collect, gather, craft, minigame, area, reach and job level; deliver-item, minigame-result, pet-trick and use-item queued | PARTLY PROVEN: talk seen natively |
| 2.6 | Free-roaming enemies with respawns | Camps for levels 1 to 20, home leash, level scaling, tunable damage | PARTLY PROVEN: seen natively while a Medic and a Brawler levelled up to level 6 (2026-10-21); higher bands tests only |
| 2.7 | Dungeon template: one new battle or dungeon end to end | The template is written ([[Battle Template]]) and eleven arena battles are built on it; *Troll Summoner Madness!* (battle 96) was seen natively start to finish | A native full loop of a newly built battle: NOT PROVEN |
| 2.8 | Five starting-region dungeons and battles | Ten more battles built from the template (tests only) | Native loop per battle: NOT PROVEN |
| 2.9 | Vendors and coin store | Shop windows seen; coin-store buy request answered natively; sell-back at one tenth | Native buy and sell at an NPC vendor: NOT PROVEN |

## M3: Playtest readiness

| # | Work | Status (2026-10-27) | Gate |
|---|---|---|---|
| 3.1 | Shared world: one set of field enemies for everyone; per-group arena instances | Open-world enemies are shared (tests). A grouped battle shares an instance but not the enemies (T738 queued) | Two clients fight the same summoner: NOT PROVEN |
| 3.2 | Minigames without dev commands | Kart from an NPC seen natively; mining and harvesting nodes, fishing, soccer, chess, derby tests only; fishing failed natively twice | Native win and payout per activity: NOT PROVEN |
| 3.3 | World Builder for admins: quest authoring, broadcast, scavenger hunt | Broadcast, live events, hunt, coin placement built (tests only); no quest authoring or review step | An admin's hunt quest is offered to a second player natively: NOT PROVEN (L029) |
| 3.4 | Shippable content: a playtest bundle from frozen packages ([[Playtest Bundle]]) | Bundle builder and gates exist | A clean checkout plus the bundle starts the full game: NOT PROVEN (needs 1.10) |
| 3.5 | Full launcher (Rust) | Its own repository, <https://github.com/FadedSociety/free-realms-launcher> | A fresh PC installs, patches, survives a killed download, and plays: NOT PROVEN here |
| 3.6 | Stress test: 10 bots playing scripted loops for an hour | Harness runs from a bundle (T431). Last figures 2026-10-16: 10 bots, 10 minutes, 0 errors | The 60-minute run and a broker-login run at this tip: NOT RUN (L027) |

## M4: Playtest night

Run it, collect bug reports and field notes, and feed them back into this plan. Needs 1.1, 1.2, 1.6, 1.10, 3.1 and 3.6 first.

## Next, in order

The order follows `ROADMAP.md` (areas by how many players each touches), with one change: the native re-check comes first.

1. **Native pass over merged-but-unseen fixes.** Re-run in the real client: pet adoption and the Pets window (T772), the fishing hole and Fish Finder (T776), town-greeter explainers (T773), Inspect (T401), the paid revive (T402), the Battles finder (T399), the crafting-station click (T400), Take Me There from the quest panel (T404), leaving a kart race (T392), the journal's quest list (T623), and the Card Duelist start order. Fix the review blockers filed against T772 and T773 (F792, F793) first. Each result goes into the Ledger.
2. **Two real clients on two PCs** (M1 gate, L025) and **reproducible setup** (1.10). Then the stress run on the tip (L027).
3. **Group fights share an arena** (T738), so two players can fight one summoner.
4. **Quests, dialog and the living world.** More zone quest rounds and evidence-built quests; multi-line talk chains and quiz retries (T543); the Job Browser's Start quest for every trained job (T542); the remaining NPC greetings; real zone ids and regions for every row, teleport by region, the "Now entering" banner and dungeon zones (T754 to T758); world population (critters, plants, props, patrols).
5. **Shops and the Marketplace.** Fill every NPC shop (T774, T781), missing item pictures (T768), the Marketplace's Sort By and Hot Items (T769), vendor buys counting for quest item goals (T794), then the item-source builds (vendors and coin shops first, then Marketplace, prize wheels, collections, drops) once the zone work is done.
6. **Pets.** Marketplace pets (T486), keeping the pet across warps (T487), grooming and petting (T488), tricks (T489), terrain step checks (T490), clothing (T530), idle life (T531), the happiness speed boost (T532), treasure (T533). Pet and Take Me There movement also need **collision and walk routes** (T534, T535, T554).
7. **Combat, jobs and dungeons.** Native pass for the four jobs not yet seen (L026); native loop for each field battle; one real dungeon scene, starting with Misty Mountain ([[Research/Dungeon Instances]], T748); prize wheel contents need evidence.
8. **Travel and the map.** Overhead and minimap icons (T555), spinning warpstones and Warp Watcher interaction.
9. **Fishing and minigames.** The nibble and tug flow (T611) after the native re-check; farm plots as the entrance to the regional Harvesting games (T553); the daily wheel (T560, T561); the real activity catalogue (T559); Snowhill copper veins (T547).
10. **Seasonal events.** Seasonal NPCs on their event clock (T775), joining the dance-off (T538), placed cakes and the Mystery Cake (T556 to T558), silly string and snowballs (T569 to T571).
11. **Housing.** Fixture placement (T783), then the housing directory (T562, T563), then house visits beyond entry, customization and farm lots.
12. **Card game.** Answer the client's cancel and log connections (T782), fix the start order the 2026-10-27 run exposed; the rules and duels then need evidence (see `ROADMAP.md` section 10).
13. **Builder tools.** Authored item definitions (T567), live collection-node authoring (T566), quest authoring, a review step that promotes finished content, then the public sandbox.
14. **Public play.** Only after items 1 to 3 and the security review: the encrypted transport in [[Public Transport]], hosting, and a database for saves.

Standing work between items: keep the hubs shrinking (the spine ratchet), keep tests reading tunable numbers from data (queued audit T645 to T652), and keep the suite fast (T721, T655).

## Later (not scheduled)

The remaining dungeons, guild features, Treasure War, Battle Mages, ship combat, a native Rust 3D editor whose renderer becomes a replacement Rust client, a database for saves, public hosting, and restarting the native function map (last recorded 830 of 901 shards, 2026-10-16).

## Waiting on the owner

- How outside testers get the startup reference package and the client login key file (item 1.10). Today both exist only in the private lab; the launcher can run a bundled server but a downloaded server cannot start without them.
- Freezing field enemies that no player is near (a behaviour change; [[Build Playbook]] §4).
- The previous plan's other waiting items (power-up key, fishing cap, full-bag policy, kart gates, merging to `main`) have since been decided. Ride prices stay (R) and tunable.

## Refresh log

- **2026-10-27, full refresh (documentation task L068):** Ledger, this plan and the Project Map were rewritten against tip `0db6412d`. The 2026-10-15 to 10-17 update log was removed from this file to keep it a plan; it is in the file's git history (last version before the refresh: commit `50f8e2f0`, 2026-10-18), and the per-task detail from 2026-10-17 on is in `notes/Autopilot/`. Not verified in this refresh: the full test suite, any real-client behaviour newer than the 2026-10-27 02:56Z run, and the queue order (the ids above come from the private queue as of that date).
