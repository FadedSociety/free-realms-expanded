# Kart native driving failures — October 9, 2026

## Current v14 acceptance: refill, Alt and Ctrl verified

Clean native022150 on accepted releasef9f4af0b verifies four refill request/reply pairs, one diagnostic pickup grant/removal, isolated Alt and Ctrl activations,448 native boost poses and one checkpoint. Original purple pickup actors render near the first bend. Finalized43,427 hashes/43,406 gateway CRCs, zero unchecked; normal menu exit0 and owned services stopped. Coins309 and inventory/XP unchanged. Full native lap HUD/results/reward/Close/reload remain pending; see [[Kart Native Boost Acceptance]]. The021651 extra-Go-click activation was ambiguous and is not keybind proof. Sections below preserve the earlier failure and its investigation.

## Fresh native probe: pickup, Ctrl and first checkpoint verified

Run `client-baseline-20261010T020033Z` (UTC), release SHA `fb68f768db9b0cb0a29a2edb226c3fdb97b581db03111e77b9ef70d6215052b3`: 37 native pickup actors spawned. Seven immediate native collision claims yielded exactly one S33/18 type5 grant followed by one S33/16 removal of handle1. Ctrl generated C33/19 and S33/20 type5; the original client visibly accelerated with flame exhaust. The direct stream records 224 boost-control poses (bit4), and the first original checkpoint produced an additional S33/14 progress reply. This verifies the native pickup/use path and one gate, not lap progression or a whole race.

This was an explicitly diagnostic package: handle1 moved half a metre from the starting grid. Canonical retail positions are preserved separately. Fixed boost drops, other item effects, respawns, native Alt, full-lap HUD, finish, payout, Close and post-win restart still need acceptance. Inventory and XP remained byte-equivalent as parsed values; coins stayed309.

New defect: C33/22 arrived at276.188s against Go265.585s (10.603s elapsed), but our reconstructed20-second refill epoch refused it; no later refill request occurred before the bounded300-second harness stopped. The scoped DeepSeek item worker is checking the native refill timing before a narrow correction. Do not count refill as repaired based on the successful pickup-sourced boost.

Final audit: 8,027 frame hashes and8,007 gateway CRCs verified, zero unchecked controls;5,670 direct native poses verified. Harness reports client/login/server stopped, independently absent by their recorded PIDs. Termination was the harness time limit, not ordinary menu exit. Receipt: `../lab/reports/kart-native-pickup-probe-20261009/native-receipt.json`.

### Refill correction under lead review

Worker evidence traces `FUN_00feaa30`: vehicle accumulator at+0x114 charges with config+0x740 and additional terms, drains with config+0x744 while boost is active, and clamps to the native maximum. `FUN_00c5f7c0` seeds it full; `FUN_00ba48a0` sendsC22 when full and fewer than3 items are queued, then resets it. There is no recovered constant20-second rule. Historical captures have10.547seconds from transmitted countdown0, roughly33.2seconds, and longer gaps; the server's scheduled Go gives10.603seconds for the same new request.

The narrow candidate accepts owned in-window refill requests while space remains in the three-slot queue. The legacy epoch field stays for saved-shape compatibility and no longer enforces pacing. Distinct identical application requests can fill available slots; underlying reliable retransmissions are deduplicated by transport. This local restoration does not yet simulate the client's config-driven energy accumulator on the server. Before acceptance the lead requested explicit lower-edge race-clock refusal for refill and use; full queue/malformed/foreign/phase/timeout/restore checks remain required.

Lead private replay of the finalized nativeC17/C22/C19 route first failed becauseS23 was missing, then passes that assertion with the candidate. Two older full-race private oracles still assumed fixed-epoch replay refusal and were updated to fill3 slots, refuse the fourth unchanged, and consume exactly3 responses before empty-queue refusal. Preserve the failed output as obsolete-policy evidence; it is not a native success.

Lead accepted the lower-edge correction: one pure race-window helper refuses clock values beforeGo for refill, use and pickup, and refuses expired races. Pre-Go pickup still consumes one observation budget entry as before; queue/actor/movement state stays unchanged. Focused service55passed/0failed/1ignored; pickup6plus1module check pass. Lead's8 private end-to-end checks now pass, including the actual captured10.603-second refill and both existing whole-race payout/reload checks. Full gate, release and fresh native refill/Alt verification follow in v14.

the owner tested the original client in `client-baseline-20261010T005751Z` (UTC run name). Handling and jumping worked. The lap display stayed at 1/3, full boost icons did not activate with Ctrl or Alt, and mystery boxes did not appear to be collected. These are open acceptance failures, not a completed race.

