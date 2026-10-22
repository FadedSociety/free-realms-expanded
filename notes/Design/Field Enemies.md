---
type: design
task: W102
updated: 2026-10-16
status: PARTLY PROVEN
---

# Field Enemies

W102 supplies the **content/design step of Build Plan 2.6**, not its native roaming/respawn acceptance. Twelve always-on actors are appended to `prototype/osfr-tools/data/content/field_enemies.json`. The original four quest-3030007 Hooligans are preserved as parsed records. `behaviours.json` is unchanged; all new actors reuse the implicit `melee` preset with inline overrides.

These are **reconstructed (R) free-field populations**. Client resource identities and regional enemy context are evidenced; historical 2014 field spawn tables, exact field levels, counts, balance and timers are not recovered. The design does not convert battle 96, its summoner or its small summoned trolls into free-field enemies. Friendly named trolls and patrollers remain unchanged.

## Scope and coordinates

All placements belong to native world **zone 0 / FabledRealms**. The 17 `zones.json` ids are regional content labels, not terrain-zone ids. There is no `zone_10.hgt` for Sanctuary. The opening and nearby job referrals reach Farnum/Cobblestone outskirts, Sanctuary/Queensfields and Lakeshore/northeast Wilds; these are the four placement regions below.

Each row is one explicit runtime actor (`spawn_area.count = 1`), not a population multiplier. Three rows make each region's count of three. Positions are `[x,y,z]` in metres, y up. `spawn_area.centre` equals `position`. Proposed idle roaming stays within a **6 m** disc; the absolute home leash is **12 m**. These two radii are (R) and are not currently enforced for field actors. The sketches put +z up only as a coordinate convention; World Builder's alignment with the native minimap remains unchecked.

## Placement table

Every row has empty `quest_ids`, 8 m aggro, 2.5 m melee reach, radius 6 m, count 1 and leash 12 m (all R). Wolves, bixies and trolls reuse the existing level-1 Bone Shaman definition's **100 HP / 7 damage** as local balance. Hooligans reuse the existing field Hooligan's **220 HP / 8 damage**. They are full-size, non-boss melee actors. No ranged or summoning behaviour is invented for these field shamans.

