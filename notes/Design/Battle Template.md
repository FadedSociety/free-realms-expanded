---
type: design
task: W123
updated: 2026-10-16
status: PARTLY PROVEN
---

# Battle Template

This checklist covers Build Plan 2.7 and the content groundwork for 2.8. Battle 96 is the existing template, not new W123 work. Its full native loop is recorded in [[Reviews/Native Round5 050310]], run **184205**: wandering marker, start screen, Go, arena, summons, win, prize wheel, results and world return. This does not establish historical fidelity for its reconstructed rules or for a second battle.

## W123 content and evidence boundary

Two additional **reconstructed playable slices (R)** use battle 96's exact JSON schema. Neither original server battle IDs nor original roster/scene/reward bindings were recovered. Local IDs are deliberately distinct from the native combat Encounter 96 namespace and from `MiniGameData.txt`, whose id 96 names Rich Iron Mining. Do not infer combat encounter IDs from that table.

| Local battle id (R) | Client title / text id | Region (R placement) | Recommended band (R) | Enemy roster (R selection/count) | Rewards (R) | Evidence |
|---|---|---|---|---|---|---|
| 12301 | Bixies Gone Wild! / 75452 | Sanctuary / Queensfields; `[-726,-7.1,152]` | 1–2; difficulty 1 | 3 Bixie Soldiers; model 220, name 440711 | 25 coins and one of the four existing battle-96 prizes | Client locale 76503 describes a clearing and 76505 says defeat all bixies; Models row 221; [[Field Enemies]] regional context; wiki Battles revision 48953 gives difficulty 1 |
| 12302 | Band of Robgoblins / 42054 | Lakeshore / northeast Wilds; `[-540,0.5,1010]` | 3–4; difficulty 2; open entry | 3 Robgoblins; model 4, name 2232 | Same reconstructed reward policy | Client locale 42066/42068 gives the kill-all objective; Models row 5; wiki Battles revision 48953 gives difficulty 2; regional placement and plain-melee roster are R |

Both use existing arena **9600 / `sg_random_encounter_clearing`**, geometry 214, sky `sky_wilds24.xml`, arrival `[136,-0.5,155]`. The scene exists in extracted client data and works natively for battle 96. **Its assignment to these two battles is R.** Base enemy values are 140 HP and 3 damage; the existing loader applies its difficulty scaling. No new level gate or balance code is supplied.

The Bixies start picture borrows image **662**, whose shipped name is the Bixie Hive panel (R assignment, not a recovered Bixies Gone Wild picture). Band uses the named Band of Robgoblins image **4540**. Both markers reuse crossed-swords image **11970** (R choice).

The shared prize list is Battle Item Mystery Pack **10482**, Medium Energy Potion **3023**, Medic's Power Shard of Vitality I **75354**, and Small Health Potion **11002**. All four are present in the round70 runtime inventory catalogue. This is a runtime availability check, not recovery of either battle's original wheel. Per-job wheels, rarity weights, authentic stars and 2014 payouts remain unknown.

