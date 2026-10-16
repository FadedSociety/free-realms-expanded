# Kart native progress codec (inner 33 subtypes 14, 1, 12)

Status: payload codecs implemented and locally verified, October 9, 2026 (UTC). Evidence label:
**ORIGINAL payload layout and native reader bounds over a RECONSTRUCTED strict-float policy and one
read-only reporting helper**. No authority, gameplay, money, content or native integration claim:
decoding a progress frame proves only that the server sent those numbers.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/kart_progress_native.rs` | subtype 14 / 1 / 12 payload codecs |
| `prototype/osfr-tools/tests/kart_progress_native.rs` | 8 committed vectors + 1 ignored private gate |
| `lab/reports/kart-progress-native-codec-20261009T1622Z/` | RED/GREEN, gate, focused-suite and rustfmt logs, evidence JSON |

No shared `lib.rs`, `kart_racing.rs`, gateway, gameplay, `Cargo.toml`, content, save, UI or release
path was touched. The module is a `#[path]` include in the test target; the test also includes the
existing `kart_racing` module read-only and drives every capture frame through
`kart_racing::decode_racing_frame`, so the tunnel framing is reused rather than restated.

## What is ORIGINAL

Framing is the already-verified vehicle-race tunnel (`u16 outer 5|6`, `u8 flag`, `u32 inner_length`,
`u16 opcode 33`, `u16 subtype`); `inner_length = 4 + payload_len` and the archived frame is
`11 + payload_len` bytes. Subtype 14 (`FUN_00b9fc80`, dispatched by `FUN_00ba74d0`) is 73 bytes:

| Offset | Type | Recovered binding |
| --- | --- | --- |
| 0x00 | u64 | racer guid — selects the racer through `FUN_00b9e0c0` |
| 0x08 | i32 | racer slot/index (0..7 in the capture) |
| 0x0c | f32 | read, NaN-flagged, no recovered consumer |
| 0x10 | f32 | NaN-flagged; stored to racer+0x1614 while the game state is 9; feeds the results sort |
| 0x14 | i32 | laps completed → racer+0x15ec |
| 0x18 | u8 | local-racer flag → processor+0x1a4 |
| 0x19 | i32 | lap limit → processor+0x1a0 |
| 0x1d | f32 | racer+0x1618, no recovered consumer |
| 0x21 | f32 | best lap → racer+0x161c / results `BestLapTime` |
| 0x25 | f32 | total time → racer+0x1620 / results `TotalTime` |
| 0x29, 0x31, 0x39, 0x41 | u64 ×4 | read, no recovered consumer |

Every read is bounds-checked and the reader wrapper reports success only when the buffer was consumed
**exactly**; truncation raises `reader+0x10`, a NaN in `0x0c` or `0x10` raises `reader+0x11`, and the
return value depends on truncation and exact consumption, not on the NaN flag. So NaN is *flagged*,
not refused, and a payload of any length other than 73 is refused natively.

Subtype 1 (`VehicleRacePacketUpdateVehicleGameState`) is one `i32` stored at `processor+0x194`;
subtype 12 (`VehicleRacePacketDestroyProxiedVehicleRace`) is one `u64` guid that removes a racer —
it is **not** a finish.

Captured distributions, re-verified by the ignored gate over all 9,358 subtype-14 frames and 7
subtype-1 frames in the private archive: laps `{0: 8869, 1: 489}`, lap limit `3` in every frame,
local flag `0` in every frame, slots `0..7`, 8 distinct guids, states `3,4,5,6,7,8,9`, no subtype-12
frame, and **0 frames at or above the lap limit** — the capture contains no finish.

## What is RECONSTRUCTED here

