# Tasks

## Current target

Active target (2026-10-15): **playtest night**, about 10 players together on our server. The ordered work is [[Design/Build Plan]] (milestones M0–M4), and the verified state of each system is [[Design/Build Ledger]]. This task list keeps detail items; the plan decides the order. The earlier [[Design/Opening Playthrough]] target is complete for the 17 local stages (native 103605).

## Next

- [x] **Studio toolkit** ([[Design/Studio Toolkit]]). All nine tabs are native-verified. The bot client is verified against the live gateway, though never alongside a native client. Terrain, the Map tab, marks, the NPC placer and behaviour presets are built; the placed NPCs and names are native-verified (run 20261016T011257Z). Remaining: bridge and building heights. Continued in [[Design/World Builder]] and Build Plan 3.3.
- [ ] **OSFR "Mounts and Pets" update (The owner, 2026-10-15, later):**
  - Map the community OSFR server's new content into ours: pets with navmesh, world events (Werewolves vs Vampires dance-off), more mounts, collection nodes (158) and collection coins (118) with community placements, Mystery Cake scaling, game-time sync, and portrait refresh.
  - **Check the OSFR repo's license first.** Treat code as reference only, as with Legacy (AGPL). Placement data may be reusable with credit.
- [x] Install REA and Ablation with pinned provenance; test local Laya on the PC and record compatibility/accuracy limits in [[Research/REA Ablation and Local Laya]].
- [x] Repair the v18 direct kart-pose dispatch gap and verify rocket/mine firing and wire lifecycles on frozenv20 native064034; mine visuals after movement and hit/effect acceptance remain open. See [[Reviews/Kart Mines and Rockets Acceptance]].

- [ ] Repair native Mining dead board: six normal chains/3000points, zero ore and no legal moves. Prove original fall/reshuffle compatibility, then record a whole round and restart.
- [ ] Review optional Harvesting111/Chef45 gateway, add the two missing original reward definitions and verify all four outputs in a new immutable content package, then test actual native completion/atomic payout/restart.

- [x] Install Scrapling/sqlite-vec, acquire the public wiki and verify populated keyword/semantic search with revision provenance (3,250 pages/3,996 chunks).
- [x] Recover verified creator terrain/starting outfits; inspect native creator/naming UI and replay its server0 submission through the fixed Rust login service.
- [x] Pass private packet simulation of the locally bound 17-step opening, cave completion/return and reloads. This is not native acceptance.
- [x] Verify native supported-appearance creation/world entry and fresh inventory (run072258). Original unsupported skin7/8 remain excluded; Bennett save preserved.
- [x] Replace the inherited research journal with typed opening quests and native incremental progress; correct title/objectives visible in084544 and same-session post-cave091836, quest remove/add observed in090058. The full quest chain remains a separate acceptance gate.
- [ ] Verify cave floor/actor reachability, then record the full native opening/dungeon/restart playthrough.
- [x] Repair cave pursuit/attack animations and melee reach; complete the reconstructed cave and reward/return in the actual client (081219), then verify normal key1 combat separately (083946).
- [ ] Resolve original crafting minigame/material and tutorial gaps before making original-content coverage claims.
- [x] Integrate the original Cooking movie/protocol into a new versioned campaign; pass focused atomic economy/gating/reconnect checks, explicit migration preservation checks and the full 17-step packet simulation. Native mouse play remains pending.
- [x] Trace actual native Cooking Go and shared payload writers; repair only phase/instance-bound game sentinel compatibility for Cooking/Smelting/Forging, prove NUL-inclusive length and pass captured/compiled regression plus full17 native-header simulation. Native phase/statistic transmission remains pending.
- [x] Verify native Cooking launch/first Tenderize round, capture original statistic suffix and preserve failed award acceptance; narrowly repair final numeric fields, pass27 ordinary/9 private/full17 checks and independent unchanged-writer review. Whole native recipe remains pending.
- [x] Prepare optional reconstructed objective locale text using the unchanged community writer; preserve all stock entries and opening/save identity, pass all51-goal/full-self/integration/preflight checks and independent review. See [[Research/Opening Journal Reconstructed Labels]].
- [x] Verify the paired journal locale and character-selection title options in native162238/164218; readable labels, live progress and saved name preserve the current campaign hash. Stock row clips the long name.

