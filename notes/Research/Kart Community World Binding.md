# Kart Community World Binding

Read-only research, 2026-10-09 (UTC). Scope: community-first search for the
encounter -> world/scene binding behind the kart speedways, the original `op31`
scene-load packets, and the shipped `.gzne` geometry entries. No UI, source,
content, save, release or process change; no executable was run; one small
text file was downloaded and preserved with its hash. This note also carries
the correction of the withdrawn `sanctuary.db` world-DB lead used earlier.

Owned artifacts: `lab/reports/kart-community-world-binding-20261009T155851Z/`
(`probe-community-world-binding.py`, `probe-scene-binding.py`,
`proof.json`, `proof-scene-binding.json`, `run*.log`, `downloads/`,
`download-inventory.txt`, `final-hashes.txt`).

## Verdict first - the binding is an op31 scene name

The original client is told the race scene by **opcode 31 with the scene name
as a length-prefixed string**: the racing capture's
`racing-13071-flow0-frame723-op31-subNone.bin` carries `sg_kt_track`, spawn
`719.60999, 72.0, 421.5`, `sky.xml`. The same packet family carries
`sg_dm_arena` for demolition derby, `sg_soccer` for soccer, and `FabledRealms`
when returning to the world. The later `41/114` encounter packet
(`917566`/`1008`) repeats the identical spawn triple, which ties the encounter
to the scene named by op31. The earlier `0xde2106fb` field is not this
selector.

Chain, all from the preserved community capture:

| t (s, relative) | Frame | Packet | Evidence |
| --- | --- | --- | --- |
| 0.000 | 135 | `41/114` | encounter `917566` / instance `1008`, state 2, spawn `719.61, 72.0, 421.5` |
| +4.282 | 723 | **`op31`** | scene name `sg_kt_track`, same spawn, `sky.xml` |
| +20.155 | 15449 | `41/2` | encounter `917566` / instance `1008`, PlayerEnter |
| +22.796 / +22.947 | 16809 / 16840 | `33/11` | CreateProxiedVehicleRace karts, also carry `sg_kt` |
| +94.427 | 42954 | `op31` | return to world scene `FabledRealms` |

## Raw racing op31 payload (87 bytes)

`reports/original-minigame-packets/racing-13071-flow0-frame723-op31-subNone.bin`,
sha256 `bace4cbaef5d50eb3601543847e77e1b4058ce765405f91556dd386f8e32c373`:

Full captured bytes are preserved only in the private `proof-scene-binding.json` and captured frame named above; this note retains the hash and field map.

Field map (file offsets):

| Off | Type | Value |
| --- | --- | --- |
| 0-1 | u16 | `0x0005` outer |
| 2 | u8 | `1` flag |
| 3-6 | u32 | `0x50` = 80 bytes remaining |
| 7-8 | u16 | `31` op |
| 9-12 | u32 | `11` name length |
| 13-23 | bytes | `sg_kt_track` |
| 24-27 | u32 | `2` |
| 28-31 | f32 | `719.60999` |
| 32-35 | f32 | `72.0` |
| 36-39 | f32 | `421.5` |
| 40-43 | f32 | `1.0` |
| 44-51 | u32 x2 | `0, 0` |
| 52-55 | f32 | `1.0` |
| 56-59 | u32 | `0` |
| 60-63 | u32 | `7` (`sky.xml` length) |
| 64-70 | bytes | `sky.xml` |
| 71-86 | tail | `00 01 00 00 00 00 00 00 00 00 65 00 00 00 01 00` |

The other op31 frames decode the same way: `sg_dm_arena` with
`7.0, 240.59, 317.32` (matches the demoderby `41/114` spawn),
`sg_soccer` with `0, 0, 0` (matches soccer), and two `FabledRealms` returns
with world spawns `-2262.04,-16.47,588.02` and `-2267.94,-16.34,587.82`.
The native Mining run's own world-load packet is `op43/12`
`runs/client-baseline-20261009T134930Z/analysis/application-6578.bin`
(sha256 `c995f3f6ea5d6b05f3f41928dde58019019c63db19bbb21c7a79385ce531c93d`),
also carrying the zone name `FabledRealms` - a second, independent
name-carrying zone packet family.

## Shipped scene geometry entries

`sg_kt_track` is an exact shipped asset name. OSFR full manifest and the local
24-pack catalogue:

| Asset | OSFR manifest id / size | Pack catalogue pack/entry/offset/size/hash-prefix |
| --- | --- | --- |
| `sg_kt_track.gzne` | `561010326` / 169 | `Assets_001.pack` #3217 off 11774451 size 342 `21705696` (`pack_13/003217.gzne`) |
| `sg_kt_track2.gzne` | `43657715` / 197 | `Assets_001.pack` #3218 off 11774793 size 366 `029a29f3` (`pack_13/003218.gzne`) |
| `sg_kt_track2_0_0.gcnk` | `1817237544` / 251770 | `AssetsW_009.pack` #433 off 9597213 size 251770 `6c50d828` (`pack_09/000433.gcnk`) |
| `sg_kt_track2_0_4.gcnk` | `377916271` / 555390 | `AssetsW_009.pack` #436 off 10045911 size 555390 `16868b6f` (`pack_09/000436.gcnk`) |
| `sg_kt_track2_4_0.gcnk` | `1400440366` / 648061 | `AssetsW_009.pack` #438 off 10776205 size 648061 `5379062e` (`pack_09/000438.gcnk`) |

