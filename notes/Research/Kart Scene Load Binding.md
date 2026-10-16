# Kart Scene Load Binding

Read-only blocker research, 2026-10-09 (UTC). Scope: follow the native `41/2`
and `41/114` dispatcher path into the encounter transition, decide what the
shared constant `0xde2106fb` is, and check whether an encounter-`917566`
lookup returns scene data. Sources used: the pinned Ghidra pseudocode
(`lab/decompiled/client/FreeRealms/pseudocode.c`, sha256
`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`), the
shipped client `Resources` tables of the pinned client snapshot, the OSFR
162k-entry asset manifest, the preserved minigame packet bodies, the wiki
SQLite and the two community POI databases. No UI, source, content, save,
release or process change; nothing was launched. No custom file format was
reversed - the shipped tables are plain `^`-delimited text.

Owned artifacts: `lab/reports/kart-scene-load-binding-20261009T154940Z/`
(`probe-kart-scene-binding.py`, `proof.json`, `run.log`, `final-hashes.txt`).

## Verdict first

`0xde2106fb` is **not the per-race scene selector**: it is byte-identical in
the racing (`917566`), demolition derby (`1310785`) and soccer (`458824`)
captures, it is not a literal anywhere in the 95 MB client pseudocode, it is
not an id in the 162,653-entry shipped asset manifest, it is not in either
community POI database, and it matches none of six common hashes over the
race/scene name candidates. The per-race identity carried by the capture is
the **encounter id plus the spawn triple**, not this constant. The client
pseudocode shows no `917566 -> scene` lookup and no `speedway`/`raceway`/
`sh_kt_track`/`LoadScene` literal at all; the encounter classes are lifecycle
and detail containers. The world/scene/geometry binding stays the specific
missing piece.

## Native dispatcher anchors (pseudocode line numbers)

The encounter packet family is built by `FUN_008a2fd0(<sub>)`; extracting that
constant plus the following `*param_1 = <Class>::vftable` gives the full sub
table:

| Sub | Class | Sub | Class |
| --- | --- | --- | --- |
| 2 | `EncounterPacketPlayerEnter` (1375068) | 109 | `EncounterParticipantRequestExitPacket` |
| 3 | `EncounterPacketEncounterComplete` | 114 | `EncounterDetailsResponsePacket` (1382631) |
| 4 | `EncounterPacketEncounterLaunchFailed` | 120 | `EncounterParticipantMessagePacket` |
| 5 | `EncounterPacketObjectiveState` | 121 | `EncounterParticipantTerminateEncounterPacket` |
| 6 | `EncounterPacketEncounterShutdown` | 122 | `EncounterParticipantResumePacket` |
| 102-105 | invitation / duel invitation + responses | 124 | `EncounterParticipantCancelPendingEncounterPacket` |
| 106 | `EncounterStatePacket` | 125 | `EncounterShowRespawnWindowPacket` |
| 107 | `EncounterZoneIsReadyPacket` (1374324) | 129/130 | `EncounterRunningListRequest/ResponsePacket` |
| 108 | `EncounterParticipantRequestEntrancePacket` | 132/133 | `EncounterOverworldCombatPacket` / `EncounterPacketIsFighting` |

So the captured `41/2` is the **PlayerEnter lifecycle** message, not a scene
payload, and `41/114` is the **DetailsResponse**
(`EncounterDetailsResponsePacket`, ctor 1382631) on the same encounter base.
The family's record container is `SoeUtil::List<EncounterDetailsCommon>`
(`EncounterRunningListResponsePacket` ctor at 1383013); the record reader
(`FUN_00aa32d0`, 1382846) stores a bool at `+0x2e8`, a u32 at `+0x2ec` and a
nested list at `+0x2f0` (`FUN_009b0700`); nothing in that reader names a zone,
scene or asset. `41/107 EncounterZoneIsReady` carries only
encounter + instance (inner `29 00 6b 00 3e 00 0e 00 f0 03 00 00`) - its
"zone" is encounter readiness, not geometry.

## Captured frames (per-race values; constant unchanged)

| Frame | op/sub | encounter | instance | state | const | spawn |
| --- | --- | --- | --- | --- | --- | --- |
| racing 418B, state 2 | 41/114 | 917566 | 1008 | 2 | `0xde2106fb` | 719.61, 72.0, 421.5 |
| racing 398B, state 4 | 41/114 | 917566 | 1008 | 4 | `0xde2106fb` | 719.61, 72.0, 421.5 |
| demoderby 418B | 41/114 | 1310785 | 1008 | 2 | `0xde2106fb` | 7.0, 240.59, 317.32 |
| soccer 521B | 41/114 | 458824 | 1008 | 2 | `0xde2106fb` | 0, 0, 0 |

`41/2` layout: encounter i32 + instance i32 + **two dwords + one flag byte**.
First entry sends `0,0,0`; the later send per minigame sends
`0x9573ca91, 0x4b52cc6a, 0` - the two halves of the same session u64 seen in
the 33/9 race entry. The zero-vs-session difference is therefore two 32-bit
fields plus the flag byte; it is preserved, not flattened. `41/106` states
2→6, `41/121` terminate is encounter+instance+0, `41/132` is an id-less
routing message.

## Shipped client tables and manifest