| Item | Why it is reconstructed |
| --- | --- |
| `NonFinitePolicy::RefuseNaN` | a local strict rule with a deliberately narrow scope: it refuses a NaN in exactly the two fields the native parser flags (`0x0c`, `0x10`). It does **not** reject infinities, does not range-check any value, and does not inspect `0x1d`, `0x21` or `0x25` — a NaN in those three fields is preserved under both policies. `Preserve` is the native-faithful mode and is what the private gate uses |
| NaN policy scope | only `0x0c` and `0x10` are checked, because those are the two the native parser flags. A NaN in `0x1d`, `0x21` or `0x25` round-trips under both policies |
| `results_row_finished(laps, limit)` | the **read-only reporting mirror of the data-source row** `+0x20`, whose recovered expression is `lap limit < laps completed + 1` evaluated on 32-bit signed ints — so the addition wraps and the predicate uses `wrapping_add(1)` |
| `results_screen_trigger(laps, limit)` | the **read-only reporting mirror of the results-screen trigger** (`FUN_00ba6530`): `lap limit <= laps completed`. It is a different expression from the row predicate and the two diverge only when `laps + 1` wraps at `i32::MAX` |

Neither helper authorizes anything: no production path may use either one to accept a result, grant a
reward or change a phase. Before this correction a single helper named `client_row_finished` used the
screen-trigger comparison and claimed to mirror the row; the independent review caught that at
`laps = i32::MAX` the two recovered expressions disagree, and the helper is now split and named for
what each one actually mirrors.

Nothing else is invented: floats are stored as raw `u32` bit patterns (`unknown_*_bits`,
`best_lap_bits`, `total_time_bits`) so NaN, negative zero, denormals and FLT_MAX round-trip byte for
byte; the four trailing `u64`s stay unmapped; no time units, no "finished" wire flag, no lap authority
and no reward logic exist in the module. FLT_MAX (`0x7f7fffff`) is observed valid in the capture at
`0x10` (6 frames) and `0x21` (5 frames), which is why no finite bound is imposed.

## Verification

Focused only — no whole-crate run, because another builder is writing in the same workspace.

| Check | Command | Result | Log |
| --- | --- | --- | --- |
| Rust RED | `test --test kart_progress_native` | `couldn't find file tests\..\src\kart_progress_native.rs` | `red-missing-module.log` |
| Rust RED (correction) | same | `signed_maximum_boundary_separates_the_two_recovered_predicates` failed while the row helper still used the screen-trigger comparison | `red-signed-maximum.log` |
| Rust GREEN | same | 9 passed, 0 failed, 1 ignored | `green-correction.log` (first implementation `green-first.log`) |
| Private gate RED | `OSFR_KART_PROGRESS_CAPTURE_DIR=<archive>` + `--ignored --exact private_capture_roundtrips_all_progress_frames` | first gate revision matched every `-sub14.bin` frame in the shared archive (other minigames use that suffix) and failed the `9358` count assertion | observed, then fixed by requiring the `-op33-sub14.bin` family |
| Private gate GREEN | same | 1 passed: all 9,358 subtype-14 frames decode, re-encode and re-frame byte-exactly, plus the 7 subtype-1 frames and every asserted distribution | `private-gate.log` |
| Focused sweep | `--test kart_progress_native --test kart_lobby_native --test kart_racing` | 10 / 11 / 13 tests: 9 + 10 + 13 passed, 2 ignored gates | `focused-suite.log` |
| Format | `rustfmt --check --edition 2024` on the two owned files | clean | `rustfmt-check.log` |

The gate also re-derives `finished_rows == 0` from the decoded frames themselves and asserts that the
row predicate and the screen trigger agree on every captured frame, so "the capture has no finish" is
a measured property of this codec's output, not a copied number; the two predicates only separate at
the synthetic signed maximum.

## Remaining gates

1. Lead integration: register `pub mod kart_progress_native;`, drop the `#[path]` includes, and route
   decoded subtype-14 frames from the gateway/racer path through `decode_progress`/`encode_progress`.
2. Decide the server's own progress/limit policy. This codec neither sets a lap limit nor authorizes a
   lap; nothing here should feed a finish, reward or phase transition without separate authority work.
3. Native acceptance: the real client must render laps/times/results from frames produced by this
   codec. Only that closes the integration gate.
4. Open evidence gaps, unchanged: the meaning of `0x0c`, `0x10` (beyond the sort use), `0x1d` and the
   four trailing `u64`s; what the client does with its NaN flag; and the exact finish frame order,
   which no capture contains.

Cross-links: [[Research/Kart Race Finish Protocol]], [[Research/Kart Native Lobby Codec]],
[[Research/Kart Racing Restoration]], [[../Design/MMO Worker Contract]].
