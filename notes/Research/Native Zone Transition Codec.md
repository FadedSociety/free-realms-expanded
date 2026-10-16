# Native Zone Transition Codec

Status: implemented, RED then GREEN, compiled-community-writer cross-check and
captured-frame round trip all green. No native claim: no world entry exists yet.

Owned paths: `prototype/osfr-tools/src/zone_transition.rs`,
`prototype/osfr-tools/tests/zone_transition.rs`, this note and the private
bundle `../lab/reports/zone-transition-codec-20261009T161736Z/`. `lib.rs`,
gameplay, gateway, Cargo, UI, save, content and release files were not touched;
the module is unregistered and the test includes it by path.

## Original field layout (first-party native reader)

The client's opcode `0x1f` dispatch case constructs `PacketClientBeginZoning` and
calls the body reader `FUN_008e4ae0`
(`lab/decompiled/client/FreeRealms/pseudocode.c`, reader at `:1029104`, opcode
case at `:1108664`). The reader consumes, in order, into these object offsets:

| # | read | object offset | field |
| --- | --- | --- | --- |
| 1 | `u16` | `+0x04` | opcode (31) |
| 2 | `i32` length + bytes | `+0x08` | zone/scene name (UTF-8) |
| 3 | `i32` | `+0x3c` | type |
| 4 | `4 x f32` | `+0x40` | position |
| 5 | `4 x f32` | `+0x50` | rotation |
| 6 | `i32` length + bytes | `+0x60` | sky name (UTF-8) |
| 7 | `u8` | `+0x70` | tutorial |
| 8 | `u8` | `+0x71` | unknown |
| 9 | `i32` | `+0x74` | id |
| 10 | `f32` | `+0x78` | update radius (native also raises its error flag on NaN) |
| 11 | `i32` | `+0x7c` | geometry id |
| 12 | `u8` | `+0x80` | override update radius |
| 13 | `u8` | `+0x81` | wait for zone ready |

Booleans are single bytes compared `!= 0`. String lengths are `i32` byte counts
with the bounds check the decompiler shows (`-1 < length`, then a bounds test
before the byte advance), so negative and overrunning lengths are refused.

## Envelope and preserved capture

The pinned community writer `Sanctuary.Packet.PacketClientBeginZoning` serializes
exactly the reader's order (strings as `i32` length + UTF-8, `Vector4`/
`Quaternion` as `4 x f32`); `PacketWriter` is the pinned serializer. The captured
frame adds the 7-byte outer envelope (family `5`, one flag byte, `u32` inner
length) around that inner packet.

`racing-13071-flow0-frame723-op31-subNone.bin` (87 bytes, sha256
`bace4cbaef5d50eb3601543847e77e1b4058ce765405f91556dd386f8e32c373`) is the
verbatim `sg_kt_track` frame:

| offset | value |
| --- | --- |
| 0-1 / 2 / 3-6 / 7-8 | `u16 5` / flag `1` / `u32 80` / `u16 31` |
| 9-12 / 13-23 | name length `11` / `sg_kt_track` |
| 24-27 | type `2` |
| 28-43 | position `719.60999, 72.0, 421.5, 1.0` |
| 44-59 | rotation `0.0, 0.0, 1.0, 0.0` |
| 60-63 / 64-70 | sky length `7` / `sky.xml` |
| 71 | tutorial `0` |
| 72 | unknown `1` |
| 73-76 | id `0` |
| 77-80 | update radius `0.0` |
| 81-84 | geometry id `101` |
| 85 | override update radius `1` |
| 86 | wait for zone ready `0` |

Raw game bytes stay in the lab; the repo test carries only these documented
field values. The captured round trip is an ignored, env-gated test
(`OSFR_ZONE_OP31_CAPTURE`).

## Rust artifact

`src/zone_transition.rs` models the complete frame and preserves the unknown
semantics instead of inventing them:

- `encode` refuses strings over 4096 bytes and non-finite position/rotation/
  update-radius floats, writes canonical `0/1` booleans, and computes the
  `u32` inner length from the actual body.
- `decode` requires the 9-byte minimum, family `5`, a declared length equal to
  `bytes.len() - 7`, opcode `31`, non-negative bounded string lengths with valid
  UTF-8, finite floats, and an exact end (trailing bytes are refused). Booleans
  decode as `!= 0`; the envelope flag byte is preserved verbatim.
- `Default` is all zeros with empty strings: the community `Type = 2` default and
  the captured flag are caller-supplied values, and no geometry id is invented.
- Opcode `43` (`PacketSendZoneDetails`) is a different packet family and is
  refused by the opcode check; nothing here forces a sub-12 path.

## Checks

| check | result |
| --- | --- |
| RED before the module existed | `couldn't find file tests\..\src\zone_transition.rs` (`red.log`) |
| `cargo.ps1 test --test zone_transition` | 9 passed, 0 failed, 1 ignored (`green.log`) |
| `cargo.ps1 test --test zone_transition -- --ignored` with `OSFR_ZONE_OP31_CAPTURE` | captured frame decodes, re-encodes byte-exact, sha matches (`green-capture.log`) |
| compiled Sanctuary C# writer oracle vs Rust module (4 vectors) | inner bytes identical for `kt`, `dm_arena`, `utf8`, `defaults`; `kt` full frame equals the capture (`oracle-comparison.json`) |
| `rustfmt --edition 2024 --check` on both owned files | exit 0 |

## Limits

- No runtime world entry, gateway wiring or scene loading; the module stays
  unregistered until the lead integrates it.
- The `unknown` byte and the true gameplay meaning of `id`, update radius and
  geometry id remain unrecovered; only their wire positions are modelled.
- The kart capture evidence remains community-sourced for the field values; the
  field order, tail offsets and string/bool semantics are first-party from the
  native reader above.