| Area | Region / kind | Proposed level / band (R) | Centre `[x,y,z]` | Respawn (R) | Community positional anchor; horizontal offset (R) |
|---|---|---|---|---|---|
| W102-F1 | Farnum outskirts / Hooligan Wolf | 1 / 1–2 | `[-1899.204,-50.59,307.902]` | 45 s | NPC 4164, Lua 2830; 26.83 m |
| W102-F2 | Farnum outskirts / Hooligan Wolf | 1 / 1–2 | `[-1887.867,-51.14,291.159]` | 45 s | NPC 4165, Lua 2831; 36.88 m |
| W102-F3 | Farnum outskirts / Hooligan Wolf | 1 / 1–2 | `[-1962.890,-46.92,284.723]` | 45 s | NPC 4166, Lua 2832; 36.88 m |
| W102-H1 | Highroad outskirts / Hooligan | 3 / 3–4 | `[-1779.455,-28.88,562.833]` | 60 s | NPC 4159, Lua 2825; 0 m |
| W102-H2 | Highroad outskirts / Hooligan | 3 / 3–4 | `[-1792.921,-29.33,579.550]` | 60 s | NPC 4160, Lua 2826; 8 m |
| W102-H3 | Highroad outskirts / Hooligan | 3 / 3–4 | `[-1789.626,-30.24,616.510]` | 60 s | NPC 4161, Lua 2827; 60.13 m |
| W102-Q1 | Sanctuary / Queensfields / Bixie Soldier | 1 / 1–2 | `[-664.762,2.40,299.608]` | 45 s | NPC 2920, Lua 1663; 8 m |
| W102-Q2 | Sanctuary / Queensfields / Bixie Soldier | 1 / 1–2 | `[-692.141,2.40,220.373]` | 45 s | NPC 2932, Lua 1675; 5.66 m |
| W102-Q3 | Sanctuary / Queensfields / Bixie Soldier | 1 / 1–2 | `[-651.667,2.40,231.947]` | 45 s | NPC 2933, Lua 1676; 0 m |
| W102-T1 | Lakeshore / northeast Wilds / Troll Bone Shaman | 1 / 1–2 | `[-347.084,0.61,1068.348]` | 60 s | NPC 3149, Lua 1892; 112.07 m |
| W102-T2 | Lakeshore / northeast Wilds / Troll Bone Shaman | 1 / 1–2 | `[-351.103,2.33,1048.674]` | 60 s | NPC 3150, Lua 1893; 116.82 m |
| W102-T3 | Lakeshore / northeast Wilds / Troll Bone Shaman | 1 / 1–2 | `[-447.084,3.03,1128.348]` | 60 s | Additional actor (R), NPC 3149 as anchor; 116.28 m |
| MID-F1 | Farnum outskirts / Hooligan Wolf | 5 / 5–6 | `[-2077,-48.70,425]` | 90 s | NPC 3156, Lua 1899; 39.8 m (T415) |
| MID-F2 | Farnum outskirts / Hooligan Wolf | 5 / 5–6 | `[-2065,-47.17,425]` | 90 s | NPC 3156, Lua 1899; 48.0 m (T415) |
| MID-H1 | Highroad outskirts / Hooligan | 7 / 7–8 | `[-2013,-37.60,609]` | 90 s | NPC 4667, Lua 3301; 39.6 m (T415) |
| MID-H2 | Highroad outskirts / Hooligan | 7 / 7–8 | `[-2013,-35.75,597]` | 90 s | NPC 4667, Lua 3301; 48.7 m (T415) |
| MID-T1 | Lakeshore / northeast Wilds / Troll Bone Shaman | 9 / 9–10 | `[-480,4.60,1110]` | 120 s | NPC 3149, Lua 1892; 153.9 m (T415) |
| MID-T2 | Lakeshore / northeast Wilds / Troll Bone Shaman | 9 / 9–10 | `[-489,6.90,1118]` | 120 s | NPC 3149, Lua 1892; 158.7 m (T415) |
| MID-Q1 | Sanctuary / Queensfields / Bixie Soldier | 10 / 10–10 | `[-608,-2.20,172]` | 120 s | NPC 2930, Lua 1673; 62.4 m (T415) |
| MID-Q2 | Sanctuary / Queensfields / Bixie Soldier | 10 / 10–10 | `[-608,-2.10,160]` | 120 s | NPC 2930, Lua 1673; 72.3 m (T415) |
| HIGH-F3 | Farnum outskirts / Hooligan Wolf | 11 / 11–12 | `[-2136,-40.20,516]` | 120 s | NPC 3158, Lua 1901; 65.1 m (T415) |
| HIGH-F2 | Farnum outskirts / Hooligan Wolf | 12 / 11–12 | `[-2148,-37.80,516]` | 120 s | NPC 3158, Lua 1901; 72.1 m (T415) |
| HIGH-T3 | Lakeshore / northeast Wilds / Troll Bone Shaman | 13 / 13–14 | `[-512,6.10,1214]` | 150 s | NPC 3149, Lua 1892; 172.2 m (T415) |
| HIGH-T2 | Lakeshore / northeast Wilds / Troll Bone Shaman | 14 / 13–14 | `[-512,5.50,1226]` | 150 s | NPC 3149, Lua 1892; 175.0 m (T415) |
| ELITE-T1 | Lakeshore / northeast Wilds / Troll Bone Shaman | 15 / 15–16 | `[-408,0.10,1362]` | 180 s | NPC 3149, Lua 1892; 192.9 m (T415) |
| ELITE-T2 | Lakeshore / northeast Wilds / Troll Bone Shaman | 16 / 15–16 | `[-420,0.00,1362]` | 180 s | NPC 3149, Lua 1892; 197.3 m (T415) |
| ELITE-Q1 | Sanctuary / Queensfields / Bixie Soldier | 17 / 17–18 | `[-572,16.30,320]` | 180 s | NPC 2915, Lua 1658; 79.7 m (T415) |
| ELITE-Q2 | Sanctuary / Queensfields / Bixie Soldier | 18 / 17–18 | `[-560,16.60,320]` | 180 s | NPC 2915, Lua 1658; 89.3 m (T415) |
| ELITE-H1 | Highroad outskirts / Hooligan | 19 / 19–20 | `[-1817,-44.45,176]` | 180 s | NPC 23623, Lua 4152; 39.8 m (T415) |
| ELITE-H2 | Highroad outskirts / Hooligan | 20 / 19–20 | `[-1817,-43.45,188]` | 180 s | NPC 23623, Lua 4152; 40.4 m (T415) |

