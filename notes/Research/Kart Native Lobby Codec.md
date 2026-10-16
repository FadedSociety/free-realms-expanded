# Kart native lobby codec (102/2)

Status: codec implemented and locally verified, October 9, 2026 (UTC). Evidence label:
**ORIGINAL framing and table layout over a RECONSTRUCTED byte-policy boundary**. No native
integration, no gateway wiring, no content and no playability claim.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/kart_lobby_native.rs` | codec: framing, five tables, typed refusals |
| `prototype/osfr-tools/tests/kart_lobby_native.rs` | 10 committed vectors + 1 ignored private gate |
| `lab/reports/kart-lobby-native-codec-20261009T1602Z/` | RED/GREEN logs, gate log, evidence JSON |

No shared `lib.rs`, `kart_racing.rs`, gateway, gameplay, `Cargo.toml`, content, save, UI or release
path was touched. The module is a `#[path]` include in the test target until the lead registers it.

## What is ORIGINAL

The framing and the five-table body come from the accepted evidence, reused verbatim:

```
tunnel   u16 outer (5 or 6) | u8 flag | u32 inner_length
inner    u16 opcode = 102 | u16 sub = 2 | i32 body_length | body[body_length]
relation inner_length = 8 + body_length
```

| # | Table | Row layout | Row bytes | Capture rows | Capture start | Capture bytes |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | games | `u32 key, i32 id, i32 game_type, i32 unknown, i32 name_id` | 20 | 6 | 0 | 124 |
| 2 | options | `u32 key, i32 × 3` | 16 | 30 | 124 | 484 |
| 3 | values | `u32 key, i32 × 3` | 16 | 43 | 608 | 692 |
| 4 | display | `u32 key, i32 unknown, i32 locale_id, i32 length, bytes[length]` | 16 + length | 37 | 1300 | 666 |
| 5 | settings | `u32 key, i32 × 5` | 24 | 17 | 1966 | 412 |

Each table is introduced by an `i32` count, the order is fixed and the capture consumes exactly
2378 body bytes. `FUN_008d67a0` only reads the two inner int16 header words (no sub comparison in its bytes, re-checked 2026-10-12 per native-map S0228); the sub == 2 gate is in its caller `0091e060` (HandleLobbyGameDefinitions), and `FUN_008bfc00` reads the body
length with a `cursor + length <= end` bound, so the native reader bounds the body inside the frame
rather than requiring the buffer to end there. Sources and hashes: [[Research/Kart Lobby Definition
Framing]] and `lab/reports/kart-lobby-independent-worker-review-20261009T155631Z/review.json`.

## What is RECONSTRUCTED here

Everything below is this codec's own bounded policy. None of it is a recovered client or server rule.

| Rule | Value | Why |
| --- | --- | --- |
| trailing bytes | `TrailingBytes::Strict` refuses bytes after the declared body **and** after the five tables; `TrailingBytes::Tolerant` accepts both and reports the count | the capture is exact-consumption; the independent review lists client tolerance as unverified, so the codec exposes both instead of guessing |
| count bounds | a negative count refuses; a count that cannot fit the remaining bytes at the table's fixed row size refuses before any allocation | bounds a hostile or truncated frame without inventing a row limit |
| text bounds | a negative display length refuses; a length larger than the remaining bytes refuses | same |
| framing refusals | wrong outer opcode, wrong inner opcode, wrong sub, negative body length, body length beyond the buffer, or a tunnel `inner_length` that disagrees with the buffer all refuse with typed reasons | mirrors the two native gates and adds explicit tunnel bookkeeping |
| encode guards | a count or display string that cannot be represented in `i32`, or a body that cannot be represented in the framing length, refuses instead of truncating | keeps the framing honest at the type boundary |

The codec does **not** invent: lobby catalogue rows, which game types are legal, what the unknown
words mean, name-id resolution, the default/game-list policy the gateway should send, or any
validation of catalogue content. Field *semantics* stay positional (`unknown`, `fields`) exactly
because the preserved sources do not recover them.

## What the capture proves

