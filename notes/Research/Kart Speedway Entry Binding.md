# Kart Speedway Entry Binding

Read-only research, 2026-10-09 (UTC). Scope: pin the captured native
`41/114` (encounter `917566` / instance `1008`) and `41/2` (player-enter)
transition, compare the five speedway POIs against the original assets and the
OSFR community databases, and state either a credible minimum native entry
candidate or the specific missing binding. No source, content, save, UI,
release or process change; nothing was launched. Packet framing is reused from
`Go Kart Native Integration Readiness.md` and `Kart Lobby Definition Framing.md`
and is not repeated here. Wiki facts are secondary; locale ids resolve in both
the locale table and the community DB.

Owned artifacts:

| Path | Role | sha256 |
| --- | --- | --- |
| this note | findings | self hash in `final-hashes.txt` |
| `lab/reports/kart-speedway-entry-binding-20261009T154437Z/decode-kart-entry.py` | reproducer | `d6f29983baee24338d9ef3093d3589e58c59a439f8859fb3a377bf2bee898e6c` |
| `lab/reports/kart-speedway-entry-binding-20261009T154437Z/proof.json` | decoded proof | `82b9686289f0eb55599041a9cf31048eb7a21a0896ea85734d913530aea63c56` |
| `lab/reports/kart-speedway-entry-binding-20261009T154437Z/run.log` | run output | `2f5789acfeb2a4a60f361db68708c9e22b73d6928c72f91eeccff45152158b0e` |

## Captured 41/114 + 41/2, pinned

The racing capture (`packets_racing_minigame.pcapng`, community test server,
player session `0x4b52cc6a9573ca91`) carries one racing encounter:

| Frame (time) | Bytes | op/sub | encounter | instance | state | spawn f32 | const | trailing u64 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| frame 135, t=1396310016.9947 | 418 | 41/114 | `917566` | `1008` | `2` | `719.610, 72.0, 421.5` | `0xde2106fb` | session `0x4b52cc6a9573ca91` |
| frame 15449/15450, t=1396310037.1499 | 398 | 41/114 | `917566` | `1008` | `4` | `719.610, 72.0, 421.5` | `0xde2106fb` | session `0x4b52cc6a9573ca91` |
| frame 15449, t=1396310037.1499 | 28 | 41/2 | `917566` | `1008` | - | - | - | 8 zero bytes |

Body map (tunnel header `05 00 01 | u32 remaining`, then inner):
`op(2) sub(2) encounter i32(4) instance i32(4) encounter-repeat i32(4)
state i32(4) flag i32(4) flag i32(4) const u32(4) session u64(8) pair u32(4,4)
f32 x3 spawn f32(4) scale(1.0) u32(4) 8-byte name field` + zeroed blocks +
a tail list. The tail decodes to `[5, 2194, 2919, 2920, 2921, 3752]`; the
locale names those `2194` "Broken Bixie bucket", `2919` "Demo Derby Driver",
`2920` its description, `2921` "Freestyle Gear" - reward/name ids, not a zone.
The `41/2` tail is all zeros on first entry and carries the same u64 session id
on later sends (frames 21639 racing, 13827 derby, 6300 soccer).

Cross-check against the other two minigame captures: demoderby `41/114` is
encounter `1310785` / instance `1008`, state `2`, spawn `7.0, 240.59, 317.32`;
soccer is encounter `458824` / instance `1008`, state `2`, spawn `0,0,0`. The
`0xde2106fb` constant, the 256/256 pair and the u64 session are identical in
all three, so the constant is a shared minigame id, not a per-speedway id;
its exact meaning stays unrecovered.

## Five speedway POIs (community DBs agree byte-for-byte on position)

`Sanctuary/src/Resources/PointOfInterests.json` and
`ofrserver/Customize/PointOfInterestDefinitions.json`:

| Id | Name (locale) | LocationId | TeleportLocationId | BreadcrumbQuestId | Position | Heading |
| --- | --- | --- | --- | --- | --- | --- |
| 31 | Wildwood Speedway (`18206`) | 2569 | 210045 | 1801 | -2267.635, -16.33359, 585.4462 | -1.3558 |
| 32 | Thunder Falls Speedway (`3606`) | 202678 | 210046 | 1803 | -915.60803, -39.1328, 1203.7272 | 0.73532 |
| 33 | Frost Ridge Speedway (`382855`) | 15729 | 210047 | 1802 | -233.04828, 1.18629, -50.43592 | 1.50844 |
| 34 | Tangletrack Speedway (`382856`) | 202681 | 210048 | 1804 | -273.78687, -58.82937, 1951.8916 | 2.2423 |
| 39 | Sandstorm Speedway (`401623`) | 208369 | 210053 | 1880 | 34.4962, -14.7574, -1188.9478 | 0.87331 |

All five share `SubNameId` 382845 "Press to Teleport...", `IconId` 14 and
`NotificationType` 6. The captured `41/114` spawn triple matches none of these
positions, and no captured details frame contains any POI id, teleport id, name
id or POI float (reproducer `poi_id_hits_in_details`: all empty), so the
capture alone cannot bind the encounter to a specific speedway.

