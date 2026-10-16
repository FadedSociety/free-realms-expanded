# Soccer Native Wire Repair

Scoped repair, 2026-10-09 (UTC). Owned files: `prototype/osfr-tools/src/soccer.rs`,
`src/soccer_match.rs`, `tests/soccer.rs`, `tests/soccer_review.rs`,
`tests/soccer_match.rs` and the new `tests/soccer_native_wire.rs`.
`lib.rs`, gateway, gameplay, content, save, UI, release and Cargo were not
touched. Evidence source: `notes/Research/Soccer Kick Pass Wire Mapping.md`
revision 3 and its independent review, re-checked against the decompiled
native reader/constructor offsets with a private probe.

Pre-repair hashes (for the record): `src/soccer.rs` `2c18c712…`,
`src/soccer_match.rs` `a1406892…`. Final hashes are in the receipt and
`final-hashes.txt`.

## Corrected layouts (revision 3 authoritative)

Sub 14 `UpdatePlayerState` - 62 bytes, reader `FUN_00b794b0`, ctor
`FUN_00b730a0(0xe)` + `SoccerPacketUpdatePlayerState::vftable`:

| Wire offset | Type | Field |
| --- | --- | --- |
| 0x00 | `u64` | player guid |
| 0x08 | `u8` | soccer state code (client enum, 16-34 kick/pass family) |
| 0x09 | `u8` | aux byte (unmapped, preserved verbatim) |
| 0x0A | `u32` | word read from `player+0xB0`; **not** a verified sequence or replay clock |
| 0x0E, 0x1E, 0x2E | `f32[4]` x3 | typed-but-unmapped vectors; never used to infer a direction |

Sub 16 `UpdatePlayerControls` - 20 bytes, reader `FUN_00b793c0`, ctor
`FUN_00b730a0(0x10)` + `SoccerPacketUpdatePlayerControls::vftable`:
`u64` guid @0x00, `f32` x @0x08, `f32` y @0x0C, `u32` control bits @0x10.

Sub 13 `SoccerPacketImpulseBall` - 36 bytes, reader `FUN_00b79580`, ctor
`FUN_00b730a0(0xd)`: `u32` @0x00, `f32[4]` @0x04, `f32` @0x14, `f32` @0x18,
`u32` @0x1C, `f32` @0x20. No captured vector exists; labels beyond the widths
stay inference.

## Repairs

* Sub 14 no longer reads a merged `u32` state plus a 34-byte body: it exposes
  `u8` state, `u8` aux, the verbatim `unknown_word` and the three vectors, and
  re-encodes byte-exact.
* Sub 16 now keeps the control dword as `u32` (previously the third direction
  float). Raw patterns `0/1/2/0x11/0x41/0x100/0xFFFF_FFFF` survive decode and
  re-encode untouched and are never classified as kick/pass.
* A named sub-14 Kick/Pass state becomes a typed, untrusted
  `StateRequest { player, sequence, state, aux, unknown_word, action, vectors }`
  through `client_packet`. `client_intent` refuses it with the explicit
  adapter-gap error - the gateway must supply the authoritative facing; the
  module never derives a direction from the aux byte or the unmapped vectors
  and never translates the state as Move. Non kick/pass states stay read-only.
* Sub 13 now decodes and round-trips its recovered 36-byte layout but is still
  refused as a client request: no authenticated client-to-server sub-13 send
  was observed and the located evidence is server-authored. No impulse is
  accepted from a client and no ball transform is trusted.
* Authority movement/time/economy rules are unchanged; kicks and passes remain
  in-process server intents (`Intent::Kick`) with possession, power, distance
  and server-clock checks untouched.

## Verification

Test-first RED (preserved `red.log`): sub-14 state byte surfaced as
`4027394577` instead of `17`; sub-16 control bits surfaced as `9.1e-44`
instead of an integer; sub-13 stayed `Unknown`. All three fail before the fix.

GREEN after the repair:

| Check | Result |
| --- | --- |
| `cargo test --test soccer` | 11 passed, 0 failed |
| `cargo test --test soccer_match` | 27 passed, 0 failed |
| `cargo test --test soccer_review` | 10 passed, 0 failed |
| `cargo test --test soccer_native_wire` (new) | 8 passed, 0 failed |
| private captured round trip | sub 14 250/250 and sub 16 1216/1216 byte-exact, 0 skipped |
| private source-offset probe | constructors `0xd/0xe/0x10`, reader offsets and vector reads all match revision 3 |

## Limits

The aux byte, the `player+0xB0` word and the three sub-14 vectors keep their
wire positions and names only; no gameplay meaning is claimed. Sub-13 field
labels remain inference, and no captured sub-13 vector exists. The module is
still unregistered (no `lib.rs`/gateway integration), so no native world entry
or client kick is tested; that stays with the lead. Independent acceptance is
the next step.

## Artifacts

Private bundle `lab/reports/soccer-native-wire-repair-20261009T16/`:
`red.log`, `green.log`, `captured-roundtrip.log`, `native-offsets.log`,
`native-offsets-proof.json`, `probe-rust/`, `probe-native-offsets.py`,
`repair-receipt.json`, `final-hashes.txt`.

## Cleanup checkpoint (2026-10-09, synthetic fixtures + full corpus gate)

Scope of this checkpoint: the four owned test files plus the private report;
`src/soccer.rs` / `src/soccer_match.rs` received comment-only corrections.

* All four test files now import the production modules
  (`osfr_tools::soccer` / `osfr_tools::soccer_match`); the `#[path]` module
  duplication is gone. `lib.rs` now exports both modules (root-owned change,
  observed at cleanup time), which supersedes the earlier "still unregistered"
  limit above; gateway wiring stays lead-owned.
* Raw captured bytes are gone from the test files (hex-literal scan is empty).
  Fixtures are built from synthetic field values through the recovered layouts,
  or manually serialized for the native-wire guards, so they stay meaningful if
  the encoder itself regresses.
* The ignored gate `captured_corpus_round_trips_byte_exact` now covers **every**
  preserved `*-op76-*` frame (2917): each decodes, re-encodes byte-exact, and
  the per-sub counts are pinned (1:1191, 2:1, 3:8, 4:83, 5:17, 6:8, 7:1, 8:1,
  9:8, 10:2, 11:5, 12:5, 14:250, 15:4, 16:1216, 18:1, 19:10, 21:8, 23:78,
  25:20) with `opaque == 0`.
* Comment-only fixes: `Unknown` no longer lists sub 13 as unrecovered, and the
  `player+0xB0` word is `unknown_word` documented as unverified meaning (not a
  sequence or replay clock).
* Comment-only verified against the independent review's frozen copies: with
  the reviewer's mechanical `//!` -> `//` doc-marker conversion normalized, the
  only deltas are those two doc lines (no code lines differ) -
  `comment-only-diff.log` in the cleanup bundle.

| Cleanup check | Result |
| --- | --- |
| four targets (`soccer`, `soccer_match`, `soccer_native_wire`, `soccer_review`) | 56 passed, 0 failed, 1 ignored |
| corpus gate (`OSFR_SOCCER_CAPTURE_DIR=lab/reports/original-minigame-packets`) | 1 passed, 2917/2917 byte-exact, 0 opaque |
| private offsets proof re-run (hex-offset parser fixed) | all checks true, exit 0 |

Cleanup bundle: `lab/reports/soccer-wire-cleanup-20261009T170215Z/`
(`four-targets.log`, `corpus-gate.log`, `offsets-proof.log`,
`native-offsets-proof.json`, `probe-native-offsets.py`, `cleanup-receipt.json`,
`final-hashes.txt`).
