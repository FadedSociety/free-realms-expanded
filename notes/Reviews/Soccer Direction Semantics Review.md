# Soccer Direction Semantics Review

Independent read-only review of the appended direction-semantics section in
`notes/Research/Soccer Kick Direction Binding.md`
(`48d1c969850faca262063d3f1b5d0e56cde71cbdb59e6a52d0205aa2f90c6340`) and its
report `lab/reports/soccer-kick-direction-followon2-20261009T173220Z/`. No
source, adapter, gameplay, UI, save or process was changed; owned artifacts are
this note and `lab/reports/soccer-direction-semantics-review-20261009T173639Z/`.

Method: the 25 MB `FreeRealms.exe` (`26cc1c52…`) was parsed with a hand-written
PE header/section walker (no `pefile`), the six floats and four vtable slots
were read from the raw bytes, and the pseudocode anchors were checked against
the reviewer's own list rather than the author's checker. The captured corpus
was then re-parsed for wire corroboration.

## Verified

| Claim | Independent result |
| --- | --- |
| `DAT_0175B420` 1.0, `DAT_017E795C` 0.1, `DAT_017E99A8` 3.0, `DAT_017ED9D0` 255.0, `DAT_017F1AE4` 1/255, `DAT_018144D0` -0.2 | all six f32s read exactly (`verify.json`) |
| vtable `0x0183A0FC` `+0x38/+0x48/+0x70/+0x7C` | `0x00CD9DF0`, `0x00CD8330`, `0x00CD9AF0`, `0x01072610` |
| vecA `+0xF0..+0xFC` from `+0x38`/`+0x30`, consumed by `+0x7C` (guard 0.1) writing `+0x50..+0x5C` | anchors matched |
| kick/back: dot(`+0x50..+0x5C`, `+0xF0`) >= -0.2 -> `0x16`, else `0x17` | anchors matched |
| vecB `+0x30..+0x3C`: squared length vs 0.1 -> `0x23`/`0x24`, fallback copies `+0x50` into `+0x30` | anchors matched (first branch `<=`, fallback `<`) |
| vecC `+0x20..+0x2C`: proximity `+0x48` uses `+0x20`/`+0x28`, `distance^2 <= 3.0`; `+0x38` target delta | anchors matched |
| aux byte = `+0xA4` charge ratio x255 on send, x1/255 on receive; `+0x70` hook clamps against 1.0 | anchors matched in the sender, receive case `0xe` and `FUN_00cd9af0` |
| word = `+0xB0` state-change counter incremented by `FUN_01072020` | anchor `param_1[0x2c] = param_1[0x2c] + 1;` matched |
| Wire offsets relative to the payload | reader `FUN_00B794B0` consumes u64,u8,u8,u32,3 x f32[4] into `+0x10/+0x18/+0x19/+0x1C/+0x20/+0x30/+0x40`; packed payload = 62 bytes at `0x00/0x08/0x09/0x0A/0x0E/0x1E/0x2E`; the 73-byte `.bin` carries an 11-byte tunnel wrapper before the payload |

Capture corroboration over the 250 sub-14/62-byte records
(`capture-stats.json`): every state byte is in `1..0x3F` (the receive guard);
vecC has `w == 1` and a small y in 250/250 (position-like); vecB is unit length
in 154 and exactly zero in 96 (the zero cases select the `0x23` StealTackle
branch); vecA is unit length in 239 and zero in 11.

## Limits confirmed

`FUN_00CD97D0` exists (vecB consumer, `(obj, float*, char)`) and is not resolved;
the writer maintaining local `+0x20`/`+0x28` between state changes is not
established; the target selector `+0x1C == 3` is a raw field with no recovered
name. The section's unresolved list is therefore accurate, not padded.

## Findings

1. No discrepancy found: every constant, slot and dataflow anchor checked out,
   and the capture statistics independently separate a position-like vecC, a
   direction-like vecB/vecA pair and the tackle threshold behaviour.
2. Precision nuance for consumers: the first vecB branch uses `<= 0.1` while the
   second (fallback) branch uses `< 0.1`; the note's "squared norm against 0.1"
   is correct but a decoder must not collapse the two operators.
3. The receive mapping (`FUN_00B80AC0` case `0xe`) is a **client** path; the
   authority note stands — none of these vectors authorizes ball motion, and
   the server must still authenticate the sender. No adapter/gameplay change was
   made or recommended here.

All raw bytes stay in the lab. Executed checks: `verify-direction-semantics.py`
(0 errors) and the corpus re-parse, both under
`lab/reports/soccer-direction-semantics-review-20261009T173639Z/`.
