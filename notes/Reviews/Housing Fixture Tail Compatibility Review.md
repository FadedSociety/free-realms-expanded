# Housing Fixture Tail Compatibility Review

Independent read-only review of `notes/Research/Housing Fixture Tail Compatibility.md`
(`7CCA7928…`) and `lab/reports/housing-fixture-tail-compatibility-20261009T2000Z/`. Writes are
this note and `lab/reports/housing-fixture-tail-independent-review-20261009T170946Z/` only; no
source, test, content, save, UI, process or worker artifact changed. Skills: RTK (normal
commands), Ponytail (own parser/oracle, no new dependencies), Caveman (terse), reverse-engineering
(the pseudocode anchors).

## Verdict

**Worker claim independently reproduced and strengthened.** All nine captured `127/40` records
decompose as 2-byte subtype + 133-byte `FixtureInstance` + 94-byte tail, and the tail is
`FixtureInstanceInfo` 28 + `FixtureDefinition` 50 + native trailer 16 (array count 4 + three `u32`
12) = 94, with cursor remainder 0. My own raw-pcap parser, my own re-encoder and the pinned native
reader chain all agree byte-for-byte, and the native chain consumes the **entire** child payload
(`len(child) - 7` = 231 bytes), not just the tail. The unchanged community sources are byte-identical
to the upstream Sanctuary repo copies, and the worker's prebuilt oracle re-ran here read-only:
`9/9 community tails byte-identical (16-byte native trailer excluded)`, `first_diff=-1`.

## Independent re-derivation (different helper)

My own pure-stdlib helper (`verify_tail.py`: raw libpcap framing, IPv4/UDP, transport
header/flag/2-byte-trailer, length-prefixed child split — no scapy, no lab decoder import):

- `p2.pcap` `65AC0801…` (26,753,651 B, 83,223 frames) → 53,055 UDP frames → 23,578 op-9
  messages, 2,526 op-13 fragments.
- **9 × `127/40`** in op-9, all server→client (`64.37.170.6:20267` → `192.168.1.145:61759`),
  frames 31507, 31919, 32023, 32098, 32196, 32282, 32342, 32578, 32623 (same set as the report).
  Reliable channel: 0 multiplexer-framed fragments and 0 `127/40` signatures, so the nine is the
  complete captured set across both channels.
- Each child is 238 B = 7-byte tunnel header + 231. Cursor: `child[9:]` = 2 subtype + 227 payload;
  133 + 94 = 227; 133+28+50+16 = 227; **remainder 0** on all nine.
- My extracted body/tail sha256 and tail hex match the reviewed `tail-report.json` for all nine
  frames (body match 9/9, tail match 9/9); the nine tails are distinct; `fixture_guid` equals the
  133-byte record guid in all nine.
- My own encoder re-emitted prefix+tail byte-identical for all nine (independent of the C#
  oracle).

## Native anchors (machine-checked against `pseudocode.c` `F84A5867…`)

| Reader | Lines | Bytes it reads |
| --- | --- | --- |
| `FUN_00ab2bc0` | 1395161-1395184 | two `i16` (family 127 + subtype 40, matching the tunnel header) = 4 |
| `FUN_00ab6f40` | 1398599-1398783 | 8+8+4+4 + 3×16 (`FUN_008e2410`) + 8+4+4 + `FUN_008e8510` (4+str+str+4+str) + str+str+4+str+4+1+4 = **133** |
| `FUN_00aafd30` | 1392498-1392554 | 8, 4, 4, `FUN_00aafa90`, 4, 4 = **28** |
| `FUN_00aafa90` | 1392338-1392383 | `i32` count + count × 8-byte items |
| `FUN_00abeb40` | 1404603-1404812 | 4×`i32`, 6 bools, 2 strings, map (`FUN_00aba9e0`), `i32`, 2×`f32`, 4 bools = **50** |
| `FUN_00aba9e0` | 1401328-1401376 | map count, then per entry key then value (`FUN_00ab0340` = `i32`,string,`i32`,`i32`) |
| `FUN_00ac2180` | 1406991-1407028 | call order 2bc0 → 6f40 → aafd30 → abeb40 → aafa90(+0x1c) → three `u32` |
| `FUN_00894b10` | 966729-966760 | string assign of the already-read byte count (length-prefixed UTF-8) |