**T415 (2026-10-22): the level 5-20 camps left the starter centres.** T245, T296 and T332 had put the MID/HIGH/ELITE rows on these W102 centres, so a level 1-4 player met level 7-20 hits (native: a level-5 player at W102-H1 took 46 and 141). The 18 rows above now sit on audited centres: the W102 checker's height, full-leash slope and NPC/town/path limits (every `npc_spawns_extra.json` record counts as an NPC anchor); at least leash + leash + aggro + 4 m from any actor whose band is more than two levels away; leash + aggro + 20 m from every difficulty 1-2 battle marker; leash + Flame Wave radius + 4 m from the power-up suite's clear point; and a walk of slopes up to 25 degrees to the region's W102 camp (R). Higher bands sit further from Sanctuary, Cobblestone and Farnum (R). The anchor is the nearest same-kind Legacy spawn. These rows keep their T245/T296/T332 stats, not the 100/7 W102 balance above. Per-centre checker output: `notes/Autopilot/T415.md`; `tests/field_camp_separation.rs` enforces the separation.

The troll relocations are substantial: they are new clearing/approach placements in the same broader region, not claims of recovered transforms. Community positions beside steep raster breaks failed the full-leash terrain check. T3 is an extra actor (R), not a third recovered community spawn. Some positions were also separated to keep centres at least 20 m apart and avoid protected corridors.

### Final-era level policy

Use the final combat-era progression anchors **1, 3, 5, 9, 13**, rather than treating the early-game 2009 quest walkthroughs as a level specification. The reviewed guides give difficulty-1 Bixie Hive and Bone Shaman encounters no minimum level; Highroad Hijinx is difficulty 2, whose recommended level is 3. This motivates the proposed 1–2 starter bands and 3–4 Highroad band. It does **not** recover individual overworld enemy levels or historical regional bands. `level` is the chosen lower-band level, not a random roll. It is currently ignored by field runtime.

The local wiki has later retrospective revisions, not a frozen 2014 server table: `Battles` revision 48953 (2025-03-16), `Bixie Hive` 49119 (2025-09-07), `Treasure of the Bone Shaman!` 49837 (2026-05-04). Exact 2014 per-kind levels remain a fidelity gap. The final-era generic level gates are also supported by client locale ids in the reviewed Bone Shaman guide. Do not promote the proposed field bands to recovered facts.

## Region sketches

Schematic only; not to scale. `=` denotes a protected town/corridor, letters are placement ids from the table. Exact safety geometry is private `lab/reports/field-enemies-w102/safe_zones.json`.

### Farnum outskirts

```text
                       +z
                        ^
  opening wolves (protected)      arrival == Jones/Cobblestone
  farmyard ======= Mac ==============> Sanctuary
                          F1
                    F3      F2
                       old Bullies marker (protected)
  ------------------------------------------------------> +x
  farm x~-2185             wolves x~-1963..-1888
                           wolves z~285..308
```

The new wolves avoid the five opening-story wolf positions near x −2101..−2085 / z 457..469. They do not replace the opening's stage logic. The passive Bullies marker's location is protected even though that battle is not routed here.

### Highroad / Cobblestone outskirts

```text
                       +z
                        ^                   H3
  [Cobblestone safe town]                 H2
        Jones                             H1
          ||
        arrival              farm-to-Queen corridor lies south
  ------------------------------------------------------> +x
  Jones (-1896,567)                 H x~-1793..-1779
                                   H z~563..617
```

The quest-gated Hooligans south of Jones remain separate, with their existing ids, stats, positions and quest gate. These new Hooligans use independent GUIDs and do not expand that quest's population.

### Sanctuary / Queensfields

```text
                       +z
                        ^
  [Sanctuary safe rectangle]                    Q1
  x -1540..-860; z 130..490
                                                  Q3
                                           Q2
  ------------------------------------------------------> +x
                                      Q x~-692..-652
                                      Q z~220..300
```

These are detached Queensfields combat patches outside the protected inner-town rectangle, not enemies in the shops, hospital or palace. Only three of the community area's many positional anchors are used. The selected `bixie_m_warrior.adr` / Soldier name pairing is reconstructed, not a recovered server assignment.

### Lakeshore / northeast Wilds

```text
                       +z
                        ^
                     Vergosh (protected)
            battle 96 marker (protected, unchanged)
                     Troggers
                       ||
                  T3   ||
                      Nog =======  T1
            Grosh ======           T2
              //
  Ty == Len / bridge approach (protected)
  ------------------------------------------------------> +x
  Ty x~-731                         T x~-447..-347
                                    T z~1049..1128
```