Exact source paths, SHA256 hashes, original locale line numbers/hash keys and resource rows are in `lab/reports/field-battles-w123/provenance.json`. No community source code was copied. The local wiki corpus is a retrospective reference; it does not constitute a frozen 2014 table. Live [Battles](https://freerealms.fandom.com/wiki/Battles) search agrees on difficulty 1 for Bixies and difficulty 2 for Band of Robgoblins; direct Fandom access was robots-blocked.

## Template checklist

1. **Establish identity and provenance.** Read the Ledger, newest native review, relevant Build Guides and the owner's notes. Resolve every numeric text id against the shipped locale through `Global.Text.<id>`, recording the hash key and line. A guide's `loc` number can be a hash, not a numeric wire id. Record recovered IDs separately from local IDs (R).
2. **Prove the required mechanics fit.** Identify the main goal, bonus goals, waves, friendly actors, spawners, chest/door actions and boss phases. The current field template completes a single goal after all relevant enemies die; a boss-marked roster has different completion semantics. Do not silently replace rescue, treasure, escort, ranged, teleport or destructible-object mechanics with killing generic actors. Defer an unsupported battle with an exact reason.
3. **Close the asset references.** Check model/name/image IDs, actual ADR/mesh/skeleton/animation availability, the arena `.gzne` and terrain, zone/geometry/sky values and safe arrival. Use the existing asset graph and readers. A named guide record with null `instance_scene_name` is research, not runnable content. Treat a plausible scene assignment as R until independently demonstrated.
4. **Reuse the runtime schema and route.** Copy battle 96's shape in `field_battles.json`; preserve 96 unchanged. Supply title, description, objective, difficulty, spawner, arena, enemies, rewards and provenance. Keep the existing limits: schema 1, at most 64 battles, 16 enemies per battle, 4 prizes and 128 KiB total. Give every actor a distinct GUID and positive local definition ID. Do not add a second encounter engine.
5. **Place the marker safely.** Use world terrain zone 0, not a regional content label. Check finite coordinates, terrain coverage/height and the full existing **5 m** wander disc. Reuse W102's **25-degree** terrain threshold and NPC, town and corridor exclusions, including the wander radius in buffers. Check separation from field enemies. These are conservative authoring rules (R); the raster does not prove water, rocks, trees, object collision or a traversable road route. Native inspection remains necessary.
6. **Use honest AI and reach.** Plain melee reuses the default preset and reviewed reach. Battle 96's skittish summoner, shielding, health-threshold small trolls and summon effects belong to that battle; do not copy those flags to arbitrary enemies. The owner's small-troll observation is video evidence; precise thresholds, scale, run/rest timings, shield and balance remain R. Never add invented invisible ranged damage to make an encounter work.
7. **Check entry and start screen.** Confirm proximity AddNpc and crossed swords, ordinary marker interaction **26/8**, details **41/114** and ready **41/107**. Pressing Go **41/108** must consume the pending battle and enter the existing instanced encounter route. Test noncombat refusal without losing the HUD. `required_combat_level` is currently not enforced: defer high-tier content rather than claim a functional gate.
8. **Prove arena entry and combat.** Observe **31/28**, ready/loaded, enemy spawn and HP, arena combat rules and objective. Kill every required enemy using ordinary **36/10** attacks and legal movement/cooldowns. Do not use `realm win`, god mode, overlevel commands, reduced fixture HP or kill-state edits as acceptance. Test summoning or other special phases separately when present.
9. **Prove the actual reward.** Verify terminal Won, dead required enemies, goal completion and **39/19**. Compare pre-win versus post-win wallet, inventory and XP; every advertised prize must exist in the **runtime** catalogue. Results/wheel packets alone do not prove a grant. Keep reward mutation atomic and check duplicate requests, full bags, losses and repeatable-win policy through the existing economy tests when changing those rules. W123 changes no economy code.
10. **Prove ordinary return and recovery.** Send results Close **26/42**, observe world return **31/12**, loaded/ready and correct world location. The entry packet **31/28** is not the return packet in the tested path. Verify active encounter cleared and reward retained. A separate reconnect test must preserve rewards and return an abandoned repeatable battle safely. Do not introduce a repeatable battle ID into opening-story reward validation.
11. **Freeze the evidence and review.** Run strict content check, preserve each bot script/receipt/log, content and binary hashes, placement checks and deliberate failing canaries. Bot acceptance pre-checks the route and state. Only lead-controlled native inspection proves visible marker/arena/AI, real reach, wheel animation and collision. Independent lead review and the native loop remain the completion gates for Plan 2.7/2.8.

## Runtime reuse and unresolved battles

Current source routes `Gameplay::load` through `EncounterEngine::load_field_battles`, registers spawners with proximity visibility, and handles Go through the existing `begin()` dungeon route. The loader appends each field battle to runtime dungeon content; it does not load the prose `data/content/dungeons/` research records as runnable arenas. Graph coverage at generation `2026-10-16T01:56:50Z` reported changed source metadata, so current targeted source reads superseded graph line numbers. The tested gateway binary is frozen separately; concurrent source changes require lead integration review.

Eight battles are deferred. They are listed here, not in `dungeons_unresolved.json`, which must match the dungeons' unresolved references exactly (`tests/content_dungeons.rs`); W123's original entries are kept in `lab/reports/field-battles-w123/dungeons_unresolved.w123-version.json`:

| Battle | Blocking binding or mechanic |
|---|---|
| Hooligan Brawling Club! | Captured travelers/rescue, original roster, arena and wheel |
| Treasure of the Bone Shaman! | Claim-treasure/chest/barrier goal, elite behaviour and scene |
| Robgoblin Adept Trouble! | Health-threshold teleport and healing |
| Explosive Robgoblins! | Proven dynamite projectile/attack pair and enforced level-13 gate |
| Hooligan Bullies! | Enforced level-13 gate, archer/elite attacks and original bindings |
| Robgoblin Creek! | Fish toss, exact fishmonger roster and arena/wheel |
| Fleetfoot Ninja! | Niobe teleport/punch/kick sequence and arena/wheel |
| Thugawug Sneak! | Rescue script and original arena/roster/prizes |

## Gates and limits

Runnable commands from `project`:

```powershell
python -I scripts/content_check.py --strict --out ../lab/reports/field-battles-w123/content-check.json
python -I ../lab/reports/field-battles-w123/scratch_gate.py
python -I ../lab/reports/field-battles-w123/evidence.py
```

The scratch content starts from `lab/reports/round70-native-20261016T061000Z/content` plus the authored file. W102's frozen NPC/zone/behaviour tables and pre-W116 quest/job tables isolate fallback from concurrent schema edits. The **scratch content's world spawn** is set to a checked 20 m marker approach; no existing save is edited. This focuses the bot gate on walking to the marker and the complete battle route. It does not prove a walk from a town. The unbound bot remains Player; Studio is disabled and no developer commands run. Only disposable loopback gateways are launched.

Retained failed probes distinguish an old-binary quest-schema failure, the correct refusal of developer commands for a non-admin, and an incorrect return-opcode expectation from content failures. No native game window was launched or controlled. Native visuals, object/water clearance, road access, authentic rosters/balance/rewards, per-job wheels and multiplayer instance ownership remain open. The two slices do not close Plan 2.8's five-battle/dungeon native gate.

Final W123 pre-checks: strict content check **0 errors / 20,198 repository warnings**; two bot exits **0**; both battles killed three enemies with ordinary attacks, awarded **25 coins / 50 Brawler XP / one prize**, and returned to zone 0 with no active encounter. The observed prizes were Small Health Potion **11002 x1** (12301) and Battle Item Mystery Pack **10482 x1** (12302). Duplicate-GUID and unavailable-runtime-prize canaries were rejected. Details and exact tested content/binary hashes: `lab/reports/field-battles-w123/gate-summary.json`, per-run `receipt.json` and `provenance.json`. The existing asset graph has no item:10482 node; its serialized runtime inventory definition and actual bot grant provide the availability proof instead. Model 4's graph also lists missing optional animations; native visual/animation closure remains open.

Ponytail: reused battle 96, W102's height/safety helpers, existing asset/locale readers and the bot/gateway. No Rust, dependencies, new gameplay engine, navigation system, reward system or unsupported boss mechanics were built.
