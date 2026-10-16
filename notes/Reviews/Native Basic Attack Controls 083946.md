# Native basic attack controls — 083946

Independent read-only review, 2026-10-08 UTC. Normal native key1 now drives the reconstructed melee basic through a four-enemy cave victory. This is a deliberately reset controls fixture, **not additional campaign progress**. The actual campaign cave completion remains run081219.

## Provenance and comparison

Both runs use `lab/state/native-combat-controls-fixture.json`, derived from campaign run081219. `native-combat-controls-fixture.provenance.json` labels the reset purpose explicitly. Identity, inventory, opening quests, profiles and the already-rewarded encounter411 flag are preserved; encounter health/enemies/phase/position were reset solely to exercise controls.

The preserved campaign parent `lab/state/opening-supported-after-native-cave.json` and original081219 ending snapshot both have SHA256 `60198b4566d6fcc42bf7ae5d2167b19d6a62a38e09ab679c17cb930d90126a0a`. Root independently verified the active campaign had that same hash through the end of083946. Later run084544 intentionally migrates the optional smelting state and normalizes f32 rotation; this review does not claim campaign bytes remain unchanged after that migration.

| Native run | Total frames | CRCs verified | Native36/10 requests | Processed player hits | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| 083419 | 1,732 | 1,711 | 2 | 0 | Type4 alone leaves bar1 requests discarded; all enemies and player unchanged |
| 083946 | 2,351 | 2,330 | 24 | 11 | All three150HP Gloamlings and500HP boss defeated; player660HP |

Both native41/114 metadata packets are423 bytes, with **MiniGameType4 at byte67** and the distinct **ZoneContext8 at byte28**. The whole153,719-byte self payload in each run matches the unmodified supplied C# serializer after applying the saved state to its source object. Captured framing, CRCs and byte hashes pass the independent SOE decoder. The first ending checkpoint SHA256 equals the second starting checkpoint SHA256 (`d78cd2ad477a88d15b16598c801968296a9a4c54d405052b0e3d8e0d999f7797`).

## Native controls, combat and return

All26 captured36/10 requests across both runs are24 bytes and decode to **ActionBarData.Id1, slot0, target0, GUID1**. GUID1 is the player: the native basic is self-originated, then the Rust handler selects one living enemy in reach. The original full multi-target sweep/effect system is not restored. The corrected handler accepts bar1 independently of equipped profile43; native36/5 still carries the equipped profile43.

In083946, all11 player hits follow native36/10 requests, deal950 damage in total and have reconstructed server distances **2.085449–2.730042**. Eleven enemy attacks deal340 damage and occur within2.51 units. Player immediate attack animation1099 appears11 times. Enemy State1 horizontal heading motion, run/stand/attack packets and final zero HP for all four actors are present. No attack menu selection103 occurs: the only26/10 selection is guide Exit101.

Native39/17 starts the activity,39/18 reports won=true at85.422s and39/19 exits at189.047s. The winning persisted state is660HP/all four enemies dead. Exit restores zone0/1000HP at the exact saved f32 return `[-1897.178833,-40.064117,413.730835]`; the native log reports World Ready at those coordinates. Wallet70, eleven inventory entries, quest910002[1,0], identity/profiles and encounter rewarded[411] stay unchanged. No second wallet/item grant or new journal update is emitted. This proves reward farming is suppressed for the already-rewarded fixture.

## Footage review

Viewed the45 sampled frames/contact sheet and detailed frames034 and041 of `normal-basic-combat.mkv` (44.966s,1536×1152,30fps; SHA256 `c38ac403b1003e77e48610d17450346add952cc2e90c49f01adb8159c9e687c9`). Frames3–27 show full-health idle preparation; frames28–39 show player hammer poses, pursuing/attacking boss and damage. Frame041 visibly shows **Goal Complete!**,660/1000HP and no remaining enemies. Sample times are video-relative, not packet wall clocks.

Limits:24 requests are not24 hits; some buffer in the same-time burst or arrive after victory. Generic/blank toolbar art, whole starting-area progression and original full ability behavior remain separate acceptance gaps. No native input, campaign edits or shared gameplay changes were made during this review.

## Reproduce

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-native-basic-controls.py'
```

Proofs: `lab/reports/native-basic-controls-paired-proof.json`; each run's `analysis/native-basic-controls-proof.json`, full self-oracle files, decoder outputs and `video-review-basic/manifest.json`. Releases:083419 `4abaebc46e42993a23603992f97bdebf67f0bd241c3352183f3e358c0fd8c4c3`;083946 `c2380ee2434ccbfcc15176f14a72cb35062aa06dbea1a36e7103491112480950`.