- [x] Continue targeted native analysis from the saved Ghidra project. Full analysis completed 2026-10-10. The native map is paused at 824/897 shards (10-14 rate limit; 6 stuck shards need a reset).
- [x] Inventory quest, job, NPC, zone and activity definitions (W22/W27/W28/W56/W58: 653 quests, 1,199 NPCs, 75 dungeons). The coverage ledger is [[Design/Build Ledger]].
- [ ] Diagnose native inventory search access violation; preserve the failing run and avoid claiming a cause without a fault address or controlled reproduction. **Mitigated:** W03/W09 withhold owned items without definitions, and "hammer" no longer crashes (SB 215401). The cause is unproven for every query.
- [ ] Recover one original enemy/quest mapping; quest 1617 Petty Yeti matching remains disabled rather than counting unrelated Robgoblins.
- [ ] Implement original soccer from archived arena/config/team/ball evidence through normal completion and rewards; current captures do not prove the full lifecycle.
- [ ] Restore native item-use requests/effects and missing icon delivery from preserved assets.
- [x] Replace captured initialization templates with typed serialization (all 46 built; native SelfData 011719, 2026-10-14). The template fallback is opt-in only.

## Done

- [x] Implement and actually play a reconstructed three-enemy quest; grant 25 coins and increment Wheeler on the Cob from 9 to 10 exactly once.
- [x] Equip Small Stitched Pants through native C gear UI; inspect changed model and saved profile1/slot4/instance2.
- [x] Complete the reconstructed checkpoint course through real walking in 18.456 seconds; grant 10 coins and persist the record.
- [x] Add byte-verified community item category initialization; native Stuff and Supplies > Food & Drink now populate.
- [x] Restart Rust and client; verify native wallet35, reward quantity10 and pants, plus exact saved gameplay state.
- [x] Pass 73 ordinary tests, explicit new private oracles, formatting/release and seven final-binary UDP checks.
- [x] Analyze original soccer/racing/derby captures; validate 37,327 CRCs and record incomplete completion evidence.
- [x] Implement bounded reliable sending, ACK/retransmission, session controls and message bundles; verify loss, rollover, and stale-session handling.
- [x] Replay the actual-client capture explicitly: 44 reliable requests, 46 byte-identical initialization replies and seven dynamic time replies.
- [x] Implement one-character/one-zone loopback Rust gateway with validated private references and atomic character snapshots.
- [x] Connect the actual client to Rust, recover deliberately dropped server sequence 6, walk with server-observed updates, then restart and restore the saved position.
- [x] Pass 43 ordinary tests, the explicit original-client gateway oracle, seven live UDP session checks, formatting, and release build.
- [x] Install requested skills, RTK, and Obsidian; prepare the project and vault.
- [x] Record the supplied client and server in [[Research/Target Intake]].
- [x] Preserve both ZIPs, verify SHA-256, safely extract, and hash extracted files.
- [x] Identify the .NET server and native x86 Windows client.
- [x] Pin community reference repositories under `lab/references`.
- [x] Install private .NET/ILSpy, Rust, JDK, and Ghidra toolchains.
- [x] Recover seven managed assemblies to 769 C# files, including dependencies.
- [x] Finish the first native pseudocode export: 153,423 function bodies; verify its index and reopen the saved Ghidra project. Initial analysis remains partial.
- [x] Write the Rust pack catalog/extractor; parse all 24 packs.
- [x] Extract and CRC-check all 108,547 asset entries, preserving duplicates.
- [x] Implement the initial session handshake in Rust and verify it against synthetic fixtures from the supplied serializer.
- [x] Pass eight unit tests and one actual loopback UDP integration test.
- [x] Start a temporary copy of the supplied server, verify its actual handshake against the fixture, and stop the process.
- [x] Review Universal Modder's eleven skills; install/configure its private CLI, Windows helpers, and GPU capture.
- [x] Search XeNTaX/ZenHAX/QuickBMS research and run the applicable community pack extractor; independently match all 6,074 entries in one pack.
- [x] Run existing FR ADR Toolkit readers on Chatdy/warpstone, validate model bounds, and inspect a static textured preview.
- [x] Implement bounded Rust CRC/compression framing and reliable receive; verify 105 compiled-serializer vectors and sequence rollover/reordering.
- [x] Replay actual supplied-server synthetic-login traffic in Rust: 1,253 frames and 46 messages/627,566 bytes; all messages match and CRC mutations are rejected.
- [x] Pass 19 ordinary Rust tests and explicit local oracle checks; build release binaries.
- [x] Resolve the real client's missing D3DX dependency and asset-manifest startup failure using official/community instructions and disposable copies.
- [x] Establish an actual game-client baseline with the supplied walking emulator: local connection, world ready, inspected game screenshots, and walking with position updates observed by the server.
- [x] Capture the real client's local session/login/zone sequence. Preserve raw traffic and configuration outside Git; exhaustive application decoding remains ongoing.

## Native acceptance follow-up, October8 16:22UTC