The troll population is placed off the referral corridor. The file does not make friendly village trolls hostile, scale down these full-size shamans, or reuse the battle summoner's skittish cycle. the owner's field-note observations about small summoned trolls and short melee reach remain requirements for battle 96, which W102 does not modify.

## Provenance and limits

Each appended JSON row carries its own source entries. Hashes of the inspected inputs and output are in `lab/reports/field-enemies-w102/input-hashes.json`.

| Fact | Evidence | What it does not prove |
|---|---|---|
| Wolf resource identities | Client `Resources/Models.txt` row 2470: model 3284 `wolf_generic.adr`; locale-id-to-text row for 5100200; existing `enemies.json` `hooligan_wolf` | Historical free-field position, level, HP or damage |
| Hooligan resource identities | Client Models row 204: model 203; locale 5100401; existing four `field_enemies.json` rows | Exact 2014 combat stats or model assignments at new centres |
| Bixie resource identities | Client Models row 221: model 220 `bixie_m_warrior.adr`; locale 440711 “Bixie Soldier” | That the historical server paired these ids; pairing remains R |
| Bone Shaman resource identities | Client Models row 169: model 168 `troll_shaman.adr`; locale 72269; existing `enemies.json` `w58_troll_bone_shaman_l1_enemy` | Historical free-field aggression, ranged spells or summons |
| Farnum wolves / Hooligan regional context | Build Guides `Content/NPCs/NPCs - Farnum Farmstead.md:85,87,129`; `Content/Dungeons/Highroad Hijinx.md:19,48` | Always-on population or precise field band |
| Queensfields bixies | `Content/NPCs/NPCs - Sanctuary.md:40,123`; `Content/Dungeons/Bixie Hive.md:18,26,48` | A field spawn table derived from the dungeon's enemy list |
| Northeast troll context | `Content/NPCs/NPCs - Wilds.md:111`; `Content/Dungeons/Treasure of the Bone Shaman.md:19,27,43` | That an encounter marker should become a hostile field actor |
| Positional/count anchors | FreeRealms-Legacy pin `9e082025391177d5a33ee18c714ddd06b457a695`, `src/Scripts/Zone/FabledRealms.lua`, row/line per placement table | Community code is not copied; positions are not native-verified transforms |
| Town bounds cross-check | Sanctuary pin `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, `src/Resources/Zones/FabledRealms.json`, `AreaDefinitions` Id 5 | Recovered road polygons or exact town collision boundaries |
| Terrain | `lab/terrain/zone_0.hgt`, parsed using the existing `terrain.rs` OSFRHGT1 layout and bilinear interpolation | Water, trees, rocks, doors, bridges, object collision or a traversable route from a road |

Web queries: `Free Realms Queensfields Bixie Soldier level 2014`; `Free Realms Hooligan Delinquent level battles 2014 Troll Summoner`. [Battles](https://freerealms.fandom.com/wiki/Battles) supports the generic difficulty anchors, [Queensfields](https://freerealms.fandom.com/wiki/Queensfields) the regional bixie context, and [Highroad Hijinx](https://freerealms.fandom.com/wiki/Highroad_Hijinx) the difficulty-2 context. These are retrospective community references, not primary 2014 level tables. The older 2009 Brawler walkthrough returned by search was not used as a final-era level oracle. No archive/tool downloads or new input assets were needed.

## Runtime: already working and remaining gates

Source was inspected after the Ledger and newest `Native Round5 050310` review. Graph project `cartoonmmo` was ready at generation `2026-10-16T01:56:50Z`; coverage reported changed source metadata. Current-source reads therefore supersede graph line numbers. Concurrent builders are changing Rust; `lab/reports/field-enemies-w102/runtime-evidence.json` freezes inspected function excerpts, line numbers and file hashes. The bot receipt separately pins the tested existing debug binaries; W102 did not build or edit them.

| Requirement | Verified implementation / exact gap | Lead's remaining gate |
|---|---|---|
| Always-on without an active quest | **Already implemented.** `encounter.rs` `EncounterEngine::load_field_enemies` sets `present` when `quest_ids` is empty; `set_active_quests` skips those entries. `field_initialization` is called by `on_zone_ready` and reload. `Gameplay::load`, `sync_field_quests` and `reload_content` route it. The scratch bot entered with no Hooligan quest and found the new Wolf and Soldier. | Native new-character/relog check, alongside unchanged quest-only Hooligans. Do not build a second spawn system. |
| Roaming | **Missing for field actors.** `EncounterEngine::tick_field` chases inside `aggro_range`, otherwise returns to the single home position. `tick_wanderers` belongs to battle markers only. `Gameplay::tick` calls `encounter.tick`; `gateway.rs` routes that tick. | Give idle field actors ground-snapped goals inside `spawn_area.radius` and pause/stop movement packets. Reuse the existing ground/movement helpers and marker wander pattern. Native observe motion, not only packets. |
| Home leash and safe paths | **Missing for field actors.** `tick_field` does not enforce `behaviour.leash`, `chase_limit` or the authored `leash_radius` as a maximum distance from home. A player can keep pulling an enemy while within its moving aggro radius. Current inline values do not make the safe-zone design effective. | Parse the authored home leash, clamp all chase/roam goals, return home on breach, reject unsafe terrain goals, stop correctly, and prove enemies cannot enter protected buffers. Native check object/water/path clearance; the height grid alone cannot prove it. |
| Respawn timers | **Already implemented per player.** `EncounterEngine::strike` schedules `respawn_at` using `respawn_seconds`; `tick_field` recreates the actor/state when due. `reload_field_enemies` resets field state. Existing `quest_gated_field_enemies_spawn_fight_and_respawn` also covers the old path. W102 did not kill/wait out a timer. | Native kill and timed 45/60 s respawn. Define reload/reset policy explicitly. Move timers with shared-world ownership; do not duplicate this timer machinery. |
| Level / spawn metadata | **Ignored today.** `Actor` / `FieldActor` do not deserialize `level`, `level_band`, `spawn_area`, `leash_radius`, `reference_spawn_guid`, `region` or `zone_id`. HP/damage are consumed, but this table's bands are not runtime scaling or presentation. | Validate/load the selected level and area/home metadata. Use zone 0; do not route regional label ids as terrain zones. No random band roll is implied. |
| Shared field world | **Not implemented.** Each session's `Gameplay` owns its `EncounterEngine` and mutable field enemy copies. Gateway session locations still use instance 0. Two players can independently damage/kill/respawn the same authored GUID. | Build Plan 3.1: one zone-owned field actor/health/aggro/death/timer state, broadcast changes to interested clients, validate attack target/session identity, and award a kill once under an explicit contribution policy (R). Keep arenas per group, with distinct instance ids and party consent. Two clients must observe one kill and one respawn. |
| Passive community copies | **Explicit integration gap.** New GUIDs `131000102001..131000102012` avoid content identity collisions after offsets. `reference_spawn_guid` records the old positional anchor; runtime ignores it. `Gameplay::load` only calls `NpcVisibility::exclude_actor_guids(engine.field_guids())`, so it suppresses matching runtime GUIDs, not these reference ids. | Suppress the selected old passive copies using explicit validated replacement metadata, or retire those passive rows through the lead-owned content pipeline. T3 reuses an anchor, not an additional reference actor. Do not suppress unrelated named/friendly trolls or the battle marker. |
| Named behaviour startup | **Load-order limitation, worked around in content.** `Gameplay::load` calls `load_field_enemies` before `load_behaviours`. `resolve_behaviour` rejects an explicit named preset before that map is populated. An inline object without `preset` uses the default melee base, then resolves against the loaded melee preset. | Current content loads. If future content names a new preset, load presets before all referring actors; keep atomic reload failure behaviour. |

The shared-world, leash and collision gaps mean this content must not be called a playable roaming-enemy milestone. It is reviewable placement/design content with a successful loader/bot gate. Plan/Ledger edits belong to the lead and are outside W102 ownership.

## Exact checks for the content-checker owner

`content_check.py` was not edited. It currently validates referenced native resources and general content ids, but not this authoring schema. The private `check_placements.py` applies these additional rules:

1. Keep the original four records unchanged. Append exactly 12 actors; runtime GUIDs and area ids must be unique. Keep the whole field file within the runtime's 64-record / 128 KiB limits.
2. For these rows, require `quest_ids == []`, `zone_id == 0`, non-boss, finite three-number positions, `1 <= level_band[0] <= level <= level_band[1] <= 20`, and respawn 1–3600 seconds.
3. Require one actor per area, centre equal to runtime position, and `0 < radius <= leash_radius <= 40`. Inline overrides must omit `preset` until load order is fixed; reach is at most 2.5 m. Aggro must be positive. Preserve the runtime's own actor/behaviour bounds too.
4. Check the centre and the entire leash disc against zone-0 grid bounds and complete four-corner coverage. Height error must be <=0.051 m. Sample every 2 m inside the leash, with centred 1 m finite differences; maximum local slope <=25 degrees (R). Missing corners fail rather than using the runtime's partial-corner mean.
5. The whole leash plus melee reach must stay at least 25 m from all known `legacy_data` non-enemy NPC centres and the additional protected story/referral/marker centres. Stay 20 m from protected town rectangles and path polylines after adding that same leash/reach margin. These reconstructed polylines are not proof of original road clearance.
6. Every row must retain client/reference evidence and an explicit `(R)` policy source. `reference_spawn_guid` is a positional association, may repeat for T3, and is not a second runtime actor id.

Out-of-bounds and wrong-height mutation canaries must fail. Before authoring, the check failed with `AssertionError: expected 12 always-on enemies`. After authoring, both deliberate mutations were rejected.

## Verification receipt

From `project`, run:

```powershell
python -I scripts/content_check.py --strict --out ../lab/reports/field-enemies-w102/content-check.json
python -I ../lab/reports/field-enemies-w102/check_placements.py
python -I ../lab/reports/field-enemies-w102/scratch_gate.py
```

Results: **0 errors** (17,276 existing warnings); **12/12 terrain-walkable centres** under the stated R criterion, 113 leash samples each; four old quest rows unchanged; both canaries rejected. The scratch bot passed **267 received messages**, `realm goto` at F1 and Q1, `realm here` finding the appropriate enemy kind, `realm reload behaviours`, and `realm reload enemies` twice with 35/2 AddNpc replies and successful text responses.

The owned scratch gateway bound only `127.0.0.1`, used `--studio-port 0`, had an independent fresh scratch state, and was stopped through its own process handle. No native client or shared service was controlled. `scratch_gate.py` copies the existing debug binaries to the report directory and pins their hashes in `scratch-receipt.json`. It supplies frozen round70 gameplay plus private copies of NPC/zone/field content; it intentionally does not read the concurrently changing shared job-abilities file. It is a loader/bot gate, not a build of the current concurrently edited Rust sources.

Evidence under `lab/reports/field-enemies-w102/`: `content-check.json`, `placements.log`, `bot.log`, `gateway.stdout.log`, `gateway.stderr.log`, `scratch-receipt.json`, `visits.bot`, `input-hashes.json`, `runtime-evidence.json`, `safe_zones.json`, `sites.json`, and the three runnable Python scripts. `field_enemies.before.json` preserves the baseline. Failed named-preset startup and shared-fallback diagnostics are retained separately. A bot expectation initially consumed a single `here` response twice; the final script issues a separate `here` for each assertion.

### Security review

Applied `secure-code-guardian` and `security-reviewer` within W102 scope. Threat surface: trusted content reaches NPC spawn packets; the gate runs privileged developer commands, reads host-selected input paths and creates a disposable state. All subprocess arguments are lists without a shell; commands, executable paths, content paths and loopback destination are host-fixed. Studio is disabled. Only the owned Popen handle is stopped. No credentials, account store, external messages, network exposure or runtime trust boundary changes are introduced.

Automated checks cover JSON/resource validity, finite coordinates, bounded counts/radii/timers and mutation canaries. A static scan plus manual review checked the owned scripts for shell execution, dynamic evaluation, destructive tree operations and embedded credentials. No security findings requiring a patch remained. **Info:** unknown authoring fields are ignored by runtime, so they must never be treated as enforced player/security boundaries; the runtime gap table and checker rules make this explicit. This is a scoped diff review, not a security certification of the shared gateway.

### Skills and verdict

Ponytail at full intensity reused the current field schema, implicit melee preset, existing Bone Shaman/Hooligan balance, terrain layout/interpolation and bot/gateway tools. Caveman kept transient communication short; this requested design/evidence note remains complete. Codebase-memory located the route, then source reads resolved stale coverage. Game-research-websearch checked level/context evidence; Debugging Wizard isolated startup failures; the security skills constrained/reviewed the helper. No new asset reader, behaviour mode, preset, Rust module, dependency, native control, agent, commit or publication was built.

**PARTLY PROVEN:** content/resource, terrain and scratch route/reload gates passed. Native appearance, actual navigability, exact 2014 field levels, enforced leash/roaming, kill/respawn observation, passive-copy suppression and shared-world combat remain open.
