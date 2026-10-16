# Housing fixture tail compatibility

Bounded compatibility test of the 94-byte `127/40` fixture-update tail against the pinned community
structures, 2026-10-09 (UTC). Reviewer owns this note and
`lab/reports/housing-fixture-tail-compatibility-20261009T2000Z/` only. The previous note
`notes/Research/Housing Placement Responses.md` is untouched (`64F9854A…`); no src/test/content/
save/UI edit. Skills: RTK (normal commands), Ponytail (two small reuse-only checkers), Caveman
(terse report), reverse-engineering (the reader anchors are decompile claims).

**Verdict: the hypothesis is confirmed and refined — the 94-byte tail decomposes exactly as
`FixtureInstanceInfo` 28 bytes + `FixtureDefinition` 50 bytes + native trailer 16 bytes = 94 for all
9 captured records. The lead's "possible FixtureDefinition 66 bytes" is not what this capture
shows: the definition is 50 bytes here (empty Category/LuaCall/actions), and the last 16 bytes are
the native reader's empty `CharacterGuid` array count (4) plus three `u32` (12) — all zero in this
capture. The community structures are byte-compatible with the captured tails; the 16-byte trailer
is outside the community classes and is covered by the native reader chain.**

## Whole-94 decomposition (units pinned)

Analyzer body `127/40` = 229 = subtype `u16` (2) + fixture record (133) + tail (94); the payload
after the subtype is 227 = 133 + 94 (231 with the base opcode 127). The tail:

| Tail offset | Bytes | Structure | Native reader anchor |
| --- | --- | --- | --- |
| 0 | 28 | `FixtureInstanceInfo`: fixture guid `u64`, item definition `i32`, `CouplingDisplayData` (Id `i32`, npc count `i32`, `u64` list, CompositeEffect `i32`, EffectType `i32`) | `FUN_00aafd30` (1392500-1392553): `u64`, `u32`, `u32`, `i32` count, `u32`, `u32` |
| 28 | 50 | `FixtureDefinition`: Id, ItemDefinitionId, Unknown3, ModelId (4×`i32`), 6 bools, Category string, LuaCall string, Actions count+entries, CompositeEffectId `i32`, Unknown14/15 `f32`, 4 bools | `FUN_00abeb40` (1404605-1404811) |
| 78 | 4 | CharacterGuid array count (`i32`, 0 here) | `FUN_00aafa90(reader, +0x1c)` (1392340-1392382) |
| 82 | 12 | three `u32` (0, 0, 0 here) | `FUN_00ac2180` tail (1407001-1407027) |

## Checks

`tail-compat.py` re-derives all nine `127/40` bodies from the pinned `p2.pcap`
(`65AC0801…`) and parses the tail with the community order and an exact cursor: **9/9 complete,
remainder 0** (`tail-compat.log`, `tail-report.json`). Per record, `fixture_guid` equals the
133-byte record's guid, `item_definition_id` pairs with the definition (`7466→34`, `14300→902`,
`77235→466`, `77259→490`, `7462→323`, `16039→68`, `18110→1074`, `16040→69`), and the trailer is
`{array_count: 0, words: [0,0,0]}`.

The unchanged C# oracle (byte-identical copies of `FixtureInstanceInfo.cs`, `CouplingDisplayData.cs`,
`FixtureDefinition.cs`, `PacketWriter.cs`, `PacketReader.cs`, `ISerializableType.cs`,
`IDeserializableType.cs`; hashes in `final-hashes.txt`) re-encodes the parsed values and prints
**`oracle: 9/9 community tails byte-identical (16-byte native trailer excluded)`**, with
`first_diff=-1` and `native_trailer=16` on every record (`oracle.log`).

## Boundaries

The capture exercises only empty coupling/strings/actions (`npc_count 0`, `category ""`,
`lua_call ""`, `actions 0`), so the community serializers' variable-length paths (non-empty
Category/LuaCall/actions, non-empty npc lists) are pinned by source order and the compiled oracle
but have no captured vector. This is capture compatibility, not native acceptance: no client ran
against a reply built from this decomposition, and the earlier `127/2` first-dword identity and
`127/53` extra-field blockers are unchanged. Raw bytes/decompiled code remain lab-only.

## Artifacts

`lab/reports/housing-fixture-tail-compatibility-20261009T2000Z/` — `tail-compat.py`,
`tail-compat.log` (9/9 complete cursor), `tail-report.json`, `oracle/` (unchanged community
sources + private runner + `Oracle.csproj`/`Program.cs`), `oracle.log` (9/9 byte-identical),
`final-hashes.txt`. Commands: `wiki-env\\Scripts\\python.exe tail-compat.py`;
`lab/tools/dotnet/dotnet.exe run --project oracle -- <report dir>`.