- [x] Prove and repair shared original crafting start/payload sentinel headers; pass focused transaction/original-writer checks and144 ordinary tests.
- [x] Add optional nonblocking live packet inspection with isolated native replay proof; final capture remains authoritative.
- [x] Verify readable mapped journal objectives and actual character names in native run162238; finalized original-writer and native capture review passed.
- [x] Complete all six original Cooking rounds with normal mouse input; independently verify score5199,28 original statistic reports, fixed4-each consumption and food35x1 once in164218.
- [x] Prove and implement native crafting39/18â†’real39/47â†’39/19 results with terminal ACK ignored and actual26/42 Close retained; pass focused compiled community/gameplay/17-stage gates.
- [x] Accept native171251 Cooking score page5133 held62seconds, automaticACK retention, actual26/42 Close, Mae hat/10coin/25ChefXP turn-in and no repeated reward on an immutable-pregame branch. See [[Reviews/Native Cooking Finish and Mae 171251]].
- [x] Repair startup35/37 catalogue payload framing and accept restored inventory/gear visibility in native173814: whole original catalogue/self comparisons, All Items, hats and Chef head gear pass. See [[Reviews/Native Inventory Repair 173814]].
- [x] Complete native Gear Up: Miner, five gathers, four Smelting rounds, actual5497-point win/recipe, terminalACK retention, playerClose and Cleetus turn-in (+10coins/+25MinerXP). Final score-page visual remains unretained; exact lifecycle/state evidence is preserved.
- [x] Repair reproduced transient Windows atomic-save denial with bounded atomic-rename retry; pass independent transient/permanent lock checks and153ordinary tests. Native181715 then completes18Forging chains/5hammers/13100points, genuine results/Close and once-only Cleetus turn-in (+16coins/+25BlacksmithXP). Forge/gravity remains unchanged; exact old lock owner is unproved.
- [x] Restart native181038 and original whole-PC oracle:154349bytes and entire gameplay preserved except safe cancellation of interrupted Forging; unused recipe materials remain. Native181715 repeats exact startup restoration with the repaired build.
- [x] Complete Fighting off the Pack in native183551: ten normal key1 attacks defeat all five pursuing wolves, native recording shows wolves/motion, goal5/5, HP525. Cleetus grants18coins/25BrawlerXP once, wallet164; saved stage7 rewarded. Whole-state/reward proof is being finalized.
- [x] Apply and accept the separately reviewed one-actor anvil relocation: guarded new-save migration preserves all other bytes/identity. Native183551 visibly separates the anvil from the boulder, approaches on foot and sends ordinary interaction; Brawler correctly receives the Blacksmith requirement. Forging at the new position will be exercised in stage15.
- [x] Implement original crafting Quit39/6 and accept native Cooking Playing Quit with unchanged economy. Offered current-instance Quit and native Smelting/Forging Quit remain separate gates.
- [x] Verify v3 actual native win/loss score pages remain open until player closes them; finalized163518/163829 independently prove4/3 defeats, zero totals, about59s held modals, genuine41/109 Close and same-session journals. See [[Reviews/Native Combat Results v3]].
- [x] Verify native character-selection title repair in163518: saved name/Add a Character replace1/0 and Go preserves identity. Long-name tail clipping remains a cosmetic limitation.
- [ ] Accept the loss-results boss-display cleanup in a fresh native run: original UI/packet evidence, narrow loss-only repair, all17 focused encounter checks and independent review pass; visual loss/Close/Retry remain pending. See [[Research/Native Loss Boss Display]].

## Native Gloam and restart, October 8 19:33 UTC

- [x] Complete The Gloam Council through ordinary native movement and three basic attacks; visible Gloam, 300 damage, goal once, player895HP. Pappy turn-in gives20coins/25BrawlerXP, totals184/175. Next interaction accepts Where's the Help? with no duplicate reward.
- [x] Restart native193318 and independently compare the entire gameplay plus original serialized PC/catalogue: items, gear, objectives, enemy deaths,184coins and895HP preserved. Native screenshot confirms equipment and health. One native f32 rotation normalization is recorded separately.

## Native Bartle, artifacts, Mac and prompt restart, October 8 20:31 UTC