## Capture evidence

Frozen gateway: `49317256be59de74199dd2f1bb2b3eb61eee8bb84da1b971e925ea6bd97e2fb8`. Separate disposable v12 character; no campaign save reset.

The reliable application audit identifies 97 client checkpoint claims (`33/7`), 17 item-use requests (`33/19`), three boost refill requests and their three empty replies (`33/22 → 33/23`), and only the initial progress frame (`33/14`). No pickup request (`33/17`) was observed. These counts cover the captured sessions, not necessarily one race.

The independent direct-stream audit verifies hashes and seeded CRCs for 35,186 position updates (`33/6`). Controls include 0, 64 and 128. Position ranges: x 386.269–795.000, y 18.224–85.265, z 271.552–830.476. This verifies actual movement rather than the earlier stationary tap tests. It does not prove a valid three-lap finish.

Current service code supports refill acknowledgements but refuses item-use subtype 19; pickup subtype 17 only records an observation. A boost icon therefore proves acquisition feedback, not activation. The missing use lifecycle is assigned to the scoped DeepSeek item worker. A separate read-only DeepSeek worker is reconciling actual checkpoint claims against the reconstructed gate geometry and per-race countdown anchors.

Evidence: `../lab/reports/kart-human-driving-20261009/`, including the direct audit, finalized capture receipt and reliable reassembly. Capture finalization and complete hash validation are recorded in the receipt rather than inferred from this note.

## Reference video

Installed `bradautomates/claude-video` skill `watch` at `%USERPROFILE%/.codex/skills/watch`. Source: https://www.youtube.com/watch?v=T1d_tPxlNrk — *Free Realms kart racing! (HQ)*, uploaded May 21, 2009, duration 446 seconds. Metadata lists no captions. All 20 initial sampled frames have been viewed locally. the owner selected local WhisperX; its isolated installation and model warm-up are in progress. The initial visual pass used no transcription and no cloud upload.

### Observed visual reference (2009)

This is **Tangetrack**, with eight racers and **five laps**, rather than the current private solo three-lap track. Treat it as historical behavior evidence; do not silently replace later-client settings.

- 00:00: original Get Ready traffic-light overlay; several other karts.
- 00:22 and 00:45: lap 1/5, place 6/8 then 3/8. Grey empty-looking item circle, separate orange energy meter, blue exhaust and a minimap with racer dots.
- 01:07: orange rocket-like icon in the item circle.
- 01:29: lap 2/5 and purple question-mark pickup cubes across the road. Their appearance is verified; this sparse sample alone does not prove the pickup event.
- 01:51 and 02:36: orange/pink flame exhaust visible; 02:13 shows blue exhaust and an empty-looking item circle. These are effect transitions, not visible keypress or packet evidence.
- 02:58, 04:28 and 05:35: HUD progresses through laps 3/5, 4/5 and 5/5 respectively.
- 07:04: Tangetrack “2ND PLACE!” results panel with ranked names, Best Lap and Total Time columns, and “Waiting For All Players To Finish.” No reward payout is established by this frame.

Frames and local media: `../lab/reports/kart-video-reference-20261009/watch-r25_2xhs/`. Twenty frames sample the video roughly every 22 seconds; they do not establish continuous behavior or exact input timing. Targeted cue frames are being extracted for pickup and final-results transitions.

Additional five cue frames were viewed from `watch-xlwd8pmw`: 01:28 empty item circle before the boxes; 01:30 and 01:32 show a purple item in the circle after passing their location. This supports pickup-to-toolbar feedback without establishing the precise collision instant or item name. At 07:20 the results overlay has cleared; 07:25 shows the Kart Race Lobby with “No Games Found,” “Create New Game” and “Join.” No reward screen was captured by these cues.

WhisperX 3.8.6 isolated environment is installed at `%USERPROFILE%/.cache/watch/whisperx-venv`; CPU PyTorch and small-model/VAD caches passed the skill's warm-up. Windows setup used the already installed Python 3.12.14 base interpreter after default uv discovery failed. Automatic transcription of the local video is running; do not treat a music/noise transcript as confirmed speech.

### Recovered boost contract

