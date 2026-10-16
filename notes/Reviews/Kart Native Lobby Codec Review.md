# Kart Native Lobby Codec Review

Independent read-only **code** review of the new full 102/2 lobby codec
(`src/kart_lobby_native.rs` `d944f33a…`, `tests/kart_lobby_native.rs`
`afd0d045…`, note `Kart Native Lobby Codec.md` `d93d9838…`). No builder code,
test, lib, gateway, content, save, UI, release or process path was modified;
writes are this note and `lab\reports\kart-lobby-native-codec-review-20261009T160824Z/`.

## Verdict

**Verified, no defects found.** An independent private probe (32 checks, 0
failed) re-encodes the real 2393-byte native capture byte-exactly, refuses a
malformed boundary corpus with the typed errors, exercises both trailing
policies, and preserves semantics for raw `u32` keys and arbitrary display
bytes. The builder's own target and private gate reproduce, and the recorded
full-suite log parses to the claimed totals.

## 1. Framing and tables vs the independent review

The probe decodes `racing-23867-flow0-frame44523-op102-sub2.bin` with
`TrailingBytes::Strict`: outer 5, flag 1, inner length 2386, opcode 102, sub 2,
body 2378, counts **6 / 30 / 43 / 37 / 17**, trailing 0; games row 1
`(1,1,1,37,3030)`, row 6 `(6,6,6,60,417078)`, display row 1 locale 3032 `"2"`
and row 7 `"64"` — all equal to my independent lobby review.

`encode_tunnel(5, 1, &body)` reproduces **all 2393 raw bytes** and `encode_body`
reproduces the 2378-byte body; decoding the re-encoded frame returns an equal
body with zero trailing bytes. This is a faithful native-vector re-encode, not
a mirror of the builder's tests.

## 2. Refusal / alloc / overflow / raw keys / display bytes / trailing policy

The probe mutates the real frame and asserts the typed refusals:

| Mutation | Refusal |
| --- | --- |
| outer 4 | `UnsupportedOuterOpcode` |
| inner 101 / sub 3 | `UnsupportedInnerOpcode` / `UnsupportedInnerSub` |
| inner length ±1 | `InnerLengthMismatch` |
| body length -1 / beyond buffer | `NegativeBodyLength` / `BodyLengthMismatch` |
| truncated frame / consistent-but-truncated tunnel | `InnerLengthMismatch` / `BodyLengthMismatch` |
| inner header 5 bytes | `TooShort` |
| games count -1 / i32::MAX | `NegativeCount` / `CountExceedsRemaining` |
| display length -1 / i32::MAX | `NegativeTextLength` / `TextExceedsRemaining` |

The count pre-check (`count > available / row_bytes`) bounds every table
**before** `Vec::with_capacity`, so the hostile `i32::MAX` count refuses in
under a millisecond with no allocation, and mid-row `TooShort` is unreachable
for the fixed-size tables; display text is length-checked against the remaining
bytes before `take`. Raw `u32` keys (`u32::MAX`, `0x8000_0000`, `0xFFFF_FFFE`)
and display bytes `[0, 255, 65, 0, 10]` roundtrip byte-exactly. Trailing policy:
`Strict` refuses frame-level and body-level trailing with position+count;
`Tolerant` accepts and reports 1/1 without changing the decoded body.

Encode-time `FieldTooLarge`/`BodyTooLarge` guards are code-inspected only (they
require > 2 GiB inputs to reach in a probe).

## 3. Builder evidence reproduced

- `cargo test --test kart_lobby_native`: 10 passed, 0 failed, 1 ignored
  (`builder-codec-target.log`), matching the builder's `green-first.log`.
- Private gate with `OSFR_KART_LOBBY_CAPTURE=<capture> --ignored --exact`:
  1 passed (`builder-private-gate-rerun.log`).
- Neighbour reruns on the current tree: lib 24, kart_racing 13, kart_review 12,
  all exit 0 (`neighbour-*.log`).
- Builder `full-suite-attempt.log` parses to **1468 passed / 0 failed / 59
  ignored** over 155 result lines, with **0** `Application Control`/`4551`
  mentions — consistent with the reported 155 groups including the five
  previously blocked targets.

## Limits

The full 155-target suite was not re-executed end-to-end by this review (the
builder log was parsed and the codec gate plus neighbour targets reproduced);
no gateway wiring or native client acceptance is claimed here.

Bundle: [`review.json`](lab\reports\kart-lobby-native-codec-review-20261009T160824Z/review.json) sha256
`f9b56043887904cd67ef8d41faf3c97109193187b2750d3a58c387304f4134df`.
