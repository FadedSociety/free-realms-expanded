# Native starting quest and cave run073831

2026-10-08 UTC. Independently reconstructed ended native run `client-baseline-20261008T073831Z`. This is **partial quest/zone/state proof and failed visible combat acceptance**. the owner reported stationary enemies, absent attack animations and long-range hits. The run ends in loss, not dungeon completion. It predates the subsequent movement/animation patch.

## Transport and restart state

The run has4807 total frames:4790 gateway frames and17 login frames. All4788 negotiated3-byte gateway CRCs verify. Ordered reliable/multipacket reassembly completes without pending fragments. Server release SHA-256 is `5063442e993cae0b7af2d69223b6b6885f03a5c24d5713c40311d8ad64bcb52d`.

The initial native self packet is153702bytes. All153689 inner payload bytes equal the supplied original `Gateway.dll` serializer when given Capo Indigocauldron's preserved identity,10 inventory instances, active Brawler43, saved position/heading,10coins, classXP25 for Adventurer/Brawler, and the accepted Hero's Journal entry with counts `[1,0]`. The native journal contains exactly this one active opening quest, selected910001, completedCount0, AutoSwitchfalse, preserved trailing fields10/30. This proves actual reconnect delivery of saved starting-quest progress, beyond the earlier fresh-empty-journal proof.

## Quest progression and cave transitions

These native opcode49 packets were captured in order:

| Subtype | Verified captured result |
| --- | --- |
| 9 | Hero's Journal910001, objective key2, count1, overall progress1.0 |
| 10 | Same objective marked complete, progress1.0 |
| 4 | Hero's Journal removed on turn-in |
| 12 | Completed-quest counter becomes1 |
| 3 | The Last Best Hope910002 added and selected, two objectives `[0,0]` |

The second quest's entire327-byte record independently equals the supplied compiled serializer. The incremental9/10 layouts were read independently using the native schema, including keyed objective ordinal2 rather than its definition ID.

After those updates, server31 requests `sg_npx_cavern_01`, zone411, geometry214, spawn `[287,28,286]`; native client13 and10 follow. Client logs confirm world-ready at that cave position. A retry repeats31→13→10 and world-ready. The earlier initial world also sent13/10, giving three complete load handshakes across the run.

## Combat evidence and the failed visual check

There are34 server opcode125 movement packets across four cave enemies, with changing positions. Every packet uses the **old State0 and all-zero heading**. Per-enemy movement counts are4,6,10,14. Their X/Z spans reach4.14/3.72,3.20/2.40,6.36/5.20 and11.92/9.11 units respectively. These sent coordinates establish wire delivery, not visible model movement. The user's report of stationary models remains authoritative acceptance evidence.

All8 damaging player attacks use native interaction menu selection103 (`26/10`), with8 corresponding32/7 attack-processed packets and enemy35/5 HP changes. **No native36/10 ability request occurs. No server35/8 animation packet occurs.** The run therefore cannot accept ability-bar combat or visible attack animation. Player38/1 HP updates reach zero in both attempts.

## Premature combat timing

Times below are seconds from the capture's monotonic start. First player125 packets merely repeat the spawn position with State1; they occur before voluntary displacement.

| Event | First cave | Retry |
| --- | --- | --- |
| Client10 | 106.672 | 151.157 |
| First player125 at unchanged spawn | 107.016 (+0.344) | 151.250 (+0.093) |
| First AI damage | 107.703 (+1.031) | 152.266 (+1.109) |
| First displaced player125 | 112.969 (+6.297) | 158.000 (+6.843) |
| First player attack selection | 115.657 (+8.985) | 158.922 (+7.765) |
| HP0/loss | 121.813 (+15.141) | 167.391 (+16.234) |

There are **zero client opcode39 packets**, including no39/5 ready acknowledgement, in either cave load. Server39 uses an8-bit subtype and sends17/start and18/game-over. This corrects an earlier transcription that mislabeled hexadecimal11/12 as decimal1/2. A gate depending solely on absent39/5 would never start this captured flow. First player125 is also insufficient readiness evidence because it is automatically emitted at the unchanged spawn. An explicit player begin interaction or a measured grace period is a reconstruction choice for root to test; the capture does not prove a native control-ready packet.

Read-only native follow-up establishes that39/5 exists but is conditional. Original Lua `MinigameHowtoHandler` calls `Ui.StartPendingMiniGame()` after loading finishes or the howto closes (7887/7900/7933). Its preserved API binding at `00c17290` calls `008a6b50`, which requires client+0x700==2. `009be7d0` then requires a pending game pointer and not-already-started flag(+0x2c3), marks it started and invokes `009bdc60(5,-1,-1)`→`008a4420` opcode39. Thus the native pending-game flow must exist before39/5 can be expected. `Ui.ReadyForActivity` is a different launch-ready packet, opcode175/subtype18 (`00c1b1c0`→`00c4e1b0`→`00c498a0`), not39/5. Binding-table pointers and tiny forwarding functions were checked with Capstone because Ghidra omitted one forwarding function.

## Persisted ending state

The ending `character-after.json` keeps the same identity fingerprint and10 inventory instances. Hero's Journal is accepted/completed/rewarded with `[1,1]`; The Last Best Hope is accepted, incomplete and unrewarded with `[0,0]`. Coins increased10→20 and BrawlerXP25→50; AdventurerXP remains25. The cave instance411/1 is `lost`, playerHP0; three150HP Gloamlings are dead and the500HP boss remains alive. No dungeon reward is recorded. Character position is `[293.21143,27.993917,290.77444]`, still zone411. This is a preserved failed attempt, not completion of the dungeon or starting area.

## Reproduction and review artifacts

Private capture proof: `lab/runs/client-baseline-20261008T073831Z/analysis/opening-playthrough-wire-proof.json`. This includes application/frame citations, full self oracle hashes, second-quest compiled comparison, all health/movement/attack records, ending snapshot hash and readiness timelines. Decoder proof: `gateway-decode-proof.json`; exact decoded applications and serializer inputs/outputs remain beside it.

Conditional ready-path evidence: `lab/reports/native-combat-readiness-functions.c/.json`, `native-combat-readiness-disassembly.txt`, reproduced by `extract-native-combat-readiness.py`.

```powershell
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/analyze-native-created-character.py' --run client-baseline-20261008T073831Z
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-native-opening-playthrough.py'
```

All bounded proof assertions pass. No game input, launch, shared Rust edits or saved progress changes were performed by this review. See [[Native NPC Motion and Animation]] for the subsequent native direction/speed/animation investigation; actual visible retesting remains a separate acceptance step.