The ignored gate reads the private capture
`lab/reports/original-minigame-packets/racing-23867-flow0-frame44523-op102-sub2.bin`
(raw sha256 `4f6d75c49554e39588a8117a94a650b0154f5d40bb0bf6a45a1e0073a2cb6932`, 2393 bytes) and
requires:

* outer 5, flag 1, `inner_length` 2386, opcode 102, sub 2, body length 2378;
* body sha256 `51c8f011c781fe376496c851889380ba36f6bbed911daa60eb964ecc71fd5a0c`;
* counts `[6, 30, 43, 37, 17]` and strict decoding with zero trailing bytes;
* `encode_tunnel(5, 1, body)` reproduces all 2393 raw bytes and `encode_body` reproduces the body,
  so the round trip is byte-exact in both directions;
* the captured games rows and the first eight display rows match the independent review's
  re-derivation, including the test-catalogue row `(1, 1, 1, 37, 3030)` and the `"64"`/`"68"`
  display strings.

No captured bytes are committed: the committed vectors are derived layouts only, and the capture is
read through `OSFR_KART_LOBBY_CAPTURE` at run time.

## Relationship to the existing writer

`kart_racing::LobbyGameDefinition::encode` mirrors the pinned community writer: `i32` count, then per
entry `i32 key` plus four `i32` fields, then four zero `i32` counts — a 140-byte subset for six rows.
It is not the native 102/2 frame. This codec is the full native framing. Wiring the gateway response
through `kart_lobby_native::encode_tunnel` (and deciding the fate of the subset writer) is the lead's
integration step; nothing here calls into or modifies that module.

## Verification

Commands run from the project directory with `scripts/cargo.ps1` (private toolchain under `../lab/tools`).

| Check | Command | Result | Log |
| --- | --- | --- | --- |
| Rust RED (module absent) | `test --test kart_lobby_native` | `couldn't find file tests\..\src\kart_lobby_native.rs` | `red-missing-module.log` |
| Rust RED (behaviour) | same, first module revision | 2 failed: frame trailing bytes reported as body trailing, and the lenient reader accepted a declared length it never bounded | `red-first-revision.log` |
| Rust GREEN | same | 10 passed, 0 failed, 1 ignored | `green-first.log` |
| Private gate | `OSFR_KART_LOBBY_CAPTURE=<capture>` + `--ignored --exact private_capture_roundtrips_the_full_2378_byte_body` | 1 passed; byte-exact round trip of the 2393-byte frame | `private-gate.log` |
| Regression sweep | `--lib` / `--test kart_racing` / `--test kart_review` | 24 / 13 / 12 passed, 0 failed | `lib-suite.log`, `neighbour-suite.log` |
| Format | `rustfmt --check --edition 2024` on the two owned files | clean | `rustfmt-check.log` |
| Whole crate | `test` | completed with no failures in this revision of the workspace | `full-suite-attempt.log` |

The two behavioural RED failures changed the implementation, not the tests: the declared body length
now bounds the body exactly (bytes after it are frame trailing data, refused under `Strict` and
reported under `Tolerant`), and the frame-trailing count is folded into `DecodedLobby::trailing_bytes`.
One test-side error (an offset expectation that assumed six games rows in a one-row vector) was fixed
in the test, not by loosening the codec.

## Remaining gates

1. Lead integration: register `pub mod kart_lobby_native;` in `src/lib.rs`, drop the `#[path]`
   include, and route the 102/2 response through `encode_tunnel`.
2. Decide the served catalogue. This codec preserves whatever catalogue it is given; it does not pick
   retail rows, resolve name ids or validate game types.
3. Native acceptance: with the gateway answering 102/2 through this codec, the original client's
   lobby list must render. Only that closes the integration gate; this note proves bytes, not UI.
4. Open evidence gaps carried over unchanged: the options/values semantics, the display word's
   meaning, the games row's fourth word and the client's trailing-byte tolerance.

Cross-links: [[Research/Kart Lobby Definition Framing]], [[Research/Kart Racing Restoration]],
[[Research/Go Kart Native Integration Readiness]], [[../Design/MMO Worker Contract]].
