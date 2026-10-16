# Native combat motion retest 075754

2026-10-08 UTC. Read-only review of ended run `client-baseline-20261008T075754Z` and its preserved 35.1-second recording.

**Result: enemy approach and attack animation now work visibly. Playable combat remains unaccepted because the retry starts attacking automatically and the player dies without fighting back.** The dungeon was not completed.

## Packet and spawn proof

The patched server release is `498fb4ee0a98c081f710d10e139117576a0344112be4ded974821b1999787ee6`. The capture has 3,320 frames: 3,299 gateway and 21 login. All 3,297 negotiated gateway CRCs pass; reliable reassembly completes without pending fragments.

An independent reader verifies all 12 captured enemy spawns have Speed **3.0**, Walk/Run/Stand animation IDs **2/3/1**, and physics MovementType **2**. Variable name/texture strings were parsed before calculating offsets; fixed offsets were not blindly applied to expanded records.

All **256 enemy opcode125 packets** use State **1**, Unknown **0**, and finite normalized horizontal headings with Y=0. Counts by enemy are 22, 46, 72 and 116. These are the expected corrections from the earlier State0/zero-heading capture.

All **100 opcode35/subtype8 animation packets** match the original community writer and native schema:

| Animation | ID | Flags | Packets |
| --- | --- | --- | --- |
| Run as base animation | 3 | 1 | 8 |
| Stand as base animation | 1 | 1 | 8 |
| Immediate hand-to-hand attack01 | 1001 | 0 | 84 |

There are 84 corresponding processed enemy attacks targeting playerGUID1. No enemy takes damage in this run. See [[Native NPC Motion and Animation]] for the independently compiled serializer fixtures and native facing-vector proof.

## Visible evidence

FFmpeg extracted one review frame per second from the original `combat-motion-retest.mkv`, retaining the source unchanged. A timestamped contact sheet and 35 individual 768×576 frames are in `lab/runs/client-baseline-20261008T075754Z/analysis/video-review`. The manifest records the source SHA-256, FFmpeg binary SHA-256, exact extraction command and video metadata: H.264, 1536×1152, 30 fps, 35.1 seconds.

At nominal video second 13, an advertisement/loading frame appears. At second 14 the boss is mostly off the left edge and the three Gloamlings are spread out. Seconds 17–20 show the boss approaching the stationary player as the Gloamlings converge. Seconds 20–28 show clearly different boss body/arm attack poses, floating 20/40 damage numbers and falling player HP. The detailed reviewed frames at seconds 14 and 24 show HP 980/1000 and 280/1000 respectively, with the boss displaced next to the player. Seconds 29–34 show the lost/idle state.

This establishes rendered boss/ordinary-enemy motion and visible boss attacks for this recorded retry. It does not individually synchronize every pose to its packet, prove player input worked, or establish a balanced/playable battle. Video timestamps are local to the recording and are not synchronized to capture wall clock.

## Boss asset delivery

`/127/gloam_m_boss.adr.z` returns HTTP200. The preserved raw ADR is 12,204 bytes, CRC32 **3361679076**, SHA-256 `ac842259cb0d62648e8c4083df6e1edf7d381d8d6681185ecd8a7ff830c477ba`. Rebuilding its original A1B2C3D4 + big-endian length + zlib envelope independently reproduces the manifest's **3,577 bytes** and wire hash. The pinned community ADR reader parses stand, run and attack01 bindings. The model visibly renders in the video.

Two optional animations remain unavailable: `gloamconstruct_boss_com_spawn.gr2` and `gloamconstruct_boss_com_despawn.gr2`, each requested 12 times with HTTP404. The visible result proves these misses do not prevent the boss's demonstrated approach and attack01; it does not accept those missing spawn/despawn effects.

## Retry timing and ending state

The first load resumes the previous already-lost instance at HP0; it is not a fresh attempt. Two chat retries follow. Times below are seconds from the capture's monotonic start:

| Event | Retry 1 | Retry 2 |
| --- | --- | --- |
| Client10 load notification | 186.516 | 272.219 |
| First AI damage | 187.766 (+1.250) | 273.485 (+1.266) |
| HP0/loss | 200.906 (+14.390) | 286.485 (+14.266) |
| Voluntary player displacement or attack | None | None |

Retry 1 emits one automatic unchanged-spawn player125 with State1. Retry 2 emits none. Neither sends client opcode39/5 or any other client39 packet. This corrects the preliminary statement that both retries lacked every player125; the actual issue is absence of voluntary movement/attack and an unproven readiness signal.

Ending snapshot: zone411 at `[287,28,286]`, phase `lost`, playerHP0, enemies still **150/150/150/500 HP**, no dungeon reward. Opening quests, inventory and class progress are unchanged from the previous run. Root's subsequent preparation/start gate is a separate change requiring its own live acceptance proof.

## Reproduction and review artifacts

`lab/reports/verify-patched-native-combat.py` passes all bounded assertions and writes `analysis/patched-combat-wire-and-video-proof.json` with frame/application citations, parsed spawns, movement/animation/HP histories, retry timelines, ending snapshot hash and asset proof. `inspect-combat-motion-video.py` reproduces the contact sheet and video provenance.

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/analyze-native-created-character.py' --run client-baseline-20261008T075754Z
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/inspect-combat-motion-video.py'
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-patched-native-combat.py'
```

No live input, launch, shared code edit or saved-progress modification was performed by this review.
