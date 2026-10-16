# Soccer Native Wire Repair Review

Independent read-only acceptance of the soccer native wire repair
(`src/soccer.rs` `babd674b…`, `src/soccer_match.rs` `4d6006ce…`, new
`tests/soccer_native_wire.rs` `aa74ea8b…`, note `Soccer Native Wire Repair.md`
`c20ad33d…`). No source/UI/content/save/release edit; writes are this note and
`lab\reports\soccer-native-wire-independent-review-20261009T164427Z/`.

## Verdict

**Repair verified.** An independent private probe (14/14 checks, exit 0)
re-encodes the captured traffic byte-exactly, preserves raw words and NaN bits,
enforces the exact subtype widths, and shows the Kick/Pass state is advisory
with no direction inference or authority bypass. One low-severity stale
documentation line is recorded below. The old-family test reruns could not be
reproduced because a concurrent Mining writer left `src/mining.rs` mid-edit
(lib `E0425: power/pressed_value`); this is unrelated to soccer.

## 1. Independent probe results

| Check | Result |
| --- | --- |
| Captured sub 14 replay | **250/250 byte-exact** |
| Captured sub 16 replay | **1216/1216 byte-exact** |
| Captured sub 13 | 0 frames (none observed) |
| Sub 14 exact layout | `u64` guid, `u8` state, `u8` aux, `u32` word, three `f32[4]`; qNaN payload / −0.0 / denormal bits round-trip |
| Sub 16 exact layout | `u64`, two `f32`, `u32` controls; `0/1/2/0x11/0x41/0x100/0xFFFF_FFFF` preserved; NaN direction bits not normalized |
| Sub 13 | 36-byte layout round-trips; refused as a client request ("server-authored") |
| Malformed | short/long/trailing/length-mismatch frames all refused |
| Authority | captured `GoalMade` decodes server-side but `client_packet` refuses it; server-only sub 11 refused from a client; sub 14 non-kick state and sub 3 registration stay `ReadOnly` with explicit intent refusals |
| State map | `16-26 | 31-34` Kick, `27-30` Pass, everything else `None`, over all 256 byte values |
| Advisory | Kick action is identical for aux `0x00` vs `0xFF`; `unknown_word` copied verbatim; `client_intent` error contains "adapter gap" and no "Move"; state 27 maps to Pass |

The probe includes doc-comment-converted copies of both modules (code bytes
identical; hashes in the manifest) and does not depend on the repo lib, so the
concurrent Mining edit could not mask a soccer defect.

## 2. Findings

- **Low, documentation only**: `src/soccer.rs` `Unknown` variant doc still lists
  `13` among sub-opcodes "without a recovered layout" while sub 13 now decodes
  as `ImpulseBall` and round-trips; the code and the repair note are correct,
  the enum comment is stale.

## 3. Builder claims not independently reproduced (bounded)

- `tests/soccer.rs` 11 / `soccer_match` 27 / `soccer_review` 10 / new
  `soccer_native_wire` 8 green: all four `cargo test --test` runs abort at
  compile time on the concurrent `src/mining.rs` edit
  (`E0425: cannot find value power/pressed_value`), so the lib cannot build
  while the Mining writer is active. The builder's own `green.log`
  (`2df04f49…`) and `captured-roundtrip.log` (`14767ff0…`) are preserved in
  their bundle; every assertion in the 8-test target is covered by the private
  probe above.
- The module remains unregistered (no `soccer` entry in `src/lib.rs`), matching
  the builder's stated integration boundary.

## Limits

No native playability, world-entry or client-kick claim; sub 13 labels stay
inference with no captured vector; aux byte, `player+0xB0` word and the three
vectors keep positions only.