* `Resources/MiniGameData.txt` (783 rows): 123 kart-family rows
(`TYPE` 10/11/13/19/20) name the five speedways' races - `Wildwood Kart
Racing` (28, 310, 561, 562, 565...), `Frost Ridge Kart Racing` (37),
`Tangletrack Kart Racing` (38), `Thunder Falls Speedway` (567), `Sandstorm
Speedway` (later rows), plus demo derby, `Shipwreck Circuit` and
`Switchback Circuit` rows. `PRELOAD_FILE` is **empty in all 783 rows**, and no
row carries `917566`.
* `GameTutorialData.txt` row 1003 = `racing.swf` icon 1314 (a racing tutorial;
`Resources` itself has no played racing SWF in this snapshot).
* `RaceDefinitions.txt` is **character races** (Human/Pixie/Robgoblin...), not
kart races - a name collision, recorded so it is not mistaken for a lead.
* `Assets_manifest.txt` (162,653 rows) contains the shipped scene assets
(`sh_kt_track_bbec_312/313.*`, `sh_kt_tracksnow1_bbe.*`, `karts1.gfx`,
`sg_billboard_raceway_*`, later `sg_raceway*`/`sh_kt_track_*.gcnk` rows) but
**no `3726649595`**, so `0xde2106fb` is not a shipped asset id.
* The only client-side `kart` scene asset found by name in the snapshot is
`Resources/SKY/sky_kartcavern_bbe.xml` (a sky dome for a kart cavern scene).

## Hash and archive checks (negative)

Six hash functions (CRC32, FNV-1a, FNV-1, djb2, sdbm, `h*31+c`) over 19
candidate names (`Wildwood Kart Racing`, `Thunder Falls Speedway`, `Kart Race
Lobby`, `raceway`, `speedway`, `sh_kt_track`, ...) produce **no** match for
`0xde2106fb`; the wiki SQLite has no `917566` or `Vehicle Race In The Wilds`;
the community POI databases and the retail self-packet contain no `917566`
either. Archive/secondary sources only confirm the five speedways and their
race names (wiki "Speedways": Tangletrack/Briarwood, Frost Ridge/Snowhill,
Wildwood/Sanctuary, Thunder Falls/Lakeshore, Sandstorm/Seaside; all but
Sandstorm also offer Demo Derby).

## What actually selects the race geometry

Nothing inspected does. The client path carries only encounter lifecycle and
detail data; the shipped minigame tables name the races but hold no scene or
preload value; the scene assets exist in the manifest but the capture's
constants are not manifest ids. The remaining candidate is server-side
content: the server sends the encounter id/instance (1008) and the details
payload, and the zone/scene assignment would come from the server's encounter
or zone data. A concrete external lead: the community `FreeRealms-Legacy` /
Sanctuary server consumes a `sanctuary.db` database file for world data
(`Sanctuary.Database` README) - that server-side DB, or the same table on a
retail server, is where an encounter -> zone/scene binding would live; it is
not present in this workspace. No POI, zone, collision or finish binding is
invented here, and the accepted entry candidate still does not make the race
playable.

## Verification

```
& '<lab>/tools/universal-modder-env/Scripts/python.exe' \
  lab/reports/kart-scene-load-binding-20261009T154940Z/probe-kart-scene-binding.py
```

The probe re-derives the sub table, the literal counts, the anchors, the
shipped table scans, the manifest id check, the capture field maps and the
hash negatives, then writes `proof.json` with `write_exact` semantics.
Output hashes are in `final-hashes.txt`; the key inputs are hashed inside
`proof.json` (`pseudocode.c` `f84a5867...`, `MiniGameData.txt`, the pinned
client snapshot's tables, and the 162k manifest).

Cross-links: [[Research/Kart Speedway Entry Binding]],
[[Research/Go Kart Native Integration Readiness]],
[[Research/Kart Lobby Definition Framing]].

## Correction (2026-10-09T16:05Z) - bounded wording, withdrawn lead, resolved pointer

The first version of this note had sha256
`7f6d6d61c4587e4c4875500ce489c5443b0bb0e9b06d6b9da4e199f6c085fe9f`. It is
narrowed here; the observations stay, the universal claims do not:

1. The identical `0xde2106fb` in the racing, derby and soccer captures proves
   only that the field does not vary across those three captures. It does not
   by itself rule out a shared scene selector (a common minigame world or
   template would look the same). Role unresolved.
2. The zero literal matches, the 19-name/six-hash negatives and the decompile
   search are **observed searches**, not proof that no lookup exists. A lookup
   can be data-driven or reached through a reader whose field flow was not
   traced here.
3. **Withdrawn lead:** `sanctuary.db` is not a shipped world database. It is
   the Sanctuary emulator's own EF Core persistence DB (entities
   `DbUser/DbCharacter/DbItem/DbMount/DbProfile/...`, SQLite/MySQL providers,
   `Data Source=D:\Games\Free Realms\sanctuary.db`). It has no zone, scene or
   encounter table. The public world data is `src/Resources/Zones/*.json`
   (only `FabledRealms.json`). See [[Research/Kart Community World Binding]].
4. `karts1.gfx` discrepancy kept explicit: `Kart Racing Restoration` quoted a
   size that `Go Kart Native Integration Readiness` could not reproduce from
   the pinned catalogue. The OSFR full manifest does carry
   `karts1.gfx,3707340300,5064`, while the local 24-pack catalogue has no
   `karts1.*` entry. Unresolved; recorded, not silently corrected.

The actual scene selection turned out to be an **op31 scene-load packet that
carries the scene name string** (`sg_kt_track` for kart racing, with the spawn
`719.61, 72.0, 421.5` and `sky.xml`), not the `0xde2106fb` field. Full wire
decode, ordering and shipped `sg_kt_track.gzne` pack entries are in
[[Research/Kart Community World Binding]]. The broader statements above stay
bounded to the searches actually performed.
