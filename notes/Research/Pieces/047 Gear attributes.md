# 047 Gear attributes

Queue item 047. Acceptance: recover item stats and profile restrictions; no fabricated balance
values hidden as historical.

Status 2026-10-09 04:15 UTC: the recovered character stat id space, the item stat wire shape, the
two item rows that actually carry a stat vector, their decoded values and the class/profile
restriction rows are recovered and tested. Combat gear stats, per-item stat types, two native stat
slots and the `ItemClasses.StatId` meaning are **not** recovered and are recorded as explicit gaps.
No native acceptance was attempted and the module is not registered in `lib.rs`.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_047.rs` | Recovered 69-value stat table, item stat decoder, item stat vectors, class rows and the reconstructed equip gate (new) |
| `prototype/osfr-tools/tests/piece_047.rs` | 8 integration vectors; includes the module by `#[path]` until the lead registers it (new) |
| `scripts/pieces/piece_047.py` | Read-only checker over the pinned client item table, the recovered stat classes, the class/profile/mount tables, the preserved locale and the native pseudocode (new) |
| `tests/pieces/test_piece_047.py` | 7 stdlib tests: four hermetic fixture cases plus three real-corpus pins (new) |
| `../lab/reports/piece-047-20261009T041533Z/` | RED/GREEN logs, `piece-047-gear-attributes.json`, `summary.md` |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Recovered original facts

1. **The character stat id space is 69 ordered values, ids 0..68.**
   `CharacterStatId.cs` lists the enum and `CharacterStats.cs` gives the default of every id except
   `InvalidStat` (0). The default literal form is the recovered wire kind: 44 ids are int
   (`MaxHealth`, `Defense`, `Luck`, `GlideEnabled`, ...) and 24 are float (`MaxMovementSpeed`,
   `WeaponRange`, `DamageMultiplier`, `GlideDefaultForwardSpeed`, `JumpHeight`, ...).
2. **The item stat wire shape is `Id`, `Type`, `Value`, `Unknown`.**
   `BaseItemStatDefinition` carries `Id` and `Type`; `ClientItemStatDefinition` adds the value and
   the bool. The serializer writes a 32 bit int when `Type == 0` and a 32 bit float when
   `Type == 1`, then the bool. The base class comment calls those types additive/multiplicative
   while the same field switches the value width; the two readings are recorded as an unresolved
   contradiction rather than merged.
3. **Only two of the 58,800 pinned item rows carry a stat vector.**
   Item 18092 `Parachute` (`Class` 158, `Slot` 6, `CategoryId` 106, no `MinProfileRank`, no
   `MembersOnly`) carries 7 stats, and item 78397 `The Fire Hawk` (`Class` 172, no `Slot` field,
   `CategoryId` 3, `Param1` 13459) carries 8. The map key equals the stat id in every recovered row.
4. **The stored values are IEEE-754 float32 bit patterns and decode to coherent glide numbers.**
   Parachute: 59 `GlideDefaultForwardSpeed` 8, 60 `GlideMinForwardSpeed` 2, 61
   `GlideMaxForwardSpeed` 18, 62 `GlideFallTime` 0.75, 63 `GlideFallSpeed` 4, 64 `GlideEnabled` 1,
   67 `GlideAccel` 4. Fire Hawk: the same ids at 12/6/22/0.75/3/1/4 plus 68 `JumpHeight` 5. Reading
   the stored integers directly instead of decoding the pattern yields absurd values
   (1090519040), so the decode is the only reading that satisfies the data.
5. **The Fire Hawk vector has an independent second source.**
   Item 78397's `Param1` is 13459, which is mount row 13459 `The Fire Hawk` in `Mounts.json` with
   the same `NameId` 442674 and the identical eight glide values (plus `MaxMovementSpeed` 12.5).
   The parachute has no bound mount row, so its vector has one source and only the internal
   ordering check.
6. **The native client has the same objects and a wider stat container.**
   The decompiled pseudocode carries `BaseItemStatDefinition::vftable`, `ItemStatDefinition::vftable`
   and a `SoeUtil::HashListMap<int,ItemStatDefinition,71,-1>` instantiation: 71 declared slots for
   69 recovered ids. The two extra slots are not identified.
