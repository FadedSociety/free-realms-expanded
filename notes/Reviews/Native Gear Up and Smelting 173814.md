# Native Gear Up and Smelting — 173814

Gear Up was accepted at Cleetus through the native interaction. Selecting Miner unlocked/activated profile 14 and added exactly the five configured starter definitions `[21665,21060,21784,21219,6561]` as instances 17–21. All 16 earlier item records and every other job's equipment remained unchanged.

Five normal counted ore interactions advanced goal `91000503` through counts 1, 2, 3, 4, 5 on actual 49/9 packets, followed by one 49/10 completion. The saved stage counts are `[1,1,5,0]`; inventory contains Copper Ore 2739 ×5 and Singing Coal 879 ×5. Wallet remains 120; Smelting has not started or awarded bars. The Miner class goal used definition key `91000502` on both update and completion, preserving the corrected incremental-key behavior.

Private immutable checkpoint proof: `lab/runs/client-baseline-20261008T173814Z/analysis-live/native-gear-up-proof-002331.json`, SHA256 `106eb6153acd8649f2d345a0bde231086ce476790effaa18157bac09a2730748`. Reproduction: `lab/reports/audit-native-gear-up.py`. The proof stores the entire captured checkpoint and checks all unrelated gameplay against startup.

The first screenshots were taken before presentation settled. Independent inspection of `gear-up-objectives-settled.png` now confirms Other Quests / Gear Up with green Talk, Miner, and Copper Ore objectives; Smelt 5 Copper Bars remains blue at 0/5. The right-edge Other Quests tab is at native screen coordinate (1236,100); the page flip needs approximately two seconds. Screenshot hash and acceptance are preserved in `analysis-live/native-gear-up-journal-visual-proof.json`.

Smelting entry is accepted on the actual native wire: client 39/5 `[77000001,-1,-1]` at frame 2614, then server Start 39/17 and StartFlash 26/12 for `game_smelting.gfx`. The actual native OnConnectMsg and ReadyForGameData use the same sentinel header and exact NUL framing. The server replies once with game 77, all four phase rows, and OnDataComplete. Saved instance 77000001 is Playing with data_sent=true; materials and wallet are unchanged, and no parser/session error is present. Snapshot: `analysis-live/native-minigame-wire-snapshot-002745.json`; whole checkpoint: `native-gear-up-proof-002714.json`, SHA256 `d2edb7099cf8f12a702d0ab15e01edf24bc124bde6c0d56cdb4c60fdad2394cc`.

Original first round 101 completed: actual bad/good/perfect/time statistics are `[17,40,28,152]`, each transmitted with one literal terminal `)` and accepted into saved state. ScoreSet 3897 is also stored correctly. The nominal phase duration 90 does not cap the original stopwatch; the actual 152-second round remains within the server's explicit time bound. Snapshot `native-minigame-wire-snapshot-004684.json` and matching Smelting summary contain exact messages; no parser/order/framing/session error is present. Materials remain unconsumed while Playing.

Round 102 first emitted `[0,0,0,122]`, then retried and emitted `[0,1,0,4]`; both exact attempts are preserved in snapshot 006282. The successful tuple and ScoreSet 3997 were saved without rejection.

Independent original timer review establishes that this overwrite is compatible: failed attempts call roundCompleted and emit statistics, retry recreates the round, and SmeltingHUD resets only `nRoundDurSecs`. The aggregate stopwatch survives and onWin transmits it separately as final game-time kind 4. Current Rust stores that field independently and uses it for results elapsed, with no equality requirement against the latest per-round sum. Proof: `lab/reports/native-smelting-retry-time-proof.json`, SHA256 `9f7f506678a98888ed956cc0aebe897a00d5e6127c0aa17c9d6f72c1a372f111`; reproduction: `prove-smelting-retry-time.py`. The actual final cumulative time remains to be checked.

All four rounds completed. Latest round tuples `[bad,good,perfect,time]`:

- 101 Break Ore: `[17,40,28,152]`.
- 102 Fill Furnace: `[0,1,0,4]`, after failed `[0,0,0,122]`.
- 103 Heat Furnace: `[0,0,0,157]`.
- 104 Mold Metal: `[0,0,0,11]`.

Actual final totals are `[time448,bad17,good41,perfect28,displayedQuantity2]`. Original GameWon separately requests score 5497 and quantity 10. The reported cumulative time 448 is accepted independently of the latest per-round sum 324; the failed 122-second attempt explains most of the difference. All stat suffixes and final StopLevel were accepted without a parser/order/session error.

The terminal native wire is OnGameWonMsg → 39/18 `(won=true,stars=0,elapsed=448,score=5497)` → 39/47 one actual `scoreTotalScore` type-4 row for 5497 → 39/19. The automatic native removal ACK 39/7 is present, while saved state remains Won and no actual Close has arrived. Held-results proof: `analysis-live/native-smelting-results-state-008928.json`, SHA256 `18adce4df51e758ea420603ccf03a433c06dd67f24982d8ea72a9433633c1de7`.

The fixed reconstructed recipe consumed exactly 5 Ore and 3 Coal, leaving Ore 0 / Coal 2 / Copper Bars 10. Stage 5 is `[1,1,5,5]`, still unturned/unrewarded; coins remain 120. Exact whole-state allowance, original items, new item metadata/IDs/order, and Miner equipment all pass: `native-gear-up-proof-008907.json`, SHA256 `7d2bb8c0c22bfdd9e3caa37d320b7d47f8cc0bb31fcc865372f360f1b1813004`.

Finalized acceptance: actual native Close is frame 8935, exact four bytes `1a002a00`, about 100 seconds after the win. Cleetus interaction 9086 turns Gear Up in; wallet updates once to 130 and Miner XP once to 25. Cleetus interaction 9147 accepts Your First Weapon, adding its exact supplies and Blacksmith starter equipment once. This second interaction is a real next-quest acceptance, not an ignored repeat. Terminal score-screen pixels were not independently captured at the Smelting boundary; the terminal wire, held Won state, exact Close, subsequent world interaction and recipe/reward state are proven.

Blacksmith selection then advances stage 6 to `[1,0]`. Forging instance 139000001 accepts six chains and two hammer drops for score 6600. The run ends with a gateway `session_ended` error `Access is denied. (os error 5)`. Last Forging server reply is frame 10693; first later unanswered chain is frame 10743, whose `(3,10),(3,9),(3,8)` values match the frozen saved board. Eight subsequent native chain requests have no server reply. Later client prediction/visual mismatch occurs after this IO failure and does not establish a gravity implementation defect. The old log has no syscall stack or identified reader process.

The entire final gameplay object equals the already-proven stage-6-acceptance checkpoint except exactly: active Miner14→Blacksmith16, first stage6 count 0→1, and one allocated incomplete Forging instance/board/timers. Ingredients, all inventory/equipment, coins, XP, completed counters and every other gameplay field are unchanged. Forging remains Playing, six chains/two hammers/6600 score, completed=0; no recipe output or reward was committed.

Authoritative final proof: `lab/runs/client-baseline-20261008T173814Z/analysis/native-inventory-smelting-forging-final-proof.json`, SHA256 `0986b04f5fe50fdb332d16e1a15e03b3c12a370a8798a85622da829d7aaef01b`. All 12,525 datagrams are hash-verified; 12,506 are gateway frames, 19 login frames, 12,504 gateway CRCs. Reassembly is complete; final live index equals final manifest; all services stopped. Startup PC payload remains byte-identical to original community serialization, and the complete corrected 311,622-byte catalog matches the compiled original writer. Native inventory and settled Gear Up journal visuals passed. Known unrelated asset HTTP404 requests remain recorded. Wire proves requests, not which person operated controls.

Restart 181038 proves persistence and cancellation of the incomplete Forge, with no economy changes; see [[Native Forging Cancellation Restart 181038]]. No reviewer source, save, movie, release or native-input mutation occurred.
