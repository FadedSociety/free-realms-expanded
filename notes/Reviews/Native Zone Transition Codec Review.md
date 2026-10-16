# Native Zone Transition Codec Review

Independent read-only review, 2026-10-09 (UTC). Reviewed source
`prototype/osfr-tools/src/zone_transition.rs` sha256
`0ecf2bca7fa0677149a3715540d6d375d327d63ad01a43b0f042c6276669ca00`, tests
`prototype/osfr-tools/tests/zone_transition.rs` sha256
`88bae93e4aa9580ea07198e147e017498c510c4f13ecae2a2d14699238cba712`, builder
note `Native Zone Transition Codec.md` sha256
`f82a7d821bd0672b23eeb0ceec26fcf66f8d0c8d24eca27ff59434a646d34008`, capture
`lab/reports/original-minigame-packets/racing-13071-flow0-frame723-op31-subNone.bin`
sha256 `bace4cbaef5d50eb3601543847e77e1b4058ce765405f91556dd386f8e32c373`.
No source, test, note, UI, content, save, release or process edit; every probe
lives in `lab/reports/zone-transition-codec-review-20261009T162508Z/`.

## Verdict

Accepted for the bounded wire-codec scope: the field order, string/bool/floats
semantics, canonical round trip and community/C# oracle parity all reproduce.
Three bounded findings are recorded (F1-F3) plus the `Default` semantics note
(F4). No native world entry is claimed or tested - the module is unregistered.

## What was reproduced

| Check | Result |
| --- | --- |
| `cargo test --test zone_transition` | 9 passed, 0 failed, 1 ignored (`builder-green.log`) |
| ignored capture gate with `OSFR_ZONE_OP31_CAPTURE` | 1 passed: decode + byte-exact re-encode (`builder-capture.log`) |
| private probe crate (10 tests, module included by path) | 10 passed (`probe-run.log`) |
| freshly re-run compiled C# oracle vs the current Rust module | 4/4 inner hex and lengths identical; `kt` inner equals the captured frame inner (`oracle-parity-review.json`) |
| `rustfmt --edition 2024 --check` on both owned files | exit 0 (`fmt-check.log`) |
| first-party reader `FUN_008e4ae0` (`pseudocode.c:1029104`), opcode case `0x1f` at `:1108664` | field order/offsets confirmed by reading the decompile |
| community writer `PacketClientBeginZoning.Serialize` + `PacketWriter` | order, `i32` byte-length UTF-8 strings, `4 x f32` Vector4/Quaternion, 1-byte `0/1` bools confirmed |

Field offsets confirmed against the reader: `u16` opcode -> `+0x04`; bounded
`i32`-length name; `i32` type -> `+0x3c`; 4 x `f32` position -> `+0x40` and
rotation -> `+0x50`; bounded sky; `u8` tutorial -> `+0x70` (`!= 0`); `u8`
unknown -> `+0x71`; `i32` id -> `+0x74`; `f32` update radius -> `+0x78`; `i32`
geometry -> `+0x7c`; `u8` override -> `+0x80` (`!= 0`); `u8` wait -> `+0x81`
(`!= 0`). The capture maps to scene `sg_kt_track`, geometry `101`, unknown `1`,
tutorial/`wait` `false`, override `true`, and re-encodes byte-exact.

## Findings

**F1 - non-finite policy is stricter than the native reader (fail-closed).**
The native reader raises its error flag only for `NAN` at `+0x78`; it does not
reject `+/-inf`, and it has no NaN check on position/rotation. The codec
enforces finite values on all **nine** float slots - position `4`, rotation
`4`, update radius `1` - through `checked_floats` (encode) and `Cursor::f32`
(decode). Exact tested coverage, not a full value x slot matrix:
the private probe encodes 4 `NaN` positions, 4 `+inf` rotations and 1
`-inf` radius, and decodes 4 `NaN` positions, 4 `+inf` rotations and 1 `NaN`
radius; the builder tests add `NaN` position decode, `+inf` rotation decode,
`+inf` radius encode and `NaN` radius decode. Decode of `+inf` position/radius,
decode of `NaN` rotation, and encode of `NaN` radius / `+inf` position are not
individually exercised; they flow through the same two finite checks. This is
a deliberate fail-closed choice, not a captured contradiction; the builder
note's "(native also raises its error flag on NaN)" is accurate but incomplete
about infinity. (Corrected 2026-10-09T16:29Z by `correction-receipt.json`;
this note's previous hash `22ca5782fb64e1ee8f17826d1b3cab9f259c2f6452e8264081675cd0f9d341b6`.)

**F2 - outer family is hard-coded to 5 while the transport accepts 5|6
(bounded policy gap).** `gateway.rs::tunnel` accepts `matches!(outer, 5 | 6)`
(`gateway.rs:181-182`) and family 6 is forwarded raw as
`Event::Application(inner)` without opcode dispatch (`gateway.rs:348-354`), so
the codec's family-5 requirement matches the current routing; a genuine
family-6 op31 frame would be refused rather than decoded. No family-6 op31
capture is preserved, so this is a policy bound, not a demonstrated defect. If
family 6 is ever routed into this codec, the family check must be revisited.

**F3 - byte-exactness holds only for canonical booleans (documented).**
Bytes `!= 0` decode as `true` and re-encode as canonical `1`, so a
noncanonical frame (`0x02`/`0xff`/`0x80`) normalizes and does not round-trip
byte-exact. The private probe asserts both halves (canonical frame exact;
noncanonical frame differs at exactly those bytes). The repo note and tests
state this; it only matters if a future path must echo noncanonical frames
byte-for-byte.

**F4 - `Default` is a wire-neutral zero, not the launch packet (documented).**
`ZoneTransition::default()` is type `0`, geometry `0`, flag `0`, empty
strings; the compiled C# oracle's explicit-zero vector matches it
(`defaults` 62-byte inner). The community class's implicit field default is
`Type = 2`, and the captured frame is type `2`/flag `1`/geometry `101`; callers
must supply those. Nothing invents them.

Boundary: opcode 43 (`PacketSendZoneDetails`) is a different family with its
own gateway layout (`gateway.rs:223-240`) and is refused by this codec's opcode
check; native world entry, gateway wiring and scene loading are not tested.
The module stays unregistered: no `zone_transition` reference exists in
`lib.rs`, `gameplay.rs` or `gateway.rs`.

## Artifacts

Private bundle `lab/reports/zone-transition-codec-review-20261009T162508Z/`:
`probe-rust/` (Cargo.toml + tests/zone31_probe.rs), `probe-run.log`,
`probe-vectors.json`, `builder-green.log`, `builder-capture.log`,
`oracle-rerun.json`, `oracle-parity-review.json`, `fmt-check.log`,
`review-receipt.json`, `final-hashes.txt`. Hashes are recorded in
`final-hashes.txt`.