7. **Class rows name their owning profile or a shared scope.**
   `ItemClasses.txt` fields are `Id^NameId^IconId^WieldType^StatId^ProfileNameId`, confirmed by the
   community reader `ItemClassDefinition.cs`. Class 2 is `Ninja Gear`, profile name id 3100
   (`Ninja`), and profile 2 lists class 2 - a job-owned gear class. Classes 50 (`Any Combat Job`),
   69 (`Any Job`), 158 `Parachute` (`All Jobs`) and 172 `Ride` (`Ride`) resolve to scope strings
   that no profile row uses as a `NameId`, so they stay scope rows with no owner.
8. **Restriction field coverage over the pinned table.**
   58,800 item rows, 184 class rows. `MinProfileRank` is present on 57,889 rows (absent 911) with
   range 1..20; `ProfileOverride` appears on 10 rows (Penguin Badge 1; Robot Outfit 3; Werewolf
   Outfit 6); `ClientEquipReqSetId` appears on 3 rows (Costume Tree Outfit, Teddy Bear Costume x2,
   all value 25); `MembersOnly` 926; `SingleUse` 1,207; `NoTrade` 10,575; `NoSale` 7,409;
   `PowerRating` 20,338; `GenderUsage` 3,836 male / 4,163 female. 5,005 rows have no `Class` field
   and every row that has one resolves to a class row.

## Reconstructed policy

`piece_047::equip_gate` is policy over the recovered rows, not a recovered code path: a class whose
profile name id names a job is owned by that job; a recovered `MinProfileRank` is a rank floor; a
recovered `MembersOnly` flag needs an active membership; a class with no recovered row is refused
instead of assumed usable. The pinned sources contain no equip rejection handler, so nothing here
claims the original server behaved this way.

## Explicit gaps

- No combat gear stat value exists anywhere in the pinned table: no item carries Defense, damage,
  crit or mitigation numbers. No combat balance value is recovered, and none is invented.
- 2 of 58,800 rows carry stats; every other item's stats are unrecovered.
- The pinned extraction keeps `Stats.<key>.Id` and `Stats.<key>.Value` but never the item stat
  `Type`, so additive/multiplicative is unrecovered per item.
- The native 71 slot container has two unidentified slots.
- The meaning of the `ItemClasses.txt` `StatId` column for gear classes (recovered values 16, 19, 0)
  is unresolved; no recovered reader uses it.
- Both recovered stat rows omit `MinProfileRank`, and no recovered reader's default is pinned.
- Native acceptance: no client run was made for any recovered item stat.

## Checks

Run from `project/` with the preserved lab present:

    & ..\lab\tools\wiki-env\Scripts\python.exe tests\pieces\test_piece_047.py -v     # 7 tests, OK
    & ..\lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_047.py --output <lab report>\piece-047-gear-attributes.json

From `project/prototype/osfr-tools`:

    & ..\..\scripts\cargo.ps1 test --test piece_047                            # 8 passed

The checker fails closed (exit 2) on enum/table drift, a non-float32 decode, broken glide
ordering, a mount row that disagrees with the item vector, an unresolved class or locale row, a
native container smaller than the recovered id space, and any drift between the recovered rows and
the Rust module constants. The four fixture tests exercise those failure paths on a synthetic
corpus.

## Source provenance

Pinned community sources (`lab/references/Sanctuary`, revision
`4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0, inspected as facts):
`src/Resources/ClientItemDefinitions.json`, `ItemClasses.txt`, `Profiles.json`, `Mounts.json`,
`src/Sanctuary.Packet.Common/ItemClassDefinition.cs`, `BaseItemStatDefinition.cs`,
`ClientItemStatDefinition.cs`, `CharacterStat/CharacterStatId.cs`, `CharacterStat/CharacterStats.cs`.
Preserved locale export `lab/reports/locale-id-to-text.json`. Native evidence
`lab/decompiled/client/FreeRealms/pseudocode.c`. The artifact records the sha256 of every input.