- [x] Complete Where's the Help? and all three Pull the Plug artifacts through ordinary native walking/interaction; each exact prop disappears and advances its ordered goal once. Bartle turn-ins add 20 coins and 50 Brawler XP in total.
- [x] Defeat Mac with five normal basic attacks, turn in Cobblestone Showdown for 25 coins/25 Brawler XP, and accept Off to the Queen. Final 193318 capture independently rederives all 11,531 datagrams and whole-state reward boundaries; wallet229/XP250/HP775.
- [x] Review native Mac recording for actual model approach, changing gray-hammer poses, contact damage and disappearance. Complete unobscured swing/death animation remains unproved.
- [x] Repair only the native AddNpc proximity float; permanent synthetic unchanged-community regression fails on frozen old source and passes after the change. Formatting, 154 ordinary tests and release build pass; 29 private checks remain explicitly ignored.
- [x] Restart native203139 with whole gameplay/original whole-PC/catalogue preserved; verify all 18 delivered opening actor packets carry12.0. Actual X menus at7.15m/10.96m, away/outside13.78m suppression and ordinary choices preserve all gameplay and rewards.
- [x] Complete stage 12 Queen road, discovery, talk and turn-in; see the native Queen continuation below. Stages 13â€“17, final restart and broader historical coverage remain open.

## Native Queen continuation, October 8 20:56 UTC

- [x] Walk the original main road, Sanctuary climb and both lilypad bridges to Queen Valerian through normal A/D/W input; no jump, teleport or state edit. Two heading guards pause and are reviewed/corrected with normal turns, with no observed physical obstruction. Final safe interaction position is9.47m fromQueen; exact road804 center is not walked into.
- [x] Trigger discovery4 once (+5coins/+25BrawlerXP), the separate25m palace-visit objective, and actual Queen X menu/choice to complete stage12 (+25coins/+25XP). Native level4/quest completion and wallet259 are visible; HP775 and gear/materials remain.
- [x] Repeat Queen menu/choice and retain byte-identical saved state, with no duplicate reward or accidental stage13 acceptance. Immutable stage12 turn-in/repeat SHA3a9d38078449b7dd95fdd8263a733b9d77e44b24ffe740d1e0a44deff07e2681.
- [x] Cleanly close prompt/Queen recording and deliberately stop native203139; harness exits0 with7065datagrams and allservicesstopped. Final independent transport and route receipts follow in owned reviews.
- [x] Return to Mae for stage13; then stage14 mining, stage15 Smelting/Forging at new anvil, stages16â€“17 job-center/class/hat checks and final restart. Historical opening gaps remain separate and incomplete. Native103605/111125 reviewed; see [[Reviews/Native Opening 17 103605]].

## DeepSeek worker setup, October 8

- [x] Install codex-on-crack and Codex Router; verify local service health and preserve the lead configuration.
- [x] Configure the user-selected DeepSeek V4.1 Flash builder with max reasoning; static role and router checks pass. See [[Research/DeepSeek Worker Setup]].
- [x] Verify real workers' client-recorded `deepseek/deepseek-v4.1-flash`/`max` metadata, tool execution and returned artifacts. Their session provider label is generic; a specific upstream provider per request is not independently attributed here. See [[Reviews/Ordinary Crafting Quit Independent Review]] and [[Reviews/Recovery Workflow Independent Review]].

## Delegated continuation, October 8

- [x] Write explicit recovery/decompile commands and evidence boundaries; resolve seven independent-review findings. [[Research/Verified Recovery Workflow]] is accepted for its bounded documented scope.
- [x] Test ordinary crafting Quit39/6 before implementation (13 failing regressions), implement the narrow service fix, pass 179 ordinary tests/29 ignored, independently reproduce old failures and review candidate economics/atomicity. Lead formatting check passes.
- [x] Verify real native Cooking Playing Quit, world restoration and unchanged whole economy in005748; fresh013155 restart offered a new instance with materials/completed count preserved. The historical response remains a reconstructed remove-only policy. See [[Reviews/Native Crafting Quit 005748]].
- [x] Verify original left-click movement with a 6.94448 m native move and unchanged whole gameplay; preserve the partial Queen-to-farm recording and separate branch.
- [x] Add user-requested opt-in named-entity developer teleport using the pinned community native location packet; meaningful failing tests, compiled-writer oracle, independent review and native live-prefix relocation/state-preservation audit pass. Developer relocation does not establish walked terrain/collision acceptance.
- [x] Improve arrival direction around competing farm actors; independent code/native prefix review013155 verifies Mae and CookingTable prompts, unchanged gameplay and fresh restart on ec43e1de release.
- [x] Continue stages 13-17 through real menus/crafting/class/equipment controls, and verify final restart. Historical starting-area gaps remain open independently. Native103605/111125 reviewed; see [[Reviews/Native Opening 17 103605]].

## Broad MMO dispatch, October8

- [x] Persist100 scoped pieces and required RTK/Ponytail/Caveman/relevant-game-skill contract in [[MMO Work Queue]] and [[Design/MMO Worker Contract]].
- [x] Verify saved multi_agent_v2 concurrency100; preserve the old pre-relaunch limit failure separately.
- [x] After relaunch, verify100 actual accepted assignments including coordinator100, zero recorded spawn failures. This is total dispatch, not concurrent active or accepted feature count.
- [x] Receive mining/harvesting/fishing/kart/soccer modules and rerun67 focused tests with no failures or warnings. Fishing lead actor/finite-input/provenance findings are repaired.
- [ ] Complete independent source review and original/native integration for those five modules. Tests alone do not establish playable minigames.
- [ ] Receive housing/derby/activity-catalogue/job-catalogue/quest-catalogue work; review and dispatch next exclusive scopes from the100-piece ledger.

