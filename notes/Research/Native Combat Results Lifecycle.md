# Native combat results lifecycle

The original client separates recording a result, populating score rows, presenting its score screen, and requesting return to the world. Native154226 confirmed that the first lifecycle repair reached presentation, but the original Flash movie automatically closed because the server supplied no score/stat/reward pages. The second narrow Rust repair supplies actual defeat counts and zero-point score rows before presentation. Unit, compiled community writer and nearest gateway integration checks pass; native v3 modal acceptance remains pending.

## Original-client proof

| Step | Exact original behavior |
| --- | --- |
| GameOver39/18 | `009ad480` stores won/lost, score, time and total fields; it does not call the score-screen presenter. |
| StateRemove39/19 | Dispatch `009bf890` case0x13 calls `009bf190`, then `009b7170` ShowMiniGameEndDialog. The existing fifth post-ObjectiveData boolean U12 becomes state+0x908 and already enables this path. |
| Lua presentation | `MinigameEndHandler:setMiniGameEndData` calls `MiniGameHandler:SetMiniGameEndData`; original Lua36641 sends Type4/1 to `ScoreScreen:Show(status,4)`. The native Lua dispatcher `009797a0` calls Lua synchronously. |
| Results close | Original scoreScreen.gfx AS2 `close()` sends `Hide`; `m_CloseButton.onRelease` calls `close`. Original Lua `ScoreScreen.OnUserEvent` routes Hide to `ScoreScreen:ExitScreen`, which calls `Ui.LeaveMiniGameOrEncounter`. |
| Native close packet | UI binding `LeaveMiniGameOrEncounter` stringVA0x182d8c8/table0x182de18 points to `00c172c0`. Chain `008d2730` → `009bd240` → `008a31a0` creates EncounterParticipantRequestExitPacket41/109 with both header ints zero, then replaces only the second int with client+0xc10, populated by41/107. The wire is **41/109 [0, active launch instance]**, not [411, instance]. |
| Fighting Quit | `QuitMiniGame` stringVA0x182d1f8/table0x182e120 → `00c17470` → `00939100` uses the same constructor and instance header. Leaving during Fighting does not imply a win. |
| Return | The close handler deletes local minigame state after issuing the exit request. The immediate39/7 following39/19 is a removal acknowledgement, not a user close action. |

Native serializer `00913db0` writes u16 main, u16 sub, i32 first header and i32 second header. Instance1 close is exactly12bytes, hex `29006d000000000001000000`, SHA256 `81e1a012e9415d916158206290d971bb1fa360819937e2f150ff9c729aec907b`.

Original Lua HUD.showMain unconditionally calls `ScoreScreen:Hide2` during world presentation. This establishes that zoning can hide the dialog; whether that was the complete observed modal failure still requires the replacement native run. Original AS2 `createScoreScreen(myPhrase,statusNum,zoneName,gameType,totalScore)` only calls `statusHolder.setText`; it ignores gameType. The Lua hub remap4→1337 is therefore not a Flash visibility gate in this preserved movie.

**Logging correction:** original Lua Logger defaults `m_Enabled=false` and its print method suppresses messages while disabled. Missing `SetMiniGameEndData` lines in091836/092345 do not prove that callback failed. The prior [[../Reviews/Native Cave Lifecycle 091836 and 092345]] recorded an investigation boundary, not a reliable negative callback observation. There is no LuaErrors.log in the reviewed win run; absence of that file is also not a substitute for visible modal acceptance.

## Narrow implementation and tests

Only `prototype/osfr-tools/src/encounter.rs` and its encounter tests change for the results repair. Terminal Won/Lost now emits39/18,39/47,39/19, retains the terminal encounter, and does not zone. A private connection-local presentation boolean suppresses duplicate Remove, resets on begin/retry/zone-ready, and is not serialized. Native41/109 accepts first-header0 (plus existing authored encounter-ID compatibility) only with the exact current instance. Only a close or guide Exit returns to the world.39/7 never exits. Existing Flash26/42 terminal-close support remains.

All14 ordinary encounter tests pass after formatting. They cover no automatic zoning, no duplicate Remove/reward, native fighting Quit without completion, rejected stale/wrong/short/long close requests, loss/retry presentation reset, saved terminal-state reload, and actual defeated counts through partial-loss/restore/retry. Both private tests pass: the existing opening-content/community packet oracle and `native_results_close_and_terminal_packets_match_compiled_community_writers`. The C# oracle links unmodified pinned community packet writers; Won, Lost, Remove, native-close and both score-row vectors match Rust byte for byte. The nearest private `opening_gateway` integration test passes. No release build or native input was performed by this investigation.

