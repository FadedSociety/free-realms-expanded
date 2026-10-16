# Native cave lifecycle and journal recovery

Independent offline acceptance of two **separate synthetic combat-health fixtures**, preserving the original campaign ending SHA256 `9ebc09e654ba8591bc1c02b456e4255e8b1cd73416079348cf9929b73d2ccea3`. These verify the StateID411 repair, normal key1 routing, Retry and same-session exit cleanup; they do not replay the whole campaign dungeon balance.

| Evidence | Win091836 | Loss092345 |
| --- | --- | --- |
| Datagrams independently hash/length-checked | 2,168 | 2,640 |
| Gateway CRCs independently verified | 2,147 | 2,619 |
| Reliable reassembly / entire initial self compiled-writer match | Pass | Pass |
| Actual native36/10 request | bar1, slot0, Target0, selfGUID1 | Same |
| Server lifecycle39 | Start17, won18, remove19 | Start17, lost18, retryStart17, remove19 |
| State/Group/Game on all lifecycle replies | 411/-1/411 | Same |
| Native client remove acknowledgement39/7 | 411/-1/-1 | Same |
| Same-session native journal after Exit | Correct title and objectives visible | Correct title and objectives visible |

Both start/remove packets match the unmodified pinned community writer. Both original native logs now record OnGameOver and LeaveMiniGame; the latter also enters ShowMiniGameEndDialog. The correct Other Quests page displays The Last Best Hope, completed Defend Darkthorne and pending Talk to Pappy in both sessions. This accepts normal journal dispatch after exit without restarting the client.

The win fixture begins with only the boss alive at1HP. One real key1 attack wins and Exit follows the story route to Farnum. Its source had not discovered the farm, so the legitimate discovery2 grants5coins and25BrawlerXP: coins70→75, XP50→75. No dungeon reward repeats, items and all quest counters are unchanged. The loss fixture begins at1playerHP with500bossHP; one key1 attack leaves the boss400HP, AI defeats the player, and Darkthorne's native radial Retry restarts all enemies/player. The accepted retry remains idle without enemy movement/damage for92.078seconds before Exit. Its wallet70, items, quests, discoveries and XP are exactly preserved.

The modal combat score screen remains unresolved. Original Lua36641–36660 routes Type4/1 through `ScoreScreen:Show`. Native OnGameOver009ad480 stores result fields; it does not directly open that dialog. Native ShowMiniGameEndDialog009b7170 requires the per-state results flag+0x908 and an unset already-shown byte+0x2c0, then resolves `Client.HandlerMiniGameEnd` before calling its Lua end-data method. The original info reader009bdd70 and state copier00c42c60 prove that flag is the **fifth boolean after ObjectiveData[]**, named U12 by the community. Existing authored details already set it true. Neither run logs the Lua SetMiniGameEndData callback. Changing an arbitrary tail flag or adding an unproved popup packet is not justified by this evidence; live Lua-handler resolution/presentation remains an investigation boundary.

Private proofs: each run's `analysis/native-state411-lifecycle-proof.json`; reproducible helper `lab/reports/verify-native-state411-lifecycle.py`; exact original native gate extraction `lab/reports/native-minigame-score-gate-proof.json`. Screenshots independently viewed: win `state411-objectives-after-exit_small.png`, loss `loss-retry-exit-journal_small.png`, native win view and loss Retry menu. This audit sent no input, launched no game and changed no save/identity/shared Rust source.

Follow-up correction and repair: [[../Research/Native Combat Results Lifecycle]] proves original Lua Logger is disabled by default, so the missing callback log above cannot establish callback failure. The exact native Close is41/109 [0, active instance], and39/19 is the score-screen presentation trigger. A narrow terminal39/18→39/19/no-zoning repair and exact native-close acceptance pass unit and compiled-writer checks; replacement native modal acceptance is still pending. The original runs and their wire proofs remain unchanged.