- [x] Receive independent mining/kart/soccer/housing review repairs with focused GREEN logs; retain native gates before integration acceptance.
- [ ] Complete harvesting independent review and integrate recovered activity routes; run actual native matches/boards and restart/economy controls.

- [x] Native054421: Cooking5199/fullstats/recipe/resultsClose/Mae stage13; duplicate turn-in unchanged; closed completecapture independently accepted.
- [x] Native054421: Miner switch/ten ore interactions/extra-ore refusal/Cleetus stage14;14/17rewarded,279coins; stage15unaccepted.
- [x] Receive all100 initial assignments with artifacts/reviews; native feature acceptance remains separate.
- [x] Repair actual production ID reuse and reward partial commits;18 focused economy/1365 ordinary full-suite entries GREEN;57private ignored.
- [x] Native candidate economy release/reload and explicit legacy-counter migration; opening/encounter refusal feedback remains to improve.

## October 9 native continuation

- [x] Complete local stages15-17 with actual smelting/forging, ordered conversations, job switches and Chef hat equip in103605; all17 rewarded/309coins.
- [x] Verify Cleetus15/Sunspark17 duplicate interactions and fresh native17 restart111125 preserve the whole save/gameplay.
- [x] Finish independent finalized103605 and111125 capture/snapshot reviews.
- [x] Enable ordinary post-campaign cave replay and pass lead native full-enemy combat/results/exit without duplicate rewards (114509; independent source/native reviews passed).
- [x] Independently accept replay code and full114509 capture.
- [x] Verify the tested goal5000/total5000 score repair in native121834; full capture independently reviewed, economy unchanged.
- [ ] Complete Mining18 native win/persistence before choosing world entry: opt-in gateway reviewed, launch/one chain/pause/quit passed121834; repair blue regular/ore sprites first; both Iron labels are original quality naming.
- [x] Add server-owned soccer authority candidate; independently find and repair sender spoofing/time-budget/restart/half-side gaps.
- [ ] Recheck soccer repairs independently, recover native kick/pass packets and integrate a playable match.
- [ ] Resolve mining/harvesting entry/profile binding gates before native launch.

- [x] Repair Mining tile indices with old/fixed differential state proof; confirm five colours/iron quality sprites natively125556.
- [ ] Complete native Mining125556 whole round/result return/restart; private mouse driver ACK/terminal/release handling under repair.
- [x] Repair Harvesting job binding to Chef45; focused24 and independent4 probes pass, candidate remains unregistered.
- [ ] Integrate opt-in Harvesting111 gateway with conservation/atomic/once-only tests, independent review and original-client playthrough.
- [ ] Resolve OS-blocked piece_097 test execution through normal trusted tooling; current broad gate explicitly partial (1405pass, eight unexecuted).

## Current continuation gates

- [x] Review native134930/143211 failure captures. Exact chain19 replay disproved the pending-fall hypothesis; preserve that qualification in [[Research/Mining Pending Fall Reshuffle]].
- [x] Repair wildcard move detection from captured chainless board; independent path enumerators agree across 400 generated boards. Frozen CB0 release is under native test151307.
- [ ] Complete Mining18 normal whole-round win, actual Close/world return and reward/restart verification. Native151307 ended Lost with two collected ore, no full quota/normal Close/world return; independent final audit pending.
- [x] Enforce Harvest runtime tile registration and excluded row0/col0 border; independently review source and immutable v2 package.
- [ ] Measure native Harvesting geometry, complete ordinary round, verify atomic payout/Close/restart on the v2 package.
- [x] Review original tutorial41/2 field mapping. Original historical quest/camera/barrier bindings remain missing; no production pilot yet.
- [x] Complete latest qualified root gate: 1407 ordinary pass, zero observed failures, 56 ignored; four explicit private checks pass.
- [x] All five previously blocked targets059/072/074/081/099 executed through normal tooling in the later builder full suite:1468pass/0fail/59ignored. No security changes. Pre-registration/pre-powerup revision only; next root gate remains required.
- [x] Independently review Kart lobby102/2 framing and client-derived finish contract; native capture has no finish. Separate worker verified raw PE sort callback as well.
- [x] Implement full native five-table Kart lobby codec; independent32 probes and private byte-exact roundtrip accepted. Root registered module/test imports; new root gate pending.
- [x] Recover actual Kart/Derby/Soccer opcode31 scene names and community serializer; original client loads preserved geometry.
- [x] Accept and register scene-transition/progress codecs, preserving stricter documented policies and signed-lap boundary correction.
- [x] Add native102/1 optional catalogue response with actual RED/GREEN and independent review.
- [ ] Run new combined root gate; native-accept lobby and actual Kart entry, vehicles, controls, gates, finish and return.
- [ ] Review clickable Mining950-953 powerup implementation, rerun root gates and accept a full native Mining round. Timed900-902 remain separate.
- [x] Independently finalize failed native151307 capture/save/media review. All17 local rewarded stages preserved; no normal Close/world or full Miningwin.

