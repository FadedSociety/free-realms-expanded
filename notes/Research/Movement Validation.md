# Movement Validation (W53b)

W53b extends the W53 transport-free validator with complete available real-trace extraction and server-issued discontinuity suppression. Implementation: prototype/osfr-tools/src/movement.rs. Replay: prototype/osfr-tools/tests/movement.rs. Shared gateway and library files remain for lead integration.

## Recovered client behavior

### Opcode 125 body

The native reader at 008e5940 (original executable file offset 1,030,083) consumes this 36-byte application body:

| Offset | Type | Meaning |
| ---: | --- | --- |
| 0 | u16 little-endian | Opcode 125 |
| 2 | u64 | Actor GUID |
| 10 | 3 × f32 | Position XYZ |
| 22 | 3 × f32 | Direction XYZ |
| 34 | u8 | State |
| 35 | u8 | Unknown |

The reader supplies homogeneous W values internally; they are not on the wire. The client sender at 0092e860 updates local clock bookkeeping by adding 0x32 (50 ms), but opcode 125 serializes no timestamp. SOE reliable sequence numbers are transport metadata, not fields in this body. Replay timestamps come from each capture record.

The native sender builder at 00934a20 constructs the state byte from actor and service flags rather than a named walk/run/jump/fall/swim/mount enumeration. Its Ghidra pseudocode shows bit 4 sourced from the entity flag at +0x650; the receive handler at 0090e0f4 copies opcode-125 bit 4 back to that flag. GetEntitySpeed at 008b3180 multiplies the entity's dynamic speed at +0x3ec by DAT_0175d738 when that flag is set, and by DAT_0175b420 otherwise. Preserved bytes at those data addresses decode as 0.25f (0000803e) and 1.0f (0000803f). This establishes bit 4 as a quarter-speed selector. It does not identify other state bits as locomotion modes.

The replay observed state values 0, 1, 2, 3, 64, 65, 66, 67, 128, 129, 130 and 131. State bit 4 (0x10) did not occur. The second byte was 0 in 14,353 updates and 1 in 629; its meaning remains unknown. The sender composes other bits from boolean, service and entity fields, but their locomotion meanings are not established by the inspected code or these traces. The validator therefore uses only the verified bit-4 quarter-speed fact to reduce the server allowance; it does not invent state-specific tolerances for walk, run, jump, fall, swim or mount. This cap adjustment is reconstructed server policy (R), and bit 4 was not exercised by the real captures.

Jump uses a separate native message handled at 0090e470 with reader 008e5ba0. The inspected path does not establish universal jump impulse, gravity, fall, swim or mount speeds. Area physics values are per-area data and do not define a universal movement-state byte.

## Validator policy (R)

All server tolerances below are reconstructed policy. The client does not provide an authoritative global player speed or universal movement cap.

- The server supplies a positive base speed. Replay-only test policy uses 12 world units/second.
- Effective speed is base speed times the larger of the server-owned mount and buff factors; they do not stack. The 1.5 cap and non-stacking rule are wiki-only in notes/Build Guides/Gameplay/Mounts.md:23; that guide records no client ride-speed constant at line 218. Bit 4 scales this cap by 0.25; untrusted client state can only lower allowance.
- Horizontal soft allowance is effective speed × elapsed seconds × 40. Overages through ×80 are clamped to the soft boundary; greater movement is rejected and retains the previous accepted pose. The multipliers 40 and 80 are reconstructed policy. The full replay includes a 19.105-unit horizontal displacement over a 50 ms receive interval in run 20261010T005751Z; at the 12-unit/s test profile the soft boundary is 24 units over that minimum interval.
- Vertical soft allowance is max(100, effective speed) × elapsed × 3 + 20 units. Overages through ×12 of the same speed-time term plus 20 are clamped; larger movement is rejected. This wider reconstructed envelope preserves vertical room without claiming recovered jump or fall physics.
- Elapsed allowance has a 50 ms minimum and no upper cap. Position components outside ±1,000,000, non-finite input, invalid server policy, and an unauthorized zone change are rejected.
- Direction is normalized in horizontal X/Z; an all-zero horizontal direction preserves the previous direction, matching the native setter at 0095dae0.

