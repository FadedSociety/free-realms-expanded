# 051 Enemy catalogue

Acceptance: original models/factions/skills/spawn data with explicit missing bindings. Work item
[[notes/WorkItems/051.md]] under the [[notes/Design/MMO Worker Contract.md]] classification rules.
Evidence bundle: `lab/reports/piece-051-20261009T053532Z/`.

Artifacts

- `prototype/osfr-tools/src/restoration/piece_051.rs` - recovered model-table parser, combat
  marker, applied enemy identities and the identity validator.
- `prototype/osfr-tools/tests/piece_051.rs` - 8 acceptance tests (`#[path]` include until the lead
  registers the module).
- `scripts/pieces/piece_051.py` - evidence checker that builds and validates the catalogue.
- `tests/pieces/test_piece_051.py` - 12 checker tests, including synthetic rejection paths.
- `lab/reports/piece-051-20261009T053532Z/` - `enemy-catalogue.json` (443 rows),
  `findings.json`, `provenance.json`, RED/GREEN logs and the checker output.

## Original facts

`Resources/Models.txt` is the client-shipped model table: 3743 rows and 17 columns -
`ID, MODEL_FILE_NAME, RACE_ID, SCALE, DESCRIPTION, GENDER, AGE, DESCRIPTOR, MATERIAL_TYPE,
WATER_DISPLACEMENT, TELEPORT_EFFECT_ID, CAMERA_DISTANCE, CAMERA_ANGLE, IS_VALID_FOR_PC,
NAMEPLATE_OFFSET, CAPSULE_HEIGHT, RADIUS`. sha256
`0bad5a154c3ef855744349eb03eb3ab02e7d8dfc3d29fd0f601257fce95076be` (identical in
`lab/working/OSFR.Client/OSFR Client/Resources/` and the preserved client run copy).

Exactly 27 rows carry a combat actor descriptor: `Combat NPC` (17), `Halloween Combat NPC` (8),
`Combat NPC: Forest Giant` and `Combat NPC: Forest Giant Boss`. They name the original combat
model families: robgoblin, banshee, ghostdwarf, bat, screecher, sasquatch, black widow and
black widow queen, vine golem, greenwood chugawug soldiers, forest giant and boss, necrowartz
zombies, forest and Hogosh trolls, man-eating plant, and eight werewolf variants.
`Combat training prop` and `Halloween Combat Powerup` are props and are rejected by the exact
marker rule.

`Resources/RaceDefinitions.txt` maps each model's race id to an original locale name id, so a
model groups by recovered creature race (102 Robgoblin Warrior, 9 Ghost, 7 Chugawug, 20 Forest
Troll Village, 105 Vine Golem, 12 Bat, 18 Spiderling, 11 Shambler, 16 Yeti, 27 Animated Plant).
The preserved original locale table (85,932 rows, community export sha256
`f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eeba6ae62844`) resolves every name id
this catalogue uses, including the enemy names the local opening spawns: 5100200 Hooligan Wolf,
5100344 Mac, 5101961 Gloam Creature, 441457 Gloamling, 5101505 Gloam Overlord.

## Community recovery

Two pinned community NPC tables bind definition id to name id and model id: Sanctuary
(`references/Sanctuary/src/Resources/Npcs.json`, 3426 definitions, sha256 `b51d2d92...`) and its
wider fork FreeRealms-Legacy (4253 definitions, sha256 `f8f22448...`). 3419 definition ids appear
in both and agree on **every** model id and name; the fork adds 834 definitions. A row is kept as
community evidence only where it also binds to the original model table, which is why the
independent agreement matters.

Placements come from the pinned zone scripts (`Scripts/Zone/FabledRealms.lua`, Sanctuary
sha256 `1fecb078...`, Legacy `bc0673f8...`). Each fork places exactly one spawn per definition id
and the spawn guid is `100000000000 + definition id` for 3421 of 3426 Sanctuary rows; ids 33081,
33167, 33558, 33602 and 36143 keep a different guid. 12 shared definition ids carry **different
positions** between the two forks (2163-2166, 2254, 2552, 2649, 2853, 2886, 3422, 4611, 33081):
which placement is original is unresolved and the catalogue flags those rows.