- [x] Repair Soccer native sub14/sub16 widths and preserve server-owned authority;14 independent probes and250/1216 private captured roundtrips accepted.
- [ ] Finish synthetic Soccer test fixtures/production imports and combined root gate; recover kick direction and implement native match route.
- [x] Correct Mining shuffle same-value placement and RNG consume/refill ordering; independent runtime review accepted4abe6b07. New root gate and native acceptance remain open.

- [ ] Finish Mining action-direction, shatter neighbour order, power/pool target-count and random-queue fixes; rerun independent review on frozen source.
- [ ] Independently review Housing placement reply prefix/directions and recover remaining94 bytes plus127/44 before building a native adapter.

## Lead gate and deeper oracle findings

- [x] Root frozen038 gate:1514 ordinary pass/0fail/65ignored plus11 explicit private compatibility checks, source unchanged.
- [x] Full registered Soccer corpus2917/2917 byte-exact and synthetic fixtures accepted.
- [x] Independently recover complete Housing fixture-update tail;9 full replies exact and nonempty source vectors reviewed.
- [x] Repair and independently verify actual Mining shuffle permutation against expected-board oracle (4abe6b07); root re-gate/release remains a separate next step.
- [x] Correct private Mining feasibility harness bottom row13→10; corrected1127-chain quota completion and four-second pacing failures recorded. Prior zero-ore reports superseded; no tuning activated.
- [ ] Finish/review lab-only Housing codec with dictionary-key/action records and nonempty compiled oracle before integration.

## Current source acceptance checkpoint 2026-10-09T17:28:55.185080+00:00

- [x] Independently verify repaired Mining4abe source: actual shuffle outputs, queue advertised batches, and default zero-powerup parity (3 executed runtime probes).
- [ ] Finish independent Housing codec malformed/nonempty/44-capture review, then register production module and framing tests.
- [ ] Run a new whole-tree root ordinary/private gate and freeze a new gateway release after source integration.
- [ ] Evaluate clickable Mining round feasibility on the repaired engine using private reconstructed settings; no shared balance changes.
- [ ] Recover concrete Soccer input-direction and Kart selected-race/start contracts before native integration.

## Root source gate 2026-10-09T17:37:22.902978+00:00

- [x] Register independently accepted Housing codec, correct body/inner units, add framing RED/GREEN and44-vector private gate; independent root-delta review passed.
- [x] Complete root1530 ordinary/0failed/66ignored plus12 explicit private checks on unchanged4abe/Housing9091; build and preserve59032cfa gateway.
- [ ] Repair one/two-tile OnIdleMsg parsing exposed by the clickable driver; retain chainminimum3 and malformed/bounds atomicity, independently review and re-gate before native deployment.
- [ ] Correct all four private click-target pools; run3 needed after parser repair. Run1/2 generator failures do not establish powerup feasibility or justify tuning.


## Fresh checkpoint 2026-10-09T17:55:17.186517+00:00

- [x] Complete fresh root gate1536/0/66ignored and12 explicit private checks; freeze21233dc7 release without deployment.
- [x] Independently accept short-idle ACK repair with18 conservation/rollback checks; preserve three-tile chain minimum.
- [x] Run corrected clickable private trials; accept failure evidence only, no balance changes.
- [x] Correct Housing identity chronology/authority qualifiers and Kart fully recovered entry layouts.
- [x] Independently verify Soccer direction/charge dataflow.
- [ ] Finish/review private Soccer state-to-authority adapter; then plan ordinary native match integration.
- [ ] Verify natural private Mining completion handshake; select evidence-based playable starting-round balance before native acceptance.
- [ ] Native Mining/Harvest whole-round victory, normal results/Close and restart remain unaccepted; UI input paused.

Evidence qualification: old173220 success log/pin files were accidentally overwritten during a failed CR-line-ending rerun. Original receipt/frozenbinary retained; fresh174628 gate supersedes it. See MODLOG and disclosure.


## Private playability checkpoint 2026-10-09T18:05:13.985122+00:00