### Server-issued discontinuities (R)

A zone-change or developer-teleport token is server-created, bound to a target zone and position, and expires after five seconds of accumulated client-report elapsed time. The reconstructed target-match radius is 16 units.

When a token is active, a finite in-bounds report farther than 16 units is Ignore: do not commit it, do not send a correction, and retain the token. The first report within the radius is accepted and consumes the token. After timeout, suppression ends and ordinary movement validation resumes against the server-committed target. Invalid or out-of-bounds data remains rejectable.

The caller must commit its own authoritative transition target when minting the token. It must retain the returned token after Ignore and clear a canceled token. This avoids snapping the client back toward its stale pre-teleport pose.

## Capture decoding and replay

run-local-baseline.py records datagram elapsed time, direction, service, path, byte count, SHA-256 and wire opcode in each manifest frame (line 342); elapsed time is measured from a monotonic run start (line 253). Four older runs without manifest.json retain equivalent records in frames/live-index.jsonl. The replay validates each referenced datagram's length and SHA-256, applies the crate Codec handshake/decompression/CRC decoder, preserves capture order and asserts gateway capture times never move backward.

For unfragmented client-to-server SOE opcode 9, the replay extracts the post-Codec application bytes after the reliable sequence and sequence-deduplicates them directly. This avoids losing later captured position updates when the ordered receiver has a missing capture sequence. The crate ReliableReceiver remains in use for opcode-13 fragment reassembly and server-to-client control messages. The test then applies the gateway tunnel and combined-message layouts from src/gateway.rs:202-215 and 410-443. It traverses all children of combined application messages and parses every matching 36-byte opcode-125 body. Position-bearing reliable sequences had no reversal; three captured reliable data sequences across all traffic appeared behind a newer sequence.

The successful snapshot processed 99 capture runs: 95 manifest runs and four live-index-only runs. It decoded 14,982 client position updates across 59 position-bearing runs. Of those, 14,971 were in combined application messages and none came from an opcode-13 fragmented message. Every run reported zero rejects and zero clamps. Aggregate outcomes were 13 ignored reports, 26 teleport tokens, 26 accepted teleport targets, 13 ignored stale teleport reports and 58 zone tokens. There were zero movement-position sequence reversals. State counts: 0=4,853; 1=4,360; 2=263; 3=116; 64=63; 65=920; 66=1,757; 67=4; 128=47; 129=962; 130=1,635; 131=2.

The dedicated real trace is client-baseline-20261014T011719Z. It contains three position updates and a server teleport token. The first report arrived 15 ms after the relocation packet and was 325.270 units from the target. It was ignored without commit or correction; the following target report resumed normal validation. Its replay had zero rejects and clamps.

Per-run rows below report position-update, ignore, teleport-token and zone-token counts. Reject and clamp counts were zero in every row.

