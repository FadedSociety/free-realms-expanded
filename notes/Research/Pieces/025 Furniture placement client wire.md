# 025 Furniture placement client wire

Status: request bodies recovered from the shipped client and encoded with tests, 2026-10-09 02:49 UTC.
Evidence label: **ORIGINAL client wire for six client-to-server requests**. The server responses for
the same family are **not** recovered and are listed as gaps. There is no native acceptance claim.

Owned paths used: `prototype/osfr-tools/src/restoration/piece_025.rs`,
`prototype/osfr-tools/tests/piece_025.rs`, `scripts/pieces/piece_025.py`,
`tests/pieces/test_piece_025.py`, this note, and
`lab/reports/piece-025-20261009T024934Z/`. No shared `lib.rs`, manifest, gateway, content, save,
launcher or release path was modified.

## Recovered original framing

Every housing packet is a base opcode `127` packet (`BaseHousingPacket`, stamped by
`FUN_00aa4990` at source `+0x04`) with an inner opcode at source `+0x08`. The body writer
`FUN_00aba910` emits the base opcode as a little-endian `u16` and then the inner opcode as a `u16`,
so each datagram begins `7f 00 <inner:u16>`. This framing is read straight out of the shipped
client pseudocode (`lab/decompiled/client/FreeRealms/pseudocode.c`, sha256
`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`).

## Recovered original request bodies

Each row was derived from the writer function that serializes the packet (`FUN_00ac0xxx` send
wrappers call it) and the constructor that stamps the opcode. The byte order is the order the
writer emits.

| Inner opcode | Packet | Body after `7f 00 <op:u16>` | Writer / constructor / send site |
| --- | --- | --- | --- |
| 1 | `ClientHousingPacketPlaceFixtureRequest` | `u32`, `u32`, `u32`, `u8` | `FUN_00abcb90` / `FUN_00aa5340` / `FUN_00ac0400` |
| 3 | `ClientHousingPacketPickupFixture` | `u64` | `FUN_00ac0510` / `FUN_00aa81a0` |
| 5 | `ClientHousingPacketSaveFixture` | `u64`, 4×`u32`, 4×`u32`, `u32`, undecoded block, `u32` | `FUN_00abf620` / `FUN_00aad140` / `FUN_00ac7ce0` |
| 6 | `ClientHousingPacketSetEditMode` | `u8` | `FUN_00abcb30` / `FUN_00aa4cc0` / `FUN_00ac02f0` |
| 19 | `ClientHousingPacketEnterRequest` | `u64`, `u32`, `u8` | `FUN_00abca80` / `FUN_00aa83c0` / `FUN_00ac01e0` |
| 20 | `ClientHousingPacketPreviewByItemDefinitionIdRequest` | `u32` | `FUN_00abcd90` / `FUN_00aa54f0` / `FUN_00ac0a80` |

Supporting facts read from the same export:

* `FUN_008fd1a0(source, writer)` writes one `u64` (it copies two `u32` words in order); it is the
  `u64` emitter in opcodes 3, 5 and 19.
* `FUN_0091a640(writer, source)` writes four `u32` words, i.e. the 16-byte groups in opcode 5.
* In opcode 5 the block after the first `u32` comes from `FUN_00abd1d0`, which writes a `u32`, then
  a string/array helper (`FUN_00911c30`) twice, a `u32`, another `FUN_00911c30`, and returns; the
  writer then emits a final `u32` from source `+0x7c`. The block is left opaque.
* Call sites pair each writer with exactly one constructor: `FUN_00ac4200` and `FUN_00ac8d90`
  build opcode 1 and call `FUN_00ac0400`; `FUN_00ac42d0` and `FUN_00ac8d90` build opcode 3 and
  call `FUN_00ac0510`; `FUN_00ac8790` builds opcode 5 and calls `FUN_00ac7ce0`; `FUN_00ac4160`
  builds opcode 6; the preview UI at `00ac5xxx` builds opcode 20 and calls `FUN_00ac0a80`. All
  six send wrappers are only reached when the client's connected flag at `*(*(process+0x10)+0x280)`
  is non-zero.

## Cross-check against the pinned community model

The pinned community server (`lab/references/FreeRealms-Legacy`,
`src/Sanctuary.Packet/BaseHousingPacket/`) implements readers for exactly two of these packets and
they agree field for field with the client writers:

* `ClientHousingPacketSetEditMode.TryDeserialize` reads one boolean (opcode 6, `u8`).
* `ClientHousingPacketEnterRequest.TryDeserialize` reads `ulong HouseGuid`, `int Unknown`,
  `bool Unknown2` (opcode 19, `u64`, `u32`, `u8`).

That agreement is evidence for the field *widths*; the community names `Unknown`/`Unknown2` mean the
community model does not know their meaning either. Both readers require the body to be consumed
exactly, which matches the writer lengths above.

The community `PlayerHousingInstanceData`/`FixtureInstance` serializers
(`src/Sanctuary.Packet.Common/Housing/`) describe the opcodes 28/39/40 payloads, but they are a
reconstruction, not client evidence. Nothing from them is encoded in this piece.

## Reconstructed policy in this piece (not recovered behaviour)

* The three leading `u32` words of opcode 1 and the trailing `u32` of opcode 5 have no recovered
  meaning and are kept as raw words (`first`/`second`/`third`, `trailing`). No names, units or
  float interpretation were invented.
* `quad_a`/`quad_b` in opcode 5 are four `u32` words each because that is what the writer emits.
  Whether they are floats or packed integers is not established by this evidence.
* The undecoded opcode 5 block is carried as an opaque `tail` so an encode/decode round trip cannot
  silently drop bytes.
* The codec rejects a wrong base opcode, an unknown inner opcode, a truncated body and trailing
  bytes. That validation is local policy, not recovered client behaviour.

## Explicit gaps

These are not implemented and must not be implied:

* no server-to-client body is recovered: opcodes 28 (`InstanceData`), 39 (`InstanceList`),
  40 (`FixtureUpdate`), 41 (`FixtureRemove`) and 53 (`FixturePlacementDenied`) remain undecoded.
  `piece_025::UNRECOVERED_OPCODES` lists them in code so a caller cannot mistake the skip for
  support;
* the opcode 5 nested block (`FUN_00abd1d0` element layout) is not decoded;
* the meaning of the opcode 1 words and of `SaveFixture.value`/`trailing` is unknown;
* no packet has been produced by the client or consumed by it under this codec, so no native
  acceptance exists;
* nothing here validates placement legality, inventory consumption or persistence; that stays with
  items 006/026/027.

## Verification

Pinned lab Python, from the project root:

```
& "$lab\tools\universal-modder-env\Scripts\python.exe" -m unittest discover -s tests/pieces -p "test_piece_025.py" -v
Ran 5 tests in 2.393s
OK

& "$lab\tools\universal-modder-env\Scripts\python.exe" scripts/pieces/piece_025.py --report "$lab\reports\piece-025-20261009T024934Z" --check-rust
{"packets": 6, "opcodes": [1, 3, 5, 6, 19, 20], "pseudocode_sha256": "f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3"}
```

```
.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_025
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test housing --test piece_022 --test piece_025 --test piece_026
housing 26 passed / piece_022 12 passed / piece_025 8 passed / piece_026 12 passed
```

The RED-first cycle is retained in the lab: the first Rust run failed with
`error: couldn't find file tests\..\src\restoration\piece_025.rs`, and the first green attempt failed
one vector (`TooShort` vs `Truncated` for a three-byte frame), which corrected the decoder's
expectation. A later run failed the cross-artifact schema check because the opcode 5 table omitted
the trailing `u32` after the nested block; the module now models that `u32` explicitly and the
Python extractor proves the two tables agree.

The full ordinary Rust suite cannot complete in this session because another worker's in-flight
`tests/piece_027.rs` includes `src/restoration/piece_027.rs`, which does not exist yet; that is not
this piece's failure. Logs are in the lab report directory.

## Next gates

1. Lead integration: register `pub mod piece_025;` (or the agreed restoration module path) and drop
   the `#[path]` include from `tests/piece_025.rs`.
2. Wire the decoded opcodes 1/3/5/6 into `housing.rs` (item 006) so a placement request is answered
   by the reconstructed state machine; inventory proposal/commit stays with item 026.
3. Recover the server responses 28/39/40/41/53 from the client's inbound parsing before any
   placement reply can be serialized. The community model is only a starting hypothesis.
4. Native acceptance (item 030): enter a lot, place, move, pick up and re-enter after restart
   through the actual client. Only that proves the reconstructed rules.

Cross-links: [[Housing Restoration]], [[../WorkItems/025]], [[../MMO Work Queue]],
[[../Design/MMO Worker Contract]].