- [x] Verify normal private Mining quota/Won/GameOver/Close lifecycle without forced state.
- [x] Test three valid even-quota starting-round presets;40%6+6 finishes85chains/340modeledseconds.
- [x] Prepare separate49file Mining testpackage with46/47basefiles identical and onlyMiningbalance changed.
- [x] Finish lead independent Socceradapter13additionalbehavior tests;14buildertests remainseparate. Correctscale/RED claims; candidateunregistered.
- [x] Finish independent Mining starting-round replay and record pre-Close quota/timer state;17/17checks accepted.
- [ ] Perform guarded original-client full Mining/Harvest acceptance once shared UI control is resolved; allnativewin/reward/Close/restart gates stillopen.


## Community references reviewed October 9, 2026

- [x] Download, pin and hash both requested Edenfps repositories, including Sanctuary PC-2009 branch.
- [x] Run bounded existing toolkit readers against unchanged original-client asset copies; preserve source overlap and limits.
- [x] Record useful fishing/compatibility/loading clues and unfinished Housing/Kart paths in [[Research/Edenfps Toolkit and Sanctuary Branch Review]].
- [x] Save explicitly requested qualified SOE evidence as a Codex memory extension note.


## Reward protocol research October 9, 2026

- [x] Inspect/hash supplied locale_ids.py, pack_extract.py and217-entry Lua archive without executing bundled binaries.
- [x] Resolve normal reward field/template selection and deferred-bank UI callbacks;36CPU/Lua/helper probes pass.
- [x] Create shareable [[Research/Reward Toast and Deferred Bank Findings]] with offsets, Mystery Chest kind16, locale helper limits and native-test boundary.
- [ ] Integrate ordinary reward presentation and run native coins-only/XP-only/both, duplicate/reload acceptance; this investigation did not perform that separate gameplay change.

## Mystery Chest checkpoint October 9, 2026

- [x] Hash/preserve four donor packet/handler files; resolve check16/use15 and item-kind16 against original native code.
- [x] Integrate optional Rust chest/key/reward transaction with owned-item checks, rollback and session/replay guards.
- [x] Pass1549 ordinary tests and4 explicit private chest checks; preserve corrected subtype16 original-baseline failure.
- [x] Original client: Cancel, confirm/consume, Kangaroo reward panel and inventory ownership.
- [x] Fresh original-client restart: whole gameplay unchanged, chest/key/Kangaroo1/1/1, no repeat award; both captures fully reassembled and services closed.
- [ ] Recover retail chest definitions and historical loot tables; current chest910100/fixed reward are reconstructed.
- [ ] Verify other chest reward types and mount riding separately; explorer-world-chest interaction remains open.

Evidence: [[Research/Mystery Chest Restoration]] and private `mystery-chest-intake-20261009T192906Z/receipt.json`.

## Delegated readiness audit, October9

- [x] Reconcile all100 assignments and independently check93 later-piece artifact paths; correct stale registration/acceptance claims.
- [x] Re-run/review focused Housing, Kart/Derby, Fishing and Soccer receipts; reproduce missing housing routing/snapshot bindings.
- [x] Test native Mining entry/progress/loss and ChefHarvest entry/render/developer cancellation; finalize three captures and prove whole-gameplay cancellation restart equality.
- [x] Return concrete findings to original workers/pieces and preserve [[Reviews/Delegated Native Acceptance Audit]].
- [ ] Mining: recorded reproducible gesture, full quotas/win, ordinaryQuit/resultsClose, outputs and restart.
- [ ] Harvest: goal names/icons, recorded gesture, ordinaryQuit/resultsClose, full round and once-only outputs.
- [ ] Housing:127 route, lot/fixture content, owned inventory instances and canonical atomic persistence before nativeplace/move/remove/visit/restart.
- [ ] Racing/Derby: real vehicle/scene entry, controls/checkpoints, validated result, priced rewards and save; derby scoring stillunrecovered.
- [ ] Fishing:138 route/register/schools/bobber/bite/reel authority; close result replay/Idle gaps and confirm version-specific layouts.
- [ ] Soccer: connect match roster/tick/ball/result/return; corrected codec/private adapter tests do not establish nativeplay.

## Five-worker repair round

- [x] Apply the owner's new five-worker maximum in configuration and project instructions.
- [x] Dispatch five DeepSeek Flash/max implementation workers; verify all five running once.
- [x] Register existing Fishing/Housing/Kart core modules; library gate24passed/0failed.
- [ ] Review Type8 goal/exit repairs and run native acceptance.
- [ ] Review/integrate Housing service into one atomic gameplay/inventory save.
- [ ] Review/integrate recovered Kart entry before vehicle/control tests.
- [ ] Review/integrate bounded Fishing authority before catch tests.
- [ ] Review/integrate Soccer lifecycle and native wire outputs before match tests.