| UTC run id | updates | ignored | teleport tokens | zone tokens |
| 20261008T015138Z | 0 | 0 | 0 | 0 |
| 20261008T015537Z | 0 | 0 | 0 | 0 |
| 20261008T015843Z | 0 | 0 | 0 | 0 |
| 20261008T020205Z | 0 | 0 | 0 | 0 |
| 20261008T020409Z | 0 | 0 | 0 | 0 |
| 20261008T020419Z | 0 | 0 | 0 | 0 |
| 20261008T022638Z | 0 | 0 | 0 | 0 |
| 20261008T023143Z | 0 | 0 | 0 | 0 |
| 20261008T030630Z | 0 | 0 | 0 | 0 |
| 20261008T030849Z | 0 | 0 | 0 | 0 |
| 20261008T031030Z | 0 | 0 | 0 | 0 |
| 20261008T031406Z | 0 | 0 | 0 | 0 |
| 20261008T031719Z | 0 | 0 | 0 | 0 |
| 20261008T032035Z | 0 | 0 | 0 | 0 |
| 20261008T032359Z | 0 | 0 | 0 | 0 |
| 20261008T033402Z | 0 | 0 | 0 | 0 |
| 20261008T033628Z | 0 | 0 | 0 | 0 |
| 20261008T061543Z | 0 | 0 | 0 | 0 |
| 20261008T063016Z | 0 | 0 | 0 | 0 |
| 20261008T063958Z | 0 | 0 | 0 | 0 |
| 20261008T065035Z | 0 | 0 | 0 | 0 |
| 20261008T065152Z | 0 | 0 | 0 | 0 |
| 20261008T072258Z | 780 | 0 | 0 | 0 |
| 20261008T073831Z | 615 | 0 | 0 | 2 |
| 20261008T075754Z | 1 | 0 | 0 | 2 |
| 20261008T081219Z | 293 | 0 | 0 | 4 |
| 20261008T083419Z | 20 | 0 | 0 | 0 |
| 20261008T083946Z | 32 | 0 | 0 | 1 |
| 20261008T084544Z | 94 | 0 | 0 | 0 |
| 20261008T090058Z | 238 | 0 | 0 | 1 |
| 20261008T091836Z | 94 | 0 | 0 | 1 |
| 20261008T092345Z | 99 | 0 | 0 | 2 |
| 20261008T093305Z | 114 | 0 | 0 | 0 |
| 20261008T151602Z | 570 | 0 | 0 | 0 |
| 20261008T154226Z | 3 | 0 | 0 | 1 |
| 20261008T154503Z | 809 | 0 | 0 | 0 |
| 20261008T162238Z | 37 | 0 | 0 | 0 |
| 20261008T163518Z | 3 | 0 | 0 | 1 |
| 20261008T163829Z | 2 | 0 | 0 | 1 |
| 20261008T164218Z | 1 | 0 | 0 | 0 |
| 20261008T171251Z | 69 | 0 | 0 | 0 |
| 20261008T172324Z | 166 | 0 | 0 | 0 |
| 20261008T173814Z | 231 | 0 | 0 | 0 |
| 20261008T181038Z | 0 | 0 | 0 | 0 |
| 20261008T181715Z | 331 | 0 | 0 | 1 |
| 20261008T183551Z | 3325 | 0 | 0 | 3 |
| 20261008T193318Z | 2685 | 0 | 0 | 1 |
| 20261008T203139Z | 1784 | 0 | 0 | 0 |
| 20261008T235041Z | 1283 | 0 | 0 | 0 |
| 20261009T005748Z | 135 | 4 | 8 | 0 |
| 20261009T013155Z | 4 | 0 | 2 | 0 |
| 20261009T040714Z | 128 | 0 | 0 | 0 |
| 20261009T054421Z | 25 | 2 | 5 | 0 |
| 20261009T103605Z | 28 | 5 | 8 | 0 |
| 20261009T111125Z | 5 | 0 | 1 | 0 |
| 20261009T114509Z | 179 | 0 | 0 | 6 |
| 20261009T121834Z | 88 | 0 | 0 | 2 |
| 20261009T125556Z | 0 | 0 | 0 | 0 |
| 20261009T134930Z | 0 | 0 | 0 | 0 |
| 20261009T143211Z | 0 | 0 | 0 | 0 |
| 20261009T151307Z | 58 | 0 | 0 | 0 |
| 20261009T194318Z | 0 | 0 | 0 | 0 |
| 20261009T194943Z | 0 | 0 | 0 | 0 |
| 20261009T200750Z | 0 | 0 | 0 | 0 |
| 20261009T202011Z | 0 | 0 | 0 | 0 |
| 20261009T202622Z | 0 | 0 | 0 | 0 |
| 20261009T214420Z | 0 | 0 | 0 | 1 |
| 20261009T220348Z | 0 | 0 | 0 | 0 |
| 20261009T220503Z | 0 | 0 | 0 | 1 |
| 20261009T221554Z | 0 | 0 | 0 | 1 |
| 20261009T222147Z | 0 | 0 | 0 | 1 |
| 20261009T222536Z | 1 | 0 | 0 | 1 |
| 20261009T223254Z | 1 | 0 | 0 | 1 |
| 20261009T223836Z | 1 | 0 | 0 | 2 |
| 20261009T224334Z | 4 | 0 | 0 | 1 |
| 20261009T225420Z | 0 | 0 | 0 | 1 |
| 20261009T230018Z | 1 | 0 | 0 | 1 |
| 20261009T232218Z | 2 | 0 | 0 | 2 |
| 20261009T234659Z | 4 | 0 | 0 | 1 |
| 20261010T000146Z | 2 | 0 | 0 | 1 |
| 20261010T005751Z | 121 | 0 | 0 | 1 |
| 20261010T020033Z | 4 | 0 | 0 | 1 |
| 20261010T021651Z | 26 | 0 | 0 | 1 |
| 20261010T022150Z | 5 | 0 | 0 | 1 |
| 20261010T023328Z | 1 | 0 | 0 | 1 |
| 20261010T030742Z | 384 | 0 | 0 | 2 |
| 20261010T032648Z | 20 | 0 | 0 | 1 |
| 20261010T034125Z | 26 | 0 | 0 | 1 |
| 20261010T035338Z | 12 | 0 | 0 | 1 |
| 20261010T040838Z | 12 | 0 | 0 | 1 |
| 20261010T054758Z | 1 | 0 | 0 | 1 |
| 20261010T061735Z | 0 | 0 | 0 | 0 |
| 20261010T062127Z | 18 | 0 | 0 | 1 |
| 20261010T064034Z | 1 | 0 | 0 | 1 |
| 20261013T215401Z | 2 | 1 | 1 | 0 |
| 20261014T011719Z | 3 | 1 | 1 | 0 |
| 20261014T023000Z | 0 | 0 | 0 | 0 |
| 20261014T023458Z | 0 | 0 | 0 | 0 |
| 20261014T023623Z | 1 | 0 | 0 | 0 |
Four manifestless capture runs are included from live-index.jsonl: 20261009T040714Z, 20261009T224334Z, 20261009T225420Z and 20261009T230018Z. One run directory, client-baseline-20261013T213348Z, has neither manifest.json nor frames/live-index.jsonl and no gateway frame capture; it is excluded. The final replay snapshot included 95 manifests, more than the 92 named in the task prompt.

