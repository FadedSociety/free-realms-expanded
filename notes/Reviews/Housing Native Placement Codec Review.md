# Housing Native Placement Codec Review

Independent read-only review of `notes/Research/Housing Native Placement Codec.md` and
`lab/reports/housing-native-codec-20261009T2100Z/`. Writes are this note and
`lab/reports/housing-native-codec-independent-review-20261009T172349Z/` only; no source, Cargo,
content, save, UI, process or worker artifact changed. Skills: RTK (normal commands), Ponytail
(own probe, no new dependencies), Caveman (terse), reverse-engineering (native anchors).

## Verdict

**Candidate verified; one doc-only correction required before/with lead registration.** Every
testable claim reproduces under my own tooling:

| Check | Result |
| --- | --- |
| Candidate + test hashes | `FE7AA64D…` / `6EE3EBCE…` match the pinned files |
| 44 capture vectors re-derived from `p2.pcap` `65AC0801…` | 8× sub2 (42 B, c2s), 9× sub40 (229 B), 8× sub41 (10 B), 19× sub44 (22 B); sha256 identical to the report manifest |
| My Rust probe against the untouched candidate | 16 checks, 0 failures |
| Candidate's own tests re-executed here | 11 passed, 0 failed |
| Single-byte mutation invariant (all 44 vectors, every offset) | 2895 probes: 2411 accepted byte-exact, 484 refused, 0 silent normalizations |
| Truncation (every cut) + trailing byte | refused 44/44 |
| Sizes/counts | 4096/4097/5000/-1 counts and string lengths refused; declared-but-short refused |
| Non-finite floats | NaN/+Inf refused at position/rotation/tilt/scale/unknown14/unknown15, place fields, house fields |
| Strict booleans | raw byte 2 refused at instance bool, 6 definition flags, 4 flags2, house-info bools |
| Invalid UTF-8 | refused (definition strings, 53 message) |
| 127/53 | exact one-string packet; trailing/negative/4097/5000/UTF-8/short all refused; empty message ok |
| `map_key != id` | key=11/id=12 and key=21/id=22 round-trip as separate fields |
| Positional `unknown_guids` | `[0xAA,0xBB,0xCC]` + words `[1,2,3]` round-trip (313-byte rich packet) |
| Oracle cross (decode path) | C# 152-byte tail and my 162/127 fixtures decode and re-emit byte-identical |
| Native full-reader comparison | candidate sub40 fields match my earlier independent parser 9×40/9×40; anchors FUN_00ac2180 chain (4+133+28+50+4+12 = 231) |
| Production hold | no `prototype/osfr-tools/src/housing_native.rs` or `tests/housing_native.rs` exists |
| Worker receipt | 21/21 `final-hashes.txt` entries match current files; tail-report untouched (`CE9500D7…`, typo retained) per its correction receipt |

## Independent probe detail

`probe_independent.rs` compiles the candidate by absolute `#[path]` (no copy, no edit) and runs:

- round-trip of all 44 re-derived vectors (subtype dispatch, exact bytes);
- trailing-byte refusal and every-prefix truncation refusal;
- every single-byte XOR-1 mutation: `Ok` must re-encode to the mutated bytes, `Err` is acceptable;
- sub40 layout/order boundary probes at offsets 2, 18, 135, 143, 147, 163, 167, 179, 209, 217
  (named field must change and re-encode exactly) and 213 (u64-list count 0→1 must refuse);
- malformed/size policies: coupling count, action count, `unknown_guids` count, category length,
  UTF-8, raw bool bytes, NaN/±Inf, on captured vectors;
- 127/53 policy matrix, dispatch (empty, one byte, unknown subtype 99), subtype mismatch, and a
  127-family-prefixed buffer (must refuse — the API unit is subtype-level);
- oracle cross by decoding `prefix(135 B empty instance) + tail + 16 zero trailer bytes` for the
  C# 152-byte tail and my 162/127 fixtures, then comparing the re-encoded tail slice.

The sub40 field dump is compared field-by-field against my earlier independent parser
(`crosscheck.json`, 9 frames × 40 checks, 0 mismatches), whose boundaries were pinned by the
machine-checked native reader chain in [[Reviews/Housing Fixture Tail Compatibility Review]].
The report's `independent-nonempty-all.bin` (162) and `independent-actions-only.bin` (127) are
hash-identical to my compiled fixtures; the 152-byte file parses under my own parser as the
community fixture (item 999, def 5, `"furniture"`, `"kick()"`, actions (1,`"sit"`),(2,`""`),
npc `[0x11,0x22]`).