The OSFR manifest and pack sizes differ for the two `.gzne` rows (169/197 vs
342/366); both are recorded, not reconciled. The same family exists for the
other speedways: `bw_kt_track.gzne`, `sh_kt_track.gzne`, `ss_kt_track.gzne`,
`ss_pro_track.gzne`, `ss_pro_track2.gzne`, `gw_racetrack_01.gzne`,
`test_designtrack_bbe.gzne`, and `sg_demo_derby_encounter.gzne`
(`Assets_000.pack` #4809 off 39073603 size 84 `c47a25d9`).

## Community layer and the withdrawn DB lead

* **Withdrawn:** `sanctuary.db` is the Sanctuary emulator's EF Core
  persistence database (entities `DbUser`, `DbCharacter`, `DbItem`, `DbMount`,
  `DbProfile`, `DbTitle`, `DbFriend`, `DbGuild`, `DbGuildMember`, `DbIgnore`;
  SQLite/MySQL providers). No world, zone, scene or encounter tables. The only
  local records are `dbs`.
* Public world data in `raisingkaines/FreeRealms-Legacy` (current `main`) is
  `src/Resources/Zones/FabledRealms.json` (1415 bytes, blob sha
  `5b8866e442646aa0ac8680efc4933a4dfb41ce6a`; downloaded copy sha
  `98b060e5e13ae8fe22628fa58076efb4a1cc960dc316727a9f0321c2d33b5c66`), plus
  POIs/NPCs/profiles. The local copy differs (1785 bytes, sha
  `a049fcfe89b40c331e9c604d613fe8a017c3a6cfe9fad1679b286b6c1ed473d1`); both are
  preserved. The server's only spatial primitive is
  `WorldZone.GetZoneAreaId(position)` over area definitions - no speedway
  areas, no scene/geometry assignment.
* Encounter packets in the public servers are name-table/stub level
  (`BaseEncounterPacket` opcode 41; `PacketReaderExtensions` maps
  2/3/4/5/6/62/102-133; `EncounterOverworldCombatPacket` only). OFR has no
  `917566`, `1310785` or `speedway` strings. Public code search for
  `EncounterDetailsResponsePacket` returns only emulator name tables/forks;
  `917566` and `sh_kt_track_bbec` return no Free Realms repository hit. These
  negatives are bounded to those searches.

## .GZNE reader status (secondary)

The original client loads `.gzne` itself; no custom parser is required to use
the binding. For optional offline inspection: the local community
`ForgeLightToolkit` has only `// TODO: Load worlds using the .gzne` in
`Editor/LoadWorldWindow.cs` and globs `<world>_X_Y.gcnk` tiles instead; the
`forgelight-toolbox` copy contains only pack/index/name utilities. GitHub code
searches for `gzne`, `gzne ForgeLight zone parser` and `GzneFile PlanetSide`
return no ForgeLight/FreeRealms `.gzne` decoder. Any future `.gzne` decode is
therefore a new parser and stays out of scope; the `.gcnk` tile geometry does
have the cached community reader (`lab/tools/cave-community-reader` over
`ForgeLightToolkit` Gcnk/Dme/Dma types).

## karts1.gfx discrepancy (kept)

`Kart Racing Restoration` quoted a `karts1.gfx` size; `Go Kart Native
Integration Readiness` could not reproduce it from the pinned catalogue. The
OSFR manifest now lists `karts1.gfx,3707340300,5064`, while the local 24-pack
catalogue has no `karts1.*` entry. Unresolved and recorded.

## Verification

```
& '<lab>/tools/universal-modder-env/Scripts/python.exe' \
  lab/reports/kart-community-world-binding-20261009T155851Z/probe-scene-binding.py
& '<lab>/tools/universal-modder-env/Scripts/python.exe' \
  lab/reports/kart-community-world-binding-20261009T155851Z/probe-community-world-binding.py
```

The scene probe parses every op31 frame field-by-field (offsets above), reads
the frame ordering from the preserved racing index, and pins the manifest and
pack-catalogue rows; it does not rely on a string grep. The community probe
inventories the emulator schema/resources, the packet tables and the code
searches. Both write `proof*.json` with `write_exact` semantics. Output hashes
are in `final-hashes.txt`.

Cross-links: [[Research/Kart Scene Load Binding]],
[[Research/Kart Speedway Entry Binding]],
[[Research/Go Kart Native Integration Readiness]].