## Integration notes and remaining gaps

No gateway or library file was changed. In gateway integration, mint tokens only from server-owned relocation/zone events, commit the server target at that point, persist the token returned by the validator, and treat Ignore as neither a commit nor a correction. Preserve ordinary reject/clamp behavior after timeout.

The corpus does not label actions as walking, running, jumping, falling, swimming or mounted, so zero rejects/clamps means zero across the captured reports, not a separate acceptance proof for each named activity. State bit 4 is absent from the traces, and fragmented opcode-125 updates were not observed. Mount/buff factors and the 12-unit/s test base remain server policy, not client facts. No native window or production gateway integration was part of this scoped task.

## Commands and verification

Native inspection used the checked map at lab/native-map/run/map.sqlite:

- python -I scripts/native_map/nmq.py show 008e5940
- python -I scripts/native_map/nmq.py dis 0090de90 176
- python -I scripts/native_map/nmq.py dis 0090df37 200
- python -I scripts/native_map/nmq.py dis 0090dfeb 230
- python -I scripts/native_map/nmq.py dis 0090e08b 150
- python -I scripts/native_map/nmq.py dis 008b3180 128
- python -I scripts/native_map/nmq.py dis 0092e860 192
- python -I scripts/native_map/nmq.py dis 0092e960 170
- python -I scripts/native_map/nmq.py show 00934a20
- python -I scripts/native_map/nmq.py dis 00934a20 180
- python -I scripts/native_map/nmq.py bytes 0175d738 4
- python -I scripts/native_map/nmq.py bytes 0175b420 4

From prototype/osfr-tools, scoped rustfmt and test command:

    rustfmt --edition 2024 src/movement.rs tests/movement.rs
    $env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W53b'
    ..\..\scripts\cargo.ps1 -CargoArgs @('test','--test','movement','--','--nocapture')

Result: 10 passed, 0 failed. The run log is lab/build/W53b/movement-final.log. Two unrelated existing dead-code warnings appeared for chess.rs::human_side and housing.rs::Lot::instance_index.