Private reproducible proof: `lab/reports/trace-native-combat-results.py`, `native-combat-results-trace.json`, `native-combat-results-disassembly.txt`, and `lab/tools/native-results-oracle`. The trace hashes the preserved exe, Lua and native pseudocode and records original function line numbers and executable binding/disassembly. Original ScoreScreen source export is `lab/decompiled/client-ui/score-screen/scripts/frame_1/DoAction.as`, from catalog-verified `pack_19/006110.gfx` (scoreScreen.gfx62126bytes, CRCb23c0d71);97 AS2 scripts were exported without modifying the asset.

## Native acceptance fixture

Fresh synthetic branches are now in `lab/state/native-combat-results-v3`, generated from the preserved083946 `character-before.json`; the v2 and old `native-state411-lifecycle-v1` evidence remain intact. They preserve inventory, wallet, profiles, quest state, identity and rewards. This is a one-hit control experiment, not a second campaign completion.

Win SHA256 `411edfc9b446c7168b27cf828baf6c10d03eafa0090378ab521da893cb57e875`; loss `12504b0a35f20fbc2b59cbab3a4ed4ccb4b0623b8f521fcbd70faa0ed254409f`. Fixture preparation launches nothing. Required native evidence is a visible terminal score page before any world load, exact39/18→39/47→39/19 ordering, actual defeat counts, ignored39/7 ACK, genuine41/109 [0,instance] only after Close, return WorldReady and same-session journal, with no repeated dungeon reward. These gates are **pending** until the main agent runs the updated release.

## Native154226 failure and exact score-page cause

Independent audit hash-checks all1847 datagrams and verifies1826 gateway CRCs with complete reassembly. GameOver39/18 and StateRemove39/19 arrive at79.313s; client39/7 acknowledges at79.328s. With no Close or Exit input reported by the main agent, the client sends genuine41/109 [0,1] at82.422s. World31/12 follows at82.469s. Thus the revised server waits for the actual native exit request; the request itself was automatic.

Original scoreScreen.gfx AS2 `initializeState()` calls `close()` when `navM.getCount()==0`; close sends Lua event Hide. Original Lua `PopulateLootWheel()` calls `SetStates()` and then initializeState when no wheel preview is available. `SetStates()` adds pages only for nonempty RewardsData, StatData or ScoreData and an allowed minigame type (Type4 is allowed). `Show()` parses the native score datasource before population. No39/47 was sent in154226, and the authored reward/preview bundles are empty, producing zero navigation pages and automatic Hide→ExitScreen→41/109.

The original native dispatch39/47 reaches `009bb030`, which reads the base header, parses row count with `009b82d0` and each row with `009b2b30`, then copies the rows into MiniGameState. The pinned community `MiniGameGameEndScorePacket` writes the same layout: base header, i32 count, then string and five i32 fields per row. The first integer is exposed as Icon ID in the native score datasource, despite the community field name NameId; using-1 avoids an icon override. The next integer is Score Type, not just sorting order. Original `CodeStringMappings.txt` verifies `scoreEnemiesDefeated`→117298 and `scoreTotalScore`→117296.

The patch supplies two original labels: type0 enemies defeated with the actual zero-health enemy count and type4 total score. Both have zero points, matching the current scoreless combat rules/GameOver. It does not invent original scoring/reward economics. The compiled C# oracle's102-byte Won and Lost row vectors are in `lab/reports/native-results-vectors-v3`; SHA256 `66559e6173093d9d3a42e07d18d4761187a8e20fc058d76e8f7348082b2db923` and `8e69efeb6e47f842a269cd91a6f6246e5dd3489d01d3164de535fd2262cf4820`.

Independent screenshots show GoalComplete, then returned-world news, then the correct same-session native journal. They do **not** accept a score modal. Items, opening quest state, rewarded411 and identity remain unchanged; coins70→75 exactly match newly recorded Farnum discovery2 reward5. Reproducible failure proof: `lab/reports/verify-native-results-auto-close.py` and run154226 `analysis/native-results-auto-close-proof.json`. Original v2 fixtures/run artifacts remain untouched.