`FUN_00ac2180` therefore consumes `4 + 133 + 28 + 50 + 4 + 12 = 231` bytes = `len(child) - 7`,
ending exactly at the child end; my byte model hits the markers 11 / 144 / 172 / 222 / 226 / 238 on
all nine records. The `FUN_00ab6f40` read order maps 1:1 onto the community `FixtureInstance` field
order (guid, house guid, definition id, unknown4, position/rotation/tilt, npc guid, tint,
unknown10, customization 4+str+str+4+str, str, str, `i32`, str, `f32`, `bool`, `i32`), and
`FUN_00aba9e0`'s key-then-value map order matches the community dictionary writer for non-empty
actions too.

## Synthetic non-empty vector (own oracle, upstream sources byte-identical)

`synth/` compiles the unchanged community serializers (hashes equal the Sanctuary repo and the
reviewed oracle copies) with my own runner:

- `non_empty_all` (2 npc guids, `Category "Parlor"`, `LuaCall "fixture.parlor"`, 2 actions): 162 B,
  info 44 + definition 118; parsed values equal the emitted values, re-encode byte-identical,
  remainder 0.
- `actions_only` (empty strings/list, 2 actions): 127 B, definition 99; same result.
- The reviewed Python parser's action loop (which omits the map-entry key) desynchronizes on these
  vectors: `non_empty_all` yields garbage names (`"\x03\x00\x00\x00sit"`), `actions_only` raises
  `short read at 89 need 1280`. This matches the note's own qualification — its variable-length
  paths have no captured vector; the private Python parser is empty-path only.

## Identity / overclaim check

- The nine pairs reproduce exactly: `7466→34` ×2, `14300→902`, `77235→466`, `77259→490`,
  `7462→323`, `16039→68`, `18110→1074`, `16040→69`. Each is a **within-record** co-occurrence
  (instance `item_definition_id` == definition `ItemDefinitionId`, plus the definition's own `Id`).
  No global item/definition identity is asserted by the note or proven here; world item identity
  stays out of scope.
- `CharacterGuid` is a label, not an anchor-proven identity: the name is supported by nearby
  `SoeUtil::Array<StrongType<u64, SoeUtilSignatureCharacterGuid>>` type text (e.g. line 1393180)
  but is not tied to the `FUN_00ac2180` call site by an anchor.
- The 28/50/94 sizes are this capture's **empty-path** sizes; both blocks are data-dependent
  (npc list / strings / actions). The note states this ("in this capture", "empty
  Category/LuaCall/actions") and the concrete claim holds for all nine records.

## Findings

1. (doc, private report) `tail-report.json` `native_cursor_proof.reader_chain` says
   `FUN_00aafa90 (1402340 region)` while the same file's `anchors` array and the note say
   `1392340-1392382` (header at 1392338). Off-by-10000 typo only; no functional impact.
2. (doc) `CharacterGuid` name as above: structure/widths proven, name plausible-only.
3. (boundary, confirmed correct) Empty-list/string-only capture, no native acceptance, and the
   transport decode inherits the pinned piece-022 model (`crc_seed_recovered=false` there too);
   my verification adds no CRC claim.

## Artifacts

`lab/reports/housing-fixture-tail-independent-review-20261009T170946Z/`: `verify_tail.py`,
`independent-report.json`, `tail-independent.log`, `native-anchor-excerpts.txt`, `synth/`
(`Synth.csproj`, `SynthProgram.cs`, byte-identical `src/`), `synth-build.log`, `synth-run.log`,
`synth-report.json`, `synth-actions.json`, `synth-check-synth-report.json`,
`synth-check-synth-actions.json`, `synth-check-all.log`, `synth-check-actions.log`,
`worker-oracle-rerun.log`, `proof-manifest.json`, `final-hashes.txt`.