The scoped worker recovered native client `UseItemRequest` C33/19: exactly eight payload bytes, the owned u32 GUID pair. All 17 captured use requests contain GUID 1. The client sender latches use while waiting for a response. `UseItemResponse` S33/20 has exactly one u32 item type. Dispatcher type **5** writes the native boost-energy field, enables boost and removes a queued item. This is distinct from refill S33/23, which adds type 5 to the queue. Source pointers: `FUN_00ba48a0`, `FUN_00ba1540` → `FUN_00ba0a70`, and `FUN_00ba74d0` case 0x14. Implementation and regression evidence are pending lead acceptance.

Lead gateway regression initially failed because C19 returned no response; after the worker implementation it passes, including a malformed-bundle rollback, exact S20 type5, unchanged coins/XP and empty-queue refusal. All four prior private gameplay checks pass. The new actual-capture checkpoint replay still fails with zero accepted claims, independently confirming the geometry blocker in Rust.

### Authored curtain recovery and review corrections

The corrected CDTA reader parses 144 vertices at offset24 and 72 triangle index triples at1756, rather than mistaking BVH words for vertices. Manifest CRC/size identify the preserved 4860-byte source. Thirty-six physical curtains are transformed by the chunk pose; the rotation-field interpretation remains inferred but all 97 human claims are within1.366–2.567m of actual triangles (median2.008m), and the original claims also match.

Mapping actual triangles supersedes the first worker's nearest-old-waypoint labels: **the first curtain is crossed on every lap**. Its earlier proposed post-lap cursor skip is rejected. The track has alternative paths: curtains26/28/27 correspond to2/3/4 at three logical route positions. One extra curtain19 is observed once and curtain20 remains unobserved; neither should advance the currently reconstructed31-step route. Recovering authored physical volumes and branch aliases is in progress; wide bands fitted to this one capture are not accepted.

A second correction: the Rust service starts at the countdown-zero scheduled `due_at`, not the later GameState9 frame. The independent private Rust replay executes the real schedule and both 5/6 tunnel prefixes; it preserves that clock behavior.

WhisperX completed analysis of the local reference and detected **no speech**. No spoken instruction, key binding or mechanic is inferred from it. The25 reviewed frames provide the visual evidence described above.

## Acceptance still required

### Lead checkpoint after source repair

Lead's `private_actual_native_checkpoint_capture_completes_three_laps` now passes against the gates-only v13 package: all97 claims execute with the real schedule,93 advance checkpoints,3 laps complete,phase Finished. Log: `../lab/reports/kart-playable-v13-20261009/lead-native-route.log`. This is replay evidence, not fresh native HUD/finish acceptance. No cursor skip was added. Authored surfaces use a reconstructed3m proximity tolerance and bounded travel credit: max3s elapsed allowance plus up to3s saved slack; sub50ms buffered claims can spend saved slack but mint no new time.

Pickup building block accepted after lead correction: staged token fields private, grant type resolved from authoritative definition, consumed boxes excluded from re-announcement. Six integration and one forged-token checks pass. Exact packets retain original captured words; type5 boost drops/radius/all-at-start/no-respawn are labelled reconstruction. Native types1..4 effects and original random odds are not implemented. Lead route test serves four recovered assets with CRC/SHA/envelope proofs. Model497's own dependencies are available; separate box model lacks `sg_random_box_pickup_bbe.dme` and `.dds` in both recovered packs and public mirror. Service lifecycle integration remains pending.

Independent math review found the first curtain validator accepted folded/concave leaves and used cross magnitude instead of true triangle area. The pickup integration worker is tightening those checks with explicit pathological fixtures. Actual34 leaves are planar/vertical with minimum triangle area2352m², so the stricter contract preserves shipped content. Movement budget has a360m total ceiling at60m/s only after banking180m; pristine idle time provides at most180m. This is bounded network tolerance, not strict instantaneous-speed proof.

A separate `kart-native-pickup-probe-20261009` diagnostic moves handle1 to0.5m from the starting grid. Its receipt records original/canonical/probe hashes and positions. It is intended to verify native actor/collision/grant/use cheaply; it cannot establish original track placement, a whole lap or a paid finish. Canonical v13 keeps all37 captured positions.

- Native item use consumes one owned queue item and visibly activates boost; empty/replayed/foreign requests cannot award effects or coins.
- Actual ordered checkpoint crossings advance the HUD through 2/3 and 3/3, then finish exactly once.
- Pickup collision emits the native request, grants the appropriate item and allows ordinary toolbar use. Effects require recovered native contracts.
- Finish displays the registered reconstructed 50 coins/25 Kart Driver XP, persists once, closes normally and survives restart.
- Normal NPC/menu entry remains separate from the developer-only `realm race` command.