## Findings

1. **Required doc correction (root-flagged, doc-only).** `candidate/housing_native.rs` line 1
   ("whole inner opcode-127 housing placement flow") and line 655 ("One decoded whole-inner-127
   housing placement packet") contradict lines 3–4 and the actual API: every entry point starts at
   the subtype `u16` and excludes the 127 family opcode. My probe confirms a 127-prefixed buffer is
   refused. Reword both to "subtype-level body; base opcode 127 excluded".
2. **Minor receipt naming (doc-only).** `green-compile.log` is an intermediate failed compile
   (E0560 `character_guids` before the rename to `unknown_guids`); the passing run is `green.log`
   (11/11). Label it as superseded or rename.
3. **Inference note (not a defect).** 127/2 names the second float4 `rotation`; the capture proves
   the layout (`u32 + f32×4 + f32×4 + f32`) but all eight captured second float4 are `(0,0,0,0)`
   and the anchor (object `+0x50..+0x5c`, `+0x74`) does not name it. `first_dword` identity stays
   unestablished, as the note states. The strict bool policy (native treats any nonzero as true)
   and 53 trailing refusal are documented fail-closed choices, not capture parity claims.

## Limits

No native acceptance/playability claim; the capture transport inherits the pinned piece-022 model
(`crc_seed_recovered=false`); non-empty variable paths are pinned by the unchanged community
writer and compiled fixtures, not by captured live traffic; the 127/2 first dword and 127/53
trailing field remain unresolved.

## Artifacts

`lab/reports/housing-native-codec-independent-review-20261009T172349Z/`: `reextract.py`,
`reextract.log`, `my-vectors/` + `my-vectors-manifest.json` (44 re-derived vectors),
`probe_independent.rs`, `probe-compile.log`, `probe-independent.log`, `candidate-tests-compile.log`,
`candidate-tests.log`, `crosscheck.py`, `crosscheck.log`, `crosscheck.json`,
`worker-probe-on-my-vectors.log`, `worker-oracle-three-rerun.log`, `rustc-version.log`,
`proof-manifest.json`, `final-hashes.txt`.

## Root-delta checkpoint (read-only, 2026-10-09)

Scope: the lead's integration delta — `prototype/osfr-tools/src/housing_native.rs`
`9091118F…`, `prototype/osfr-tools/tests/housing_native.rs` `AB0CDD54…`,
`prototype/osfr-tools/src/lib.rs` `0ADD0835…`, and
`lab/reports/housing-codec-lead-integration-20261009T173110Z/`. Candidate `FE7AA64D…` untouched.
Receipt: `lab/reports/housing-codec-root-delta-review-20261009T1735Z/`.

- **Source diff vs candidate is exactly**: line-1 doc rewrite, `rotation` qualification comment
  (my finding 1/3), enum doc line correction, three format-only struct-literal reflows, and the new
  `HousingPacket::encode_inner`/`decode_inner` (127 family prefix, exactly once). No field layout,
  parse, encode or policy path changed. `lib.rs` delta is one line: `pub mod housing_native;`.
- **Tests delta**: candidate tests reformatted, plus 3 ordinary framing tests (exact prefix on
  remove, refusal matrix for wrong family/double prefix/trailing/every cut, same boundary for all
  five subtypes) and 1 `#[ignore]` private gate (`OSFR_HOUSING_CAPTURE_DIR`) verifying the 44-vector
  manifest hashes/counts (2:8/40:9/41:8/44:19) and body+inner round-trips. 15 `#[test]` total.
- **RED/GREEN receipts verified**: `framing-red.log` fails with E0599 `encode_inner`/`decode_inner`
  missing (tests-first RED, red-tests has the framing tests and the gate while red-source has the
  doc edits only); `framing-green.log` = 14 passed, 0 failed, 1 ignored; `private-green.log` =
  1 passed, 0 failed; `receipt.json` hashes match the current three files.
- **My probes on the production copy**: `probe-prod` reruns the full 16-check probe unchanged
  (0 failures) and `probe-inner` runs all 44 capture vectors through the wrapper (51 checks,
  0 failures): `encode_inner` = `127,0` + body exactly once, `decode_inner` round-trips, body API
  rejects inner bytes, inner API rejects body/wrong family/double prefix/trailing/every
  truncation.
- **Verdict**: root delta accepted; codec behavior identical to candidate `FE7AA64D…`; no findings.