The wiki corpus contributes the documented enemy roster: 92 pages filed under `Category:Enemies`
with a verbatim category list, 39 of whose titles match a community NPC name exactly. Faction
coverage is thin and stays labelled as community documentation: a category counts as a faction
only when it agrees with a page the wiki itself files under `Category:Groups` (Skullz biker gang,
Gloam, Manguana Shaman, ...). 19 pages carry such a category; the catalogue resolves 133 rows
(Manguana 70, Skullz 59, Gloam 4). The client ships no faction field and the wiki `faction`
infobox row is empty, so the remaining rows have no faction binding at all.

## Reconstructed boundary

The local opening spawns 7 enemy actor identities. Hooligan Wolf 3156 is a preserved community
definition. Gloam Creature 920001, Mac 920002, Gloamling 920100-920102 and Gloam Overlord 920110
are **local definition ids**: their models (`gloam_m_construct.adr`, `human_m_hooligan_boss_02.agr`,
`gloamling_m_default.adr`, `gloam_m_boss.adr`) and names are original, no preserved table defines
the ids. Models 3933 and 3897 have no community NPC row at all.

The first world wolf placement equals community definition 3156 exactly
(`[-2100.965, -45.98195, 456.7631]`, guid `100000003156`). The other four local wolves reuse
definition 3156 with positions stepped `(+4, 0, +3)` from it, while the community table places
distinct definitions 3156-3160 at distinct coordinates. Health and damage in the local encounter
are reconstructed local rules, not recovered enemy stats.

The closest recovered enemy *skill* evidence is the shared creature attack animation slot
(`com_h2h_attack_01`, animation id 1001) owned by [[052 Enemy patrol and idle]] and
[[054 Melee attacks]]; those are animation and reach contracts, not a per-enemy ability table.

## Explicit missing bindings

| binding | state | what would close it |
| --- | --- | --- |
| Enemy skills/abilities | not recovered for any enemy | server-side NPC/encounter data source or a capture holding enemy ability cast messages |
| Enemy health/damage | not recovered; local values are reconstructed | original server stat table or capture |
| Faction | only 133 of 443 rows have a wiki-documented group | original faction field or filled wiki infobox rows |
| Definition id | 6 local-only rows (920001, 920002, 920100-920102, 920110) | preserved NPC definition table or spawn capture binding the model/name pair |
| Names | 111 of the 180 combat-marker rows have no recovered name | server-side NPC name binding |
| Placement | 6 local-only rows have no community placement; 12 shared rows have divergent fork placements | a capture of the original spawn records |

Two smaller drifts are reported rather than repaired: the original table keeps a trailing space
in `cray_king_lowtide.adr ` (model 4446) that both community rows trim, and the original locale
keeps a trailing space in 4 names (3043, 3227, 4280, 4286) that the community rows trim.

## Checks

```
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_051.py" -v
  Ran 12 tests ... OK
& '..\..\scripts\cargo.ps1' test --test piece_051        (from prototype/osfr-tools)
  test result: ok. 8 passed; 0 failed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_051.py'
  {"rows": 443, "local_applied_enemies": 7, "findings": [...]}
& '..\..\scripts\cargo.ps1' check                        (from prototype/osfr-tools)
  Finished dev profile
```

RED first: `tests/piece_051.rs` failed with `couldn't find file
tests\..\src\restoration\piece_051.rs` and `tests/pieces/test_piece_051.py` failed with
`FileNotFoundError: scripts\pieces\piece_051.py` before either artifact existed
(`red-rust-test.log`, `red-python-test.log`).

## Gates

No native or gameplay claim: this item binds identities and rejects fabricated ones. Not
exercised: any native spawn/render of these rows, enemy combat, and the 12 divergent placements.
Integration for the lead: register `pub mod piece_051;` and drop the `#[path]` include in
`tests/piece_051.rs`; the catalogue rows are recovery metadata and must not be turned into
granting or stat authority.
