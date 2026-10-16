# Fishing independent review

2026-10-08. **No blocking findings in the fishing wire codec or the actor-binding/validation path. Two documented authority gaps sit behind a catch authority that does not exist yet.** Fishing is still not playable and no native client run was made; this review is behavioral and source-level only.

Scope: MMO003 `prototype/osfr-tools/src/fishing.rs` and `tests/fishing.rs` after the actor-binding and finite-input fixes. The module under review was read-only; the reviewer wrote `tests/fishing_review.rs`, this note and the lab report only.

## Verdict

The module reproduces the recovered native wire contract and refuses bad input before mutating session state. Independent byte-level fixtures (hand-built, not produced by the module's own encoder) confirm the envelope, all five request field orders, all eight decoded server bodies and the result body's unusual field order. The actor binding, foreign-handle refusal and validation-before-mutation rules behave as claimed. What remains is not a codec defect: the tracker cannot distinguish a replayed result from a new one, and it lands from Idle, because the missing catch authority is exactly the thing that would have to supply that correlation. Both are recorded as falsifiable tests and as findings R1/R2 in `lab/reports/fishing-review-20261009T024626Z/report.json`.

## Native verification (re-derived, not inherited)

Checked against `lab/decompiled/client/FreeRealms/pseudocode.c` and the preserved server enum `lab/decompiled/server/LibSOE/SOE/BaseFishingPackets.cs`; quoted line ranges are in `native-evidence.md`.

- Envelope: `FUN_00b66830` reads and `FUN_00b69550` writes two little-endian u16 (opcode 138 then sub-opcode). A hand-built `Special` frame decodes and re-encodes byte-for-byte, length 12.
- Primitives: `FUN_008fb810` writes the eight-byte handle (two u32, id then seq); `FUN_0091a640` writes and `FUN_008e2410` reads four f32.
- Requests: `FUN_00b6a400` (2) handle+flag 13 bytes; `FUN_00b6a380` (5) handle+f32+vector 32; `FUN_00b6a460` (6) handle+vector+flag 29; `FUN_00b6a4d0` (7) handle+flag 13; `FUN_00b6b510` (15) handle 12. All five match `ClientRequest` exactly, including a field-order sensitivity check on the cast-animation distance.
- Server bodies: sub 1 handle+vector (24); sub 5 echo handle+NaN-flagged float+vector (28); sub 7 echo handle+byte (9); sub 8 handle+u32+two vectors (44); sub 10 handle+u32+two bytes (14); sub 12 u32 (4); sub 13 u32+vector (20); sub 14 result. Each `ServerPacket::decode_known` size equals the native reader's consumption.
- Result body: the module consumes handle, at_18, at_1c, at_20, at_24, blob_2c, at_38, at_3c, at_40, at_44, at_48, at_4c, at_50, blob_58, blob_64, at_74, at_7c, at_80, then at_78 read last from its own trailing dword, matching `FUN_00b66be0` store for store. Minimum body 74 bytes; one trailing byte is refused. A distinct value in every field catches any transposition.
- Numbering: all 16 sub-opcode values match `BaseFishingPackets`; 9 and 17 are absent from both sides and decode as `UnknownSub`.
- Duplicate cast: `FUN_00b6cd80` compares the incoming handle pair and requires controller state `!= 6` before entering state 6 (1539878-1539880 and 1539938-1539940), which is consistent with the module's `Duplicate` for a repeated cast while Casting. The decompiler renders that switch expression as an unassigned local, so the exact sub-to-case mapping was not re-derived; the module labels the policy reconstructed.

## Findings

**R1, low, gated on a future catch authority.** A repeated `FishingResult` is re-applied. `Session::server` re-enters `Landing` for the same body because the actor binding survives `Landing` and results carry no sequence or nonce. The falsifying probe asserted `Err(OutOfOrder)` and failed with `Ok(Some(Landing))` on the second application. Nothing duplicates today because no reward path exists; before any reward is granted, the server needs request/result correlation or one-shot landing consumption. Documented test: `replayed_result_is_reapplied_without_a_nonce`.

**R2, low, same gate.** A valid result lands while the session is still Idle after registration, before any cast or reel-in. The probe asserting `Err(OutOfOrder)` failed with `Ok(Some(Landing))`. Without a catch authority this is only a phase label, but the phase cannot be treated as catch evidence. Documented test: `result_lands_while_idle_after_registration`.

**R3, informational.** Native strictness is not uniform and the module is not uniform in the same way. `FUN_00b668f0` flags a NaN distance in the cast-animation echo, which the module decodes silently; `FUN_00b66be0` flags NaN only, while `ResultBody::valid` also refuses infinities. The stricter infinity rule is a documented local choice, but the code comment saying the NaN rule is applied "the same way" overstates it. Covered by `echo_and_result_value_strictness`.

**R4, cosmetic.** `ClientRequest::decode` returns `UnknownSub(n)` for the recovered but non-request sub-opcodes 3, 4, 11, 16 and 18 even though `Sub::from_u16` knows them; the message reads "unknown fishing sub-opcode n". No behavioral consequence, but logged errors will mislead. Covered by `request_decode_rejects_non_request_sub_opcodes`.

**R5, informational.** Sub 11 (spawn school) is opaque in the module, but `FUN_00b69130` shows a recovered top-level prefix (u32 + vector + vector + nested readers + 8 bytes); only the nested readers are unrecovered. The opaque fallback is byte-lossless, so nothing is lost; the research note's wording ("reader located" next to "body not recovered") is what needs alignment.

## Checks

- `scripts/cargo.ps1 test --test fishing` (builder suite, reviewer rerun): 12 passed, 0 failed.
- `scripts/cargo.ps1 test --test fishing_review`: RED first, 14 passed / 3 failed (the two probes above plus one reviewer fixture bug: the echo fixture included the four-byte header in a body-only call, fixed in the harness); then 18 passed, 0 failed, no warnings.
- `scripts/cargo.ps1 test --test fishing --test fishing_review`: 12 + 18 passed, no warnings.
- `rustfmt --edition 2024 --check tests/fishing_review.rs`: clean; the reviewer's file only.
- Integrity: `src/fishing.rs` SHA256 `eee55aed…83cd` and `tests/fishing.rs` SHA256 `5d560b0d…b926` are unchanged from the builder report (`lab/reports/fishing-restoration-20261009T015330Z/report.json`), proving the review did not alter the artifact it judged.

## Residual risk and limits

The tracker's ordering rules, phase labels and validation remain reconstructed, not recovered; this review confirms their implementation, not their historical accuracy. Sub 3, 4, 11, 16 and 18 bodies are unrecovered, so no bite or catch decision exists. Native acceptance (a live fishing attempt over opcode 138, gateway integration, reward granting) is untouched by this review and stays with the lead. The largest practical risk is treating `Session` as catch evidence once a reward path appears; R1/R2 must be closed at `Session::server` before that happens.

Evidence: `lab/reports/fishing-review-20261009T024626Z/` (report.json, native-evidence.md, sha256-manifest.txt, red/green cargo logs).