Evidence and ownership: [[Reviews/Five Worker Repair Round]]. At most five active children, no descendants; stop/review each result before replacement.

## Kart finishing acceptance, October9
- [x] Integrate scoped kart entry, countdown/checkpoint/lap authority and canonical reward/save transaction.
- [x] Register reconstructed track31 price50coins/25RacerXP; verify once-only award, bank limits, exact toast, rank-up and disk reload in private gateway tests.
- [x] Recover16track chunks/40dependencies and original35/20 racing activation/return packets.
- [x] Observe native33/9, Loaded, enabled Go, textured kart and correctly timed countdown after activation repair.
- [x] Replace narrow captured-path spheres with opt-in reconstructed lane bands; validate alternate-line three-lap reward/save/replay flow.
- [x] Implement owned C33/22 boost requests and empty S33/23 responses with persisted refill epochs; independently review and verify three real native acknowledgements/HUD rockets in finalized v12.
- [x] Verify repaired caret-containing kart decal delivery in the original client; six Python delivery regressions pass.
- [x] Verify native timeout returns to the world without coins/XP in finalized v11.
- [x] Verify actual interrupted-race reconnect cancels the activity, restores its saved pre-race pose, and preserves inventory/XP in live restart000146; final capture pending.
- [ ] Replace developer-only `realm race` entry with a normal NPC/menu binding.
- [x] Verify native driving and checkpoint trigger emission: the owner's handling/jump test and97 captured checkpoint claims.
- [x] Implement native `33/19` use -> `33/20` type5; pass owned queue/refusal/gateway rollback/economy regressions. Fresh native activation remains pending.
- [x] Replace failed bands with recovered authored curtains/branch aliases and bounded movement credit; lead97-claim Rust replay accepts93 and finishes3laps.
- [x] Verify fresh native Ctrl boost activation/flames/forward movement and first checkpoint progress in diagnostic020033; inventory/XP unchanged.
- [x] Verify isolated native Alt and Ctrl activation on acceptedv14: two request/reply pairs,448 boost poses in clean022150.
- [x] Verify three-lap native completion: human success report and captured0/1/2/3 progress with3-lap limit on canonicalv14. Individual HUD screenshots are not claimed.
- [x] Repair/refinalize refill timing: remove guessed20-second pacing, enforce owned race window and3-slot queue; clean022150 verifies four acknowledgements, first3.281s afterGo.
- [ ] Restore native mystery-box collision, pickup acknowledgement and item-use effects.
- [x] Integrate pickup spawn/collection/use/consumed-save transaction; seven private kart gateway checks pass, including captured-route once-only payout. Diagnostic native020033 proves one collision/grant/removal/use; retail placement and types1..4 remain open.
- [x] Drive native three-lap race, verify50+25 bundle/bank deltas, native26/42 UI return and earned rewards after native reload032648. Alternative39/7 score-screen path remains covered in private gateway tests only.
- [x] Finalize human030742:27,915 frame hashes/27,896 gateway CRCs, normal exit0 and services stopped. Reload032648:5,310 hashes/5,290 CRCs, allinventory/XP preserved; new unfinished race retained separately.
- [x] Prepare reconstructed15-box layout and native-test bounded same-handle8-second regeneration in disposable034125; four returns/unchangedpaid banks, finalized hash/CRC proof. Canonical full-race density tuning still awaits a new human drive.
- [x] Correct shield expiry: raw PE proves S21value4; finalized035338 verifies visible effect/controlbit1 clears after5seconds,278on/108off poses, unchanged economy.
- [x] Repair/native-test mixed toolbar consistency: original state9 resets the SWF toolbar; defer actors until after state9, preserve Go clock/FIFO. Final040838 visible queue[4,4,5] and three Alt uses4/4/5 match shield/shield/boost effects; normal exit0, paid economy unchanged.
- [ ] Implement/native-test actual missile and mine actors/effects; typed queue/layout/asset foundation is verified but does not satisfy gameplay.
- [x] Repair CRC-verified direct pose transport and quiet session-cached placement; test-first decoder/Session/45-second idle regressions, frozenv20 full1733/0/83 and private checks pass.
- [x] Accept fresh v20 native firing: left-click rocket, Ctrl mine after30.437s quiet pose; wire actor lifecycles,28,619frame hashes/28,600CRCs, unchanged economy and normal exit verified.
- [ ] Complete mine arming/placement visual acceptance after owner movement, explosion arguments and actual weapon hits/knockback with a live rival.
Evidence: [[Reviews/Kart Playable Integration]].