Original assets (pinned 24-pack catalogue, `lab/manifests/rust-asset-catalog.tsv`)
carry the world raceway and kart-scene side: `sg_raceway01.cdt` /
`sg_raceway_01_lod0.dme` / `sg_raceway01_flag01_lod0.dme` /
`sg_raceway_tollbooth01.dds` / `sg_raceway_speedster01/02_lod0.dme`,
`sg_billboard_raceway_wildwood.dds` (+ `_fr_FR`), `..._thunderfalls.dds`
(+ `_fr_FR`), `..._frostride.dds` (only three named billboards),
`sh_kt_track_0_0` / `_4_0` / `_12_0` / `_12_4.gcnk` +
`sh_kt_track_bbec_313.dme` (kart track scenes), `kt_kart07_body_bbe.cdt` /
`kt_kart08_*` bodies, `racetrackloop.ogg`. No encounter/zone definition asset
for the speedways is identified by name, and the community DBs contain no
entry for encounter `917566` or `1310785`.

## SDK/test capture vs retail content

The captured `102/2` lobby catalogue is the test catalogue (row 0
`1/1/1/37/3030` = "Test Lobby Game", rows 4-6 placeholder) from the same
community test server that produced the racing/derby/soccer `41/*` traffic;
it is framing evidence, not retail catalogue content. Retail-side evidence used
here is the captured retail self packet (`ofrserver/Customize/PacketSendSelfToClient.json`,
sha256 `8e0948344311e445eb4bf9396f6b1d773c88d75e401d8e60dea51921b5c998ea`) whose job blocks pin **Kart Driver** (JobGUID `48`,
JobName `2917`, description `2918`, EnableAbility `5`, JobLevel `20`) and
**Demo Derby Driver** (JobGUID `49`, JobName `2919`, EnableAbility `7`,
JobLevel `20`), plus the retail asset packs. Wiki (secondary): Kart Driver
trainers Mustang Alli (Frost Ridge), Dane Blueshift (Thunder Falls), Johnny
Thunder (Wildwood), Cale Yardbird (Briarwood/Tangletrack); tracks Frost Ridge
(Snowhill), Thunder Falls (Lakeshore), Tangletrack (Briarwood), Wildwood (near
Sanctuary), Sandstorm (Seaside); "Chase Wheeler ... stands next to the
billboard on the west side of the gate entrance" at Thunder Falls Speedway
(Kart Driver quest series).

## Minimum entry candidate and the missing binding

Credible minimum native entry, bounded to what the sources actually carry:

1. World trigger candidate: one of the five speedway POIs (Community DB, both
   tools agree). The single best-evidenced POI for an entrance interaction is
   **Thunder Falls Speedway** (POI 32, LocationId 202678, TeleportLocationId
   210046, quest 1803) because the only wiki-named entry-side NPC (Chase
   Wheeler) stands at its gate/billboard; the other four have trainers only.
   This is a ranked candidate, not a proven mapping.
2. Profile candidate: the retail **Kart Driver** job (`JobGUID 48`,
   `EnableAbility 5`, `JobLevel 20`), with Demo Derby Driver (`49`) as the
   sibling racing job; both are retail send-self data.
3. Encounter entry: `41/114` (encounter `917566`, instance `1008`, state `4`
   at the start frame) followed by `41/2` with the encounter/instance pair -
   the lifecycle the reconstruction already emits for the cave, except that
   the reconstructed `41/2` writes `u64 = 1` (`src/encounter.rs:458-461`)
   while the minigame capture writes 8 zero bytes on first entry and the
   session u64 on later sends, and the reconstructed `41/114` zero-fills the
   const/u64/spawn fields (`src/encounter.rs:1055-1093`). Those are concrete
   input differences for a future wiring, not a defect claimed here.

**Missing binding (not invented):** no captured or community value ties
encounter `917566` (or the shared `0xde2106fb` constant) to a speedway POI,
zone instance, scene/geometry id or spawn transform. The `41/114` spawn triple
is an instance-space spawn, not a world POI; the community DBs carry
POI/teleport/quest/job rows but no minigame or encounter ids; the retail packs
carry `sg_raceway*`, the three named raceway billboards and `sh_kt_track*`
scenes but no recovered POI->scene table. Zone id, collision id and finish
binding stay unrecovered and are not asserted.

## Verification

```
& '<lab>/tools/universal-modder-env/Scripts/python.exe' \
  lab/reports/kart-speedway-entry-binding-20261009T154437Z/decode-kart-entry.py
```

The reproducer re-derives every field above from the pinned frames, asserts
the POI-id/position byte search is empty for all captured details frames,
extracts both retail job blocks from the pinned self packet, resolves the
locale names, reads the wiki pages as secondary facts, and writes `proof.json`
with `write_exact` semantics (re-running is byte-identical).
Input hashes are recorded inside `proof.json`; key ones:
`racing-12959-...op41-sub114` `7fbc0afce7649b7ce22d7e35e3dc93d325f6adec3b0d04e477d44e8523aaf936`,
`racing-13236-...op41-sub114` `4096e47b547321af6ce0b944148baa4d1c9f2bb63d093171a0aae1f298119496`,
`racing-13237-...op41-sub2` `4dec121872675d6dc6cb2417158f9e606aed64d268410e71d1050445255bd547`.

Cross-links: [[Research/Go Kart Native Integration Readiness]],
[[Research/Kart Lobby Definition Framing]], [[Research/Kart Racing Restoration]].
