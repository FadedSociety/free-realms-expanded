# Kart Progress and Gateway Catalogue Review

Independent read-only review of two new pieces: the subtype 14/1/12 progress
codec (`src/kart_progress_native.rs` `ed14887d…`, tests `8fe85fd2…`, note
`Kart Native Progress Codec.md`) and the root-authored gateway catalogue
integration (`src/gameplay.rs` `9d58cdc4…`, tests `kart_lobby_gameplay.rs`
`1682549a…`). No source/UI/process/content/save edit; writes are this note and
`lab\reports\kart-progress-and-catalogue-review-20261009T162607Z/`.

## Verdict

**Both verified; one bounded documentation-level finding.**
`client_row_finished` equals the recovered *trigger* expression `limit <= laps`
and the DS row expression `limit < laps + 1` for every representable
`laps + 1`; at `laps = i32::MAX` the DS expression wraps (`MAX < MIN` → false)
while the helper returns true. The module documents this helper as
reporting-only and gives it no authority; no reward, phase or finish acceptance
uses it.

## 1. Progress codec — independent probe (11/11 PASS, exit 0)

- Private 9,358-frame replay: every `racing-*-op33-sub14.bin` payload decodes
  and re-encodes **byte-exactly** under `NonFinitePolicy::Preserve`; 0 mismatches.
- Distributions re-derived: laps `{0: 8869, 1: 489}`, limit `{3}`, flag `{0}`,
  slots 0..7, 8 guids, **0 frames at or above the limit**.
- 7 subtype-1 frames (`3,4,5,6,7,8,9`) and 0 subtype-12 frames, both byte-exact.
- Malformed lengths 0/1/72/74/146 and short/long subtype 1/12 payloads are
  refused with `WrongLength`.
- NaN policy: `Preserve` keeps NaN at `0x0c`/`0x10` (native-faithful, since the
  native reader only raises a flag); `RefuseNaN` returns the typed `NonFinite`
  for exactly those two fields; NaN at `0x1d`/`0x21`/`0x25` round-trips under
  both policies.
- Raw-bit/signed extremes (`u64::MAX` guid, `i32::MIN/MAX`, `-0.0`, denormal,
  `FLT_MAX`, `inf`, quiet NaN, flag 255) round-trip byte-exact — unknown fields
  stay unmapped and unauthoritative; the module contains no lap authority,
  reward or phase logic.

## 2. Gateway catalogue integration (current tree)

- **Load**: optional `kart_lobby.json`; missing file is inert; symlink or
  non-regular file refused; 256 KiB cap enforced before parse; JSON parsed into
  `LobbyNativeBody` (`deny_unknown_fields`); reply bytes are
  `kart_lobby_native::encode_inner(body)` cached in an `Arc` at load time
  (once-cached, never re-read per request).
- **Reply**: developer-tools only; the request must be exactly four bytes
  `[102, 0, 1, 0]`; other subs fall through empty; a truncated or
  trailing-byte request errors when a catalogue is configured; with dev tools
  off no reply is produced.
- **No double tunnel**: the gameplay module returns the inner packet only, and
  the gateway's `wrapped()` (`gateway.rs` 421-426) prepends
  `[5, 0, 1] + u32 inner_length`.
- **Atomicity**: the reply is returned through an `Outcome::default()` early
  exit — no changed flag, zoning or relocation — and the private test asserts
  `game.state()` unchanged and the `kart_lobby.json` file byte-identical.
- **RED→GREEN**: the root `red.log` (`50fe3e32…`, 16:23:49Z) captured 0/2 —
  0 replies and a malformed case accepted. The current tree
  (`gameplay.rs` 16:24:22Z) passes both private gates **2/2**, reproduced by me
  with `OSFR_KART_LOBBY_CONTENT=<private campaign>` (`gameplay-gate-rerun.log`).
- Registration: `lib.rs` now has `pub mod kart_lobby_native;` (line 16);
  `kart_progress_native` is still reached through the test's `#[path]` include
  until the lead registers it.

## Limits

No whole-crate suite was run (the Mining writer is active); the integration
review is source + private-gate level, and no native client acceptance is
claimed. The helper divergence above is bounded to `laps = i32::MAX` and has no
production caller.
