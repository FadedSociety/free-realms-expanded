# Native cave completion 081219

2026-10-08 UTC. Independent read-only review of ended `client-baseline-20261008T081219Z`.

**Accepted: a complete reconstructed melee cave battle, reward, guide exit and native overworld return.** The player defeats three Gloamlings and the Gloam Overlord, survives at280 HP, earns50 coins plus one Wheeler on the Cob, then returns to zone0 with1000 HP. Normal action-bar abilities and the whole starting-area campaign are still unaccepted.

## Capture and restart identity

Release SHA-256: `e6c07a27cafc12294d9055c6a9b33fa0da49f331b90b530469c8a37f1f907054`. All5,747 manifest frames are accounted for:5,730 gateway and17 login. All5,728 negotiated gateway CRCs pass; reliable reassembly completes. The entire153,689-byte self payload, including the existing supported identity, Brawler43, inventory10 and typed quest910002 at[0,0], equals the independently compiled original supplied serializer. Raw sources, keys and game material remain private in the lab.

The run starts by resuming the previous lost cave. Two subsequent attempts lose; the third retry wins. Those failures are retained, not omitted from the proof.

## Preparation and observed reach

Across the three retries, no NPC movement or processed attack occurs between client10 and the explicit Start/first valid melee strike. The idle windows are **79.406,152.000 and127.656 seconds**. Preparation keeps the player at1000 HP and the enemies idle. The first attempt explicitly chooses guide menu104/Start; the latter two start with a valid enemy strike. The first AI damage follows these triggers by1.625,0.219 and0.172 seconds respectively. No conditional client39/5 acknowledgement is present.

All20 processed player hits in the whole run correlate with native26/10 selection103 requests within150 ms and at measured server-observed distances **1.970–2.434 units**. The winning attempt uses11 hits, totaling950 damage, at approximately2.108–2.434 units. This fixes the earlier visibly long-range behavior for the demonstrated melee loop. It proves observed close hits and idle preparation, not every possible rejected-distance boundary; no explicit out-of-reach rejection test is inferred from the capture.

Enemy delivery includes293 opcode125 State1/nonzero-horizontal-heading updates. Animation counts are10 run,10 stand,101 immediate enemy attack01 and20 player com_swing1099. All20 player swing packets correspond to the20 processed player hits. The winning attempt's last enemy-health records are0/0/0/0, and the preserved winning snapshot records phase`won`, player280, all enemies0 and wallet70.

## Visible battle

The preserved `combat-melee-playthrough.mkv` is50.066 seconds, H.264,1536×1152 at30 fps. Offline FFmpeg extraction produces50 review frames and a labeled contact sheet in `analysis/video-review-melee`, with source/tool hashes and commands recorded.

Nominal video seconds0–23 show stationary preparation with full HP and distant idle enemies. At24–26 the player turns and walks up to a Gloamling; its native X combat prompt appears. Seconds27–42 show hammer swing poses, floating100 damage, enemy pursuit/attacks and successive removals. Detailed reviewed frames28 and34 show player attack poses and adjacent enemies; second43 displays the native **Goal Complete!** overlay, no remaining enemies and **280/1000 HP**. The surviving player remains afterward. Local video seconds are not synchronized capture wall-clock timestamps.

This is real native-client visual evidence for the reconstructed melee controls. The blue/generic toolbar remains visible; its appearance does not prove original ability restoration.

## Reward and return

At capture second557.969, server39/18 reports won=true. Native38/19 sets coins70, once;38/2 adds original definition1195/Wheeler on the Cob, instance11, quantity1. Before wallet20 becomes70 and inventory10 becomes11. The winning save has the same values and only one awarded cave411 entry.

At682.375 the client chooses guide26/10 selection101/Exit. At682.469, server39/19 closes the activity; typed49/9 changes quest910002 objective-key1 to1 with progress0.5, then49/10 marks that objective complete. Opcode31 returns to `FabledRealms`, zone0, geometry214, with exact original saved f32 return position **[-1897.178833,-40.064117,413.730835]** and facing. Client13/client10 and native World Ready follow; health is restored to1000.

The ending snapshot has zone0, no active encounter, HP1000, rewarded[411], coins70, item instances11 and quest910002 accepted/[1,0]/not yet complete or rewarded. Original class progress and discoveries remain unchanged. Subsequent overworld walking changes the final saved position to approximately[-1943.505,-37.38244,371.92014]; this does not replace the separately proven initial return coordinate. Pappy's objective and quest turn-in remain outstanding.

## Normal abilities and remaining limits

Four native36/10 requests occur; each is24 bytes and decodes, using the pinned original community schema, to **ActionBarData.Id1, Slot0, Target0, Guid1(self)**. The fields are not all zero. None produces a processed attack. All successful player damage is from26/10 selection103, so normal abilities remain unaccepted. A later isolated Type4 control checkpoint is a separate test and cannot retroactively establish success here.

Battle bindings, damage/balance/reward rules and guide placement are explicitly reconstructed around recovered story/models; this is not recovered original full-game behavior. Boss spawn/despawnGR2, cave detail textures and some overworld detail assets still have HTTP404 gaps. The recording does not validate missing effects or full original content.

## Reproduction

`lab/reports/verify-native-cave-win.py` passes bounded assertions and writes `analysis/native-cave-win-proof.json`: whole-self original-serializer comparison, per-attempt preparation windows, exact action requests, measured hit distances, four defeats, animations, native rewards, exit/quest/zone ordering, snapshot hashes and visible-review provenance.

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/analyze-native-created-character.py' --run client-baseline-20261008T081219Z
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/inspect-combat-motion-video.py' --run client-baseline-20261008T081219Z --video combat-melee-playthrough.mkv --output video-review-melee --frames 60
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-native-cave-win.py'
```

No live input, game launch, shared code edit or saved-progress modification was performed by this review.
