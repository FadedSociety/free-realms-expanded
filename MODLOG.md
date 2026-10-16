# Project journal

## 2026-10-15 — Studio toolkit: terrain heights, map, marks, NPC placer, bot arena

All nine Studio tabs were verified against the live game. The bot client now
re-sends ready and loaded after a zone change (31/28), so it can enter battle
arenas; covered by the regression test `bot_zones_into_battle_arena_and_wins`.

GCNK terrain vertices are world coordinates. Against 3,264 Legacy spawns, the
new `osfr-terrain` 2 m grid has a median height error of 0.05 m. Build it with
`scripts/build_terrain.py`, which writes `lab/terrain/zone_<id>.hgt` (private);
the gateway loads it with `--terrain`.

Enemy movement now follows the ground. Snapping is skipped beyond 6 m, so
enemies keep their height on bridges. The tutorial cave's flat y = 0
placeholder terrain is refused; its floor is model geometry.

New developer commands: `realm goto x z`, named marks (`data/content/marks.json`)
and the NPC placer (`realm place` and `unplace`, `data/content/npc_placements.json`).
The new Studio Map tab draws the relief; click to teleport or to place NPCs.

Behaviour presets (`data/content/behaviours.json`: melee, skittish,
stationary) replace the hard-coded enemy constants. An enemy record chooses
one with `"behaviour"`, and unknown presets or fields are refused. The
piece-054 reach guard now checks the presets instead. The full suite passed:
2060 tests, 0 failures. A native client landing check is pending. Design and status: [[Design/Studio Toolkit]].

## 2026-10-14: Built world entry, accounts, NPCs, quest engine native

Reconstructed from the 2026-10-15 audit; details are in [[Reviews/Native Round5 050310]].

- **Self data:** first native run with all 46 startup messages built (SelfData 011719).
- **Accounts:** account store and login handoff key work natively, single account (Login Accounts 023623).
- **NPCs:** regional NPCs spawn upright and interactable (Round4 035300). Dialogue closes, and movement validation passes (Round5 050310).
- **Quest offers:**
  - From 05:38 to 14:14 we hunted the missing quest offer. The Lua probe showed the client dropped it in native code.
  - Frida then found 6 extra bytes in the 49/1 encoder, and the original offer window appeared.
  - "Introduce Yourself" now works end to end, followed by the 3-quest Cobblestone chain and the dialog camera.
- **Native map:** the dispatcher stopped on a rate limit.
- **Disk:** C: filled to 0 GB; build caches and old client copies were cleared.

## 2026-10-13: Luna build wave W01–W62

- **Workers:** GPT-6-Luna builders produced startup builders, inventory fixes, accounts, the content model, the player registry, chat, social, the quest engine, job and quest content, NPC data, minigame lifecycles and more ([[Design/Build Ledger]], 10-15 audit).
- **Power loss:** seven workers died in a power loss at 21:36Z and were all resumed.
- **Native:** startup builders verified natively (Startup Builders 215401), with 45 of 46 messages built and the inventory search no longer crashing on "hammer".
- **Multi-session gateway:** W35b.

## 2026-10-10 — Native function map pipeline and complete Ghidra analysis

Claude Code is now lead (Codex weekly limit reached). Fresh import of working
FreeRealms.exe (SHA-256 26CC1C52…3351) into `lab/decompiled/ghidra-full-20261010b`
with 16G heap and no timeout: auto-analysis completed in759s ("Analysis
succeeded"). Same153,423 internal functions at identical addresses as the earlier
900s-capped project, so the old partial analysis had no boundary differences.
`ExportNativeMap.java` exported every function (0 decompile failures, 947s) with
callers/callees, strings, vtable addresses, data refs and pseudocode.
`scripts/native_map/` builds the map: exact rule labels (52,802 EH funclets,
4,484 constant stubs, 876 thunks), 805 shards for DeepSeek mappers, a validator
that machine-checks every cited anchor, independent reviewer sampling, and vault
rendering to [[Native Map/Native Map]]. Five-shard pilot on the old export: first
two shards accepted with 0% and 3% review disputes; all 354 accepted pilot labels
re-validate against the new export. Names are evidence-backed proposals, not
original symbols; server behaviour remains labelled hypothesis.

## 2026-10-10 — Rocket direct transport and idle placement correction

Reproduced missing direct C33/6 dispatch with a private Session regression; strict
CRC/bounded decompression now forwards only owned session poses. Native v19
062127 proves rocket activation and audited actor lifecycle, but exposes the
invented one-second placement expiry during quiet traffic. Removed that expiry
for session-cached visual placement; future/invalid/spoofed/implausible poses and
race-window violations still refuse. Quiet45-second rocket/mine regression is
red before and green after. Frozen v20d481865c: whole1733/0/83, private kart9/0,
Session1/0. Fresh native064034 verifies left-click rocket activation and Ctrl
mine use after30.437seconds without a new pose. Independent wire lifecycle:
rocket37updates/136.220m/4s, mine110updates/12s; all28,619frame hashes and
28,600gateway CRCs verify. Inventory/XP/coins unchanged, normal menu exitcode0,
all services stopped. Mine visuals after owner movement, explosions and hits
remain open.
[[Reviews/Kart Mines and Rockets Acceptance]] preserves logs, captures and limits.

## 2026-10-10 — Additional reverse-engineering tools and local Laya

Installed pinned REA6.3.0/official release-aligned Codex skill and registered its
MCP with existing Ghidra12.1.4/JDK21; scoped readiness checks pass, live MCP needs
reconnection. Installed pinned Ablation2.49.0/isolated environment/local adapter;
actual PE32 smoke output is invalid due architecture/address mapping assumptions.
Upstream GPLv3 LICENSE conflicts with MIT packaging metadata; no source reused.
Installed Laya0.4.1 and hashed pinned English model locally. CPU inference works,
but repeated prompts score4/8,3/8,3/8 on the same small reward-handler diagnostic;
not accepted as reliable function filtering. [[Research/REA Ablation and Local Laya]]
records versions, paths, logs, limits and measured timings. No speedup claim.

Preserved failed v18 native weapons input:17use requests/no activation, direct
poses never reach the gameplay dispatcher. Pose guard leaves rocket queued;
narrow direct C33/6 transport repair/new frozen native acceptance were pending
at that capture and are now resolved by the v20 milestone above.
[[Reviews/Kart Mines and Rockets Acceptance]] records the concrete cause and the
retracted unsupported left-click binding claim.

## 2026-10-09 — Human native three-lap kart completion

the owner reports successful race completion and overworld return on canonical v14. Lead capture audit independently verifies completed-lap transitions0/1/2/3, one50coin/25XP bundle, native26/42 UI exit and FabledRealms scene return. Saved coins309→359, Kart DriverXP0→25, unchangeditems/otherXP, finished3laps/rewardflags and activeNone. Finalized27,915frame hashes/27,896gateway CRCs, normal exit0 and recorded PIDs absent. Native032648reload preserves all paid banks/items/XP; its later unfinished race remains separately preserved. Twenty canonical boxes collected. See [[Reviews/Kart Human Completion]].

User box feedback now has native v17 diagnostic acceptance: reconstructed15-position layout/8s timer, shields alongside boosts, corrected S21value4 expiry and matching mixed FIFO toolbar. Original JPEXS/Lua/native evidence shows state9 resets toolbar icons; Rust now announces actors afterward while preserving Go clock. Whole1713/0/82, nine privatechecks and80focusedchecks pass; frozen release80ec131a. Final040838 proves15initial actors/five8s returns/two shield expiries/Alt4-4-5 matching visible heads,108boost poses and97.4m motionwithin3s.15,715hashes/15,694CRCs verified, zero unchecked; normal menu exit0/PIDs absent/paid economy unchanged. Failed034125 and mismatched035338 remain preserved. Canonical15-position full-race tuning, NPC/menu admission, random drop odds and missile/mine world effects remain open. [[Reviews/Kart Pickup Respawn Acceptance]] records exact evidence and gaps.

## 2026-10-09 — Clean native v14 boost/refill acceptance

Subsequent bounded research completed and children were interrupted. Nonboost correction/checker independently passes9frames/160counted subtypes/84citations in5.7seconds. Typed Rust restore, projectile/trap authority and shield expiry/lifetime are required before porting nonboost drops. NPC method/caller corrections accepted; proposed16-byte entity classification rejected after lead hash-matched two originals to35/20 player-state packets. Canonical023328 is open atGo with an unstarted timer and live capture; whole native finish still pending. Detailed research reviews are linked from [[Reviews/Kart Native Boost Acceptance]].

Original-client022150 on frozenf9f4af0b verifies separate Alt/Ctrl use/activation exchanges,448 boost poses, four refill acknowledgements, one diagnostic pickup grant/removal, visible purple pickup actors and one authored checkpoint. One Go click suffices; an additional transient-panel click confounded the prior021651 activation, now explicitly excluded from keybind evidence. Final43,427 hashes/43,406 gateway CRCs/38,174 direct poses, zero unchecked; normal menu exit0, recorded service PIDs absent. Coins309 and all inventory/XP unchanged. Accepted source/log fingerprints still match. Read-only capture audit now correctly handles14 compressed direct poses; no runtime source change. Full native three-lap HUD/finish/reward/Close/reload and retail-position collection remain pending. Exact proof and checklist: [[Reviews/Kart Native Boost Acceptance]]. The supplied2009 video stays a historical visual reference. One read-only DeepSeek worker investigates nonboost effects.

## 2026-10-09 — Config-driven kart refill correction accepted in tests, v14

Native code/captures disprove a fixed20-second refill floor. Refill now accepts an owned in-window request if the recovered3-slot queue has room. Legacy epoch stays only for snapshot shape compatibility; reliable transport deduplicates retransmissions, but distinct identical application requests can fill free slots. Client configuration drives energy; server-side energy simulation remains future work. A shared pure lower/upper race-window guard covers refill/use/pickup; pre-Go pickup preserves the existing observation-counter behavior.

Lead's actual nativeC17/C22/C19 regression now passes; two old fixed-epoch private oracles were corrected to assert3 grants, fourth refusal unchanged, exactly3 uses then empty refusal. Full Rust1694passed/0failed/81ignored, private8passed, focused91passed/0failed/2ignored. Release SHA `f9f4af0b7d8278f9af8849d69082f69440963b44251ce3c15c803cfbef0d25b2`; source/log/content fingerprints in `../lab/reports/kart-playable-v14-20261009/build-receipt.json`. Asset code unchanged, so prior6 Python/4route proofs are reused and explicitly identified. Fresh native600-second refill/Alt probe launched; no full native completion claim. One read-only worker examines original items1..4 separately.

## 2026-10-09 — Native kart pickup and Ctrl boost verified; refill timing defect isolated

Original client020033 accepts one diagnostic grid-adjacent pickup:37 spawns,7 collision claims, exactlyone type5 grant/removal. Ctrl produces exact use/response, visible flame exhaust and acceleration;224 native boost-control poses and one accepted authored checkpoint independently captured. Inventory/XP unchanged, coins309. Finalized8,027 frame hashes/8,007 gateway CRCs/5,670 direct poses, all recorded services stopped at300-second harness bound. Receipt in `../lab/reports/kart-native-pickup-probe-20261009/`. This does not prove retail pickup placement, Alt, three-lap HUD, finish/payment or normal Close/restart.

New refill failure: nativeC22 at10.603seconds afterGo was refused by reconstructed20-second epoch, with no further retry before shutdown. Returned a scoped timing investigation/repair to the existing DeepSeek item worker. User asked asynchronously for a held-key three-lap drive; the native UI tool supports taps/chords only.

## 2026-10-09 — Kart curtains, boost use and pickup integration accepted in tests

Optional authored CDTA curtains and branch aliases now accept93 of the owner's97 actual claims and complete3laps with the unchanged ordered cursor. Movement uses bounded elapsed allowance plus saved credit; original surfaces have reconstructed3m thickness. Strict validator review fixed folded/concave/sliver acceptance. C33/19 now consumes one queued boost and replies exact S33/20 type5. Pickup lifecycle emits exact S15 once, validates C17 against an owned live actor and shared movement budget, grantsS18/destroysS16 once, preserves full-queue actors and saves consumed handles. All37 captured spawn positions are in canonicalv13; all drops are reconstructed type5, no random odds/other effects/respawns.

Lead seven private end-to-end checks pass, including actual97-claim gateway completion/payment, pickup grant/use/rollback/restore, alternate10m lane, once-only50coins/25job48XP, timeout and reconnect. Full Rust1692passed/0failed/80ignored, focused89passed/0failed/2ignored, six Python delivery checks and four recovered-asset route proofs pass. Alternate-lane test required correcting its generator to choose a horizontal ring edge and move inward; production geometry was not widened. Logs and build receipt are in `../lab/reports/kart-playable-v13-20261009/`. Release/native verification remains pending at this checkpoint. Separate grid-adjacent pickup probe is explicitly diagnostic and preserves canonical positions/campaign save.

## 2026-10-09 — Kart native driving failures and historical video

the owner's finalized human run005751 verifies43,393 frame hashes and43,374 gateway CRCs, including35,186 direct driving updates,97 checkpoint claims,17 use requests and3 refill acknowledgements. Handling/jump worked; laps, boost activation and mystery boxes failed. Services stopped normally. Evidence: `../lab/reports/kart-human-driving-20261009/capture-receipt.json` and [[Reviews/Kart Native Driving Failures]].

Installed `bradautomates/claude-video` watch skill, FFmpeg and isolated local WhisperX3.8.6; readiness and model warm-up pass. Reviewed20 sampled frames and5 cue frames from supplied May21,2009 Tangetrack reference; five-lap historical UI differs from the private later-client run. WhisperX detects no speech. Video media/frame/setup evidence stays in the private lab.

Recovered C19 owned-u64 use and S20 u32 type5 activation. Worker implementation mirrors/persists the three-slot queue; lead gateway RED proves missing reply, then all4 prior private gameplay checks pass including the new activation/rollback assertions. New actual97-claim Rust replay remains RED on estimated bands. Correct CDTA parsing recovers36 authored physical curtains; all human hits lie within2.57m of them. Ordered31-step route has3 paired branch alternatives. Rejected the nearest-waypoint audit's proposed lap-cursor skip: authored curtain0 refires every lap. Optional authored-volume integration and bounded buffered-claim handling are in progress.

Recovered S15/S16/C17/S18 mystery-box layouts with50 creates/13 destroys/33 claims/4 responses and byte-exact roundtrips. Model497 resolves to original random-pickup actor. Pickup implementation is separately scoped; no native-completion claim. At most3 DeepSeek Flash/max workers ran concurrently and finished children were stopped. Campaign save and original archives remain preserved.

## 2026-10-08 — Workspace setup

- Request: install the requested skill repositories and Obsidian, then prepare for a game decompilation project.
- Reference games: FusionFall, Toontown Online, and Free Realms.
- Created this local Git project, an Obsidian vault under `notes`, and a separate analysis lab at `../lab`.
- Installed the reverse-skill Codex adapter and retained the full package at `%USERPROFILE%\.codex\tools\reverse-skill`.
- Installed Ponytail, i-have-adhd, and Caveman skills and companion commands.
- Installed RTK 0.50.0 and registered its Codex integration. A Codex restart is needed to load the new hook configuration; hook trust is managed by Codex.
- Installed Obsidian 1.14.4.

The intake and stack decisions below supersede this initial setup state. The setup entry retained its original recorded date; the recovery session uses local October 7 time.

## 2026-10-07 — Free Realms intake and Rust foundation

- the owner supplied OSFR.Server.zip and OSFR.Client.zip and selected a faithful Free Realms revival, with Rust preferred for reimplementation.
- Preserved both ZIPs, verified their SHA-256 hashes, safely extracted them, and hashed every extracted file. Inputs and full manifests remain outside Git in `../lab`.
- Identified the managed .NET server and native x86 Windows client. The main executable is 25,067,520 bytes; 24 packs contain the main asset corpus.
- Pinned the archived OpenSourceFreeRealms reference and the newer Sanctuary repository. Kept them separate from the supplied binary server.
- Installed private .NET 10.0.401, ILSpyCmd 11.1.0.9782, Rust 1.99.0, Ghidra 12.1.4, and JDK 21.0.12.1+1 toolchains.
- Recovered seven managed assemblies to 769 C# files, including dependencies; all ILSpy runs exited successfully. Recovered projects have not been rebuilt.
- Wrote original Rust pack tools, independently matched all 24 pack inventories, and extracted 108,547 entries totaling 3,020,467,857 bytes with every CRC verified. Preserved four repeated names as separate entries.
- Built a Rust one-shot SOE session probe. Synthetic fixtures generated with the supplied serializer match the Rust response. Eight unit tests and one actual loopback UDP integration test passed. Both release binaries built successfully.
- Started a temporary copy of the supplied server on a test port, verified its actual session response matches the fixture, and stopped it. This was not an original-client gameplay test.
- Completed the first native export: 153,423 inferred function bodies, zero attempted export failures, 95,176,801 bytes of pseudocode. Initial analysis reached the 900-second limit and remains partial. Verified index counts, hashed artifacts, and reopened the saved Ghidra project read-only; its 545 external function records explain the difference from the total count of 153,968.
- Updated the Obsidian vault and project instructions around the restoration target and a staged Rust roadmap.

**Next playable milestone:** actual client login to a local Rust service, one zone, movement, and saved character restoration. Reliable transport and application login remain unimplemented; this is currently a recovery/tooling foundation, not a playable Rust game.

## 2026-10-07 — Community tools, verified transport, and actual client baseline

- Reviewed Universal Modder's eleven skills and applied reconnaissance, reverse-engineering, community/archive research, and actual-game automation. Installed its private CLI and Windows/GPU capture tools; rejected the scanner's false ASI-loader label caused by Miles audio codecs.
- Corrected the earlier omission of community research before the custom pack reader. Searched XeNTaX's public backup and ZenHAX/QuickBMS material, pinned applicable community tools, and added the requirement to project instructions.
- Ran the unmodified ForgeLight Toolbox/DbgPack extractor on Assets_008.pack: all 6,074 entries/45,765,884 bytes agree with Rust metadata, SHA-256, and CRC.
- Ran FR ADR Toolkit readers on supplied Chatdy/warpstone assets; inspected a static textured Chatdy preview (4,256 vertices, 6,317 triangles, 92 bones). Animation and a general Rust importer remain unverified.
- Added bounded Rust CRC/compression framing and reliable receive, tested against 105 compiled serializer vectors and repeated sequence rollover. A focused review found and fixed an oversized-future-DATA acknowledgement issue; buffered semantic rejection now poisons the receiver.
- Captured supplied-server login/ready replies with synthetic requests. Rust replay verified 1,305 input hashes, decoded 1,253 frames, reassembled 46 messages/627,566 bytes exactly, and rejected every CRC-mutated frame.
- Passed 19 ordinary Rust tests, explicit compiled .NET decoder/capture oracles, formatting, and release build. Reliable sending/retransmission, persistent sockets, and Rust gateway/gameplay remain.
- Diagnosed the actual client's missing x86 d3dx9_31.dll from executable imports and the pinned community launcher. Downloaded Microsoft's official signed redistributable, verified signatures/hashes, and added only that DLL to disposable baseline copies.
- Followed the community release's manifest-hosting instructions, serving the supplied manifest's own CRC/size on loopback. Confirmed the client's runtime version, graphical startup, and actual three-byte CRC negotiation.
- Corrected relay receive loss with an 8-MiB buffer and deferred disk writes. The real client reached World Ready and Initialized-Client. After the owner explicitly authorized input while active at the PC, dismissed the news panel and held W for 1.2 seconds. Screenshots changed and the emulator logged changed character positions.
- Baseline captures, original configurations, raw packets, screenshots, private tools, and decompiled/game material remain in the lab. Test processes are bounded and stopped after each run. The baseline is the supplied walking emulator, not a complete restored game or a Rust server.

Next: implement verified reliable sending/control handling and gateway initialization in Rust against this actual-client reference, then add saved character restoration before expanding gameplay coverage.

## 2026-10-07 — First persistent Rust gateway verified

- Implemented bounded reliable sending, selective/cumulative ACK handling, retries, cooked control bundles, persistent loopback sessions, and clock reflection. Unit checks cover loss, reordering, duplicates, wire limits, repeated rollover, retry exhaustion, and queue bounds.
- Implemented validated actual-client login/bundles, ready/loaded messages, movement and time replies. Forty-six private, hash-checked captured emulator messages provide initialization; only proven self/zone fields are patched. Other GUIDs/zones are rejected.
- Added versioned character snapshots with same-directory replacement, validation, and corruption preservation. No C# server is needed for the Rust runs.
- Explicitly replayed the actual-client capture: 44 reliable requests, 46 byte-identical initialization replies, seven dynamic time replies, one world-entry signal. Full suite: 43 ordinary tests pass; three private-input oracles are explicit opt-ins. Formatting/release build pass.
- Ran the actual client against Rust in `client-baseline-20261008T022638Z`. It reached World Ready despite deliberately dropping server sequence 6. Independent analysis verified 1,479 frame hashes, 1,477 post-session CRCs, byte-identical retransmission, and subsequent cumulative ACK. Actual selective ACKs independently confirm opcode 17 semantics.
- With existing input authorization, dismissed the news modal and held W for 900 ms. Four position messages and three saves show 7.334 units of movement; the final save is approximately (-1412.029541, -28.219860, 344.728607).
- Started a fresh Rust process and client in `client-baseline-20261008T023143Z`. Saved position/rotation restore exactly as f32 values, the client's World Ready coordinates agree, and the restored world screenshot was inspected. Both bounded runs stopped their exact process handles.
- Final binary passed seven real-UDP session checks, including rejected replacement preservation and stale-seed rejection. Aggregate evidence: `../lab/manifests/rust-gateway-milestone.json`; details and commands: `notes/Research/Rust Gateway Milestone.md`.

Next: one evidenced NPC interaction/quest with an objective, inventory/reward update, and restart persistence. Captured initialization is still a dependency; this milestone does not establish complete gameplay restoration.

## 2026-10-07 — Playable Rust gameplay and persistent rewards

- Added typed inventory/equipment, quest progress, local enemy combat, checkpoint activity, native chat commands and unified version-2 snapshots. Domain bundles stage on clones; reward flags/items/wallet persist before live commit or reward replies. Added invalid-command feedback and rollback/reward-once checks.
- Followed community-first research: pinned FreeRealms-Legacy with AGPL provenance, corroborated existing serializers, ran MIT FreeRealmsLocaleTools against preserved locale files, and analyzed archived original minigame captures. No new asset-format parser was introduced.
- Played the native client against Rust through guide interaction, chat quest acceptance and three Robgoblin defeats across consecutive persisted runs. The final defeat in run032359 completed reconstructed quest900001, granted 25 coins and increased original item1195/instance1077 from 9 to 10. Original quest1617 does not count these unrelated enemies.
- Equipped Small Stitched Pants through the native C screen; request42/3[2,1,4], response, changed model and saved equipment agree. Completed the reconstructed ordered checkpoint course with real keyboard movement in 18,456 ms for 10 more coins; an earlier timeout granted nothing.
- Added community item category references44/2, exactly 2,675 bytes matching the unmodified C# serializer. Native Stuff now populates. Fresh run033628 restored wallet35, Wheeler on the Cob quantity10 and pants; all gameplay state is exactly preserved. A separate run033402 inventory-search test crashed the native client with 0xC0000005 and is explicitly excluded from clean acceptance.
- Independently verified gameplay run: 4,125 hashes/4,123 CRCs; restart run: 2,696 hashes/2,694 CRCs. Actual restored self and emitted category bytes match. Both accepted runs were stopped deliberately by exact client PID then harness-owned cleanup. Original archives remain unchanged.
- Final verification: 73 ordinary tests pass, eight private-input opt-ins; new private inventory/quest/combat/complete-loop/category checks explicitly pass; formatting/release and seven final-binary UDP checks pass. Final release SHA-256: 949f89a53f2dc644d3b58792d6661d9c12e4ae062beb945e8c8f3c0648513b72.
- Saved [gameplay milestone](notes/Research/Gameplay%20Milestone.md), coverage ledger, wire proofs, restart comparison, native search-crash research and aggregate `../lab/manifests/rust-gameplay-milestone.json`.

Next: diagnose inventory search, recover an original quest/enemy mapping, and implement original soccer through match completion/rewards. The present quest, combat balance and checkpoint rules are explicit reconstructions; native original abilities/AI, broad content and a replacement Rust client remain ahead.

## 2026-10-08 — Opening playthrough started

the owner requested sustained implementation through character creation, the starting area and a full dungeon before handoff, with progress/reviews in Obsidian. Created `notes/Design/Opening Playthrough.md` with native acceptance gates. Delegated original starting-content, character-creation and dungeon/class audits; preserving the existing research character and milestone saves. No completion or original-content coverage claim yet.

- Recovered community-verified creator terrain and starting-outfit resources into disposable runs; inspected the native character/naming UI. Diagnosed the subsequent submit spinner from its actual captured server0 request and corrected the sole-local-realm route with capture replay regression coverage. Native submission retest is pending.
- Implemented opening class/inventory/world/encounter integration and scene-aware persistence. Private packet simulation completes all17 locally bound opening steps, the cave, return and repeated reloads. Independent review corrected prerequisite enemy gating and missing starter definitions/slots. This does not satisfy the actual-client acceptance gate.
- Completed requested Scrapling/wiki/sqlite-vec research database with3,250 pages/3,996 indexed text/vector chunks, source/revision/hash provenance and citation-bearing search proofs. Wiki-proven five-wolf requirement and16/18 coin rewards are applied; unrecovered crafting/quest details remain labeled reconstructions.

- Actual native run072258 creates a second supported-appearance test identity and reaches World Ready/Initialized-Client. Bennett's failed unsupported texture08 identity/save is preserved. Verified community SkinTone/FaceType tables remove unavailable7/8 choices; appearance1..6 match original manifest CRCs. Fresh inventory and native Brawler switching work; firstdiscovery reward saves. Full quest/dungeon playthrough still pending.
- Added original native asset envelope delivery with rawCRC/proof validation and verified source paths. Independent review passed envelope/path/cache probes and found a daemon-worker metrics cleanup race; protected updates/snapshots with a lock for future runs.

- Native firstopeningquest completed/turned in and originalcavescene reached inrun073831. User'scombat feedback is explicitly failedacceptance:stationary/noanimation/longapparentreach; allthreeGloamling defeats saved,thenlosswithboss500HP. InvestigatednativeNPCmovement/heading/animation; patchedspeed/state/animations/pursuit/reach, fixedreviewedf32/aggro regressions. Focusedprivatepacketoraclesand17stepsimulationpass.
- Recovered exactmissingbossADR withoriginalrawCRC/wirelength, noappearancefallback. Rebuiltrelease498fb4ee0a98c081f710d10e139117576a0344112be4ded974821b1999787ee6 andstartednativecombat-retestrun075754. OriginalcraftingminigameGFx/AS2research progressingindependently; nofullnativecompletionclaim.

## 2026-10-08 — Cave combat repaired and completed

- Native footage and independent packet/asset checks confirm actual enemy pursuit and attack poses after the motion fixes. Added a connection-local preparation gate so loading/rejoining/retrying does not cause immediate damage; Darkthorne's Start or a valid first attack arms combat. Tight melee range and cooldowns remain server-validated.
- Actual campaign run081219 defeats all three Gloamlings and the boss, survives with 280 HP, grants 50 coins and one food item once, and returns through Darkthorne's Exit to zone0 at full health. The saved wallet is70 and The Last Best Hope objective counters are [1,0]. Full serializer, transport, footage and reward/return proofs are in `notes/Reviews/Native Cave Completion 081219.md`. This accepts the reconstructed cave fight; it does not complete the full starting-area request.
- Corrected native encounter MiniGameType1 (sample) to4 (combat), preserving the separate ZoneContext8. Corrected basic request handling to bar1/slot0/self GUID1, as proven by native code and actual capture. The supported basic resolves one live enemy within server reach; original multi-target sweep effects remain unrecovered. A separate, explicitly synthetic controls checkpoint in run083946 completes the fight using normal key1 with 660 HP. Its reward flag stays set and grants nothing again. Final wire/footage audit is pending.
- Integrated original Smelting77 movie/protocol with optional compatible snapshot state and atomic material consumption, bar grants and quest progress. Five valid counted Miner harvests temporarily grant local ore/coal drops; original drop balancing is not recovered. Private tests prove rollback, invalid entry, no farming after exhausted goals, duplicate-win safety, close/reconnect cancellation and all17 opening steps through the native packet lifecycle. Actual furnace play remains pending.
- Ordinary Rust suite, focused new combat tests, private compiled packet oracles, the17-step integration, formatting and release build pass. Current release SHA-256: c2380ee2434ccbfcc15176f14a72cb35062aa06dbea1a36e7103491112480950. Campaign and controls-test saves remain separate; original archives and existing identities are preserved.

## 2026-10-08 — Farm transition and dungeon cleanup

- Independent run083946 audit confirms normal key1 combat: 24 native requests, 11 processed player hits, all four enemies defeated and 660 HP remaining. This separate controls branch grants no duplicate reward; the campaign save remains preserved.
- Fixed encounter lifecycle messages to use native StateID411. The original client now invokes OnGameOver and LeaveMiniGame. Separate native win run091836 returns to the farm and opens the correct quest journal in the same process. The results modal still needs investigation; its absence is not accepted as intended behavior.
- Original locale evidence establishes Darkthorne's magic door to Farnum's Farmstead. Native campaign run090058 reaches the farm, earns its discovery reward, talks to Pappy, completes The Last Best Hope and accepts Saving Farnum's Farm. Wallet progresses 70 to75 to100; farm objective counts remain zero. The chosen arrival offset is reconstructed.
- Native farm testing found missing well/fire dependencies. Community readers and original manifest CRC checks identify complete alternate well3519 and fire204 assets. A new private content version incorporates these disclosed visual substitutions, persistent extinguishing and original Smelting77/Forging139 integration. The old content and save are retained; migration and native farm/crafting acceptance remain pending.

## 2026-10-08 — Original Cooking integration prepared

- Integrated the original cooking movie/protocol as an optional compatible gameplay service. Game46 binding, six-phase order/tuning and fixed one-food output remain explicit reconstructions. Wiki recipe inputs are four each Bat Meat, Gummyfruit, Rainleaf and Spice; genuine definition35 is StrongArm Stew, while14794 remains the separate hat reward.
- New versioned campaign grants bounded local acceptance supplies and one leaf/fruit per newly counted Chef gather. Station entry/completion require accepted pending objective, Chef, proximity and materials. Inventory consumption, food grant, quest progress and minigame win stage/persist together. Deferred the duplicate inherited starter hat until the existing quest reward only in the new campaign.
- Prepared one combined new private Cooking/artifact variant: opening SHA-256 dfd39d7a893f8acc0687b64c81dda7ed93d0843aec51442b04c81dd23684baaa. It changes exactly four untouched future gather thresholds3→4, appends only complete food definition35 and applies the independently verified three alternate artifact1579/Y-only repairs. Current cef512 campaign files and user save remain unchanged; migration was not executed.
- Six ordinary isolated tests, old optional-state serde, one explicit private configuration, two private gameplay checks, six compiled community protocol vectors, five compiled Legacy item serializers and the full17 opening packet simulation pass. Synthetic migration preserves all state except the opening hash and rejects18 unsafe state/content transformations. Native cooking entry, mouse play, win/close/restart remain pending. Details: notes/Research/Original Cooking.md.

## 2026-10-08 — Original Gloam artifact recovered for Cooking campaign

- Independent asset research recovered all10 missing original model3934 dependencies with exact manifestCRC/native-envelope proof; complete closure has25 verified files. Preserved the earlier unplayed alternate1579 candidate and prepared a separate final Cooking variant using original3934, unchanged model/file/GUID/X/Z/action, and only the three approved terrainY alignments.
- Final private directory is lab/references/rust-opening-cooking-original-assets, openingSHA2563e6f4228efc76460b8b4ff5c9ecefb36264e2caf308da7d0cc7fc3ea8e34229e. Cooking economy/count changes and completion disappearance remain unchanged. Currentcef512 campaign and snapshots are untouched; no migration/native input/release build performed.
- Updated immutable proof/default paths, added rejection of artifact substitution, and reran all19 migration preservation/rejection checks plus Cooking gameplay and full17 opening packet flow successfully. Native artifact collision/reach and crafting mouse play remain pending. Aggregate proof: lab/reports/cooking-original-assets-integration-proof.json.

## 2026-10-08 15:42 UTC — Native farm complete; Cooking checkpoint migrated

- Native run151602 completed the well and all three fires, recorded each visible fire disappearance, walked back around the original logs/cliff, and turned Saving Farnum's Farm in to Pappy. Stage910003 is completed/rewarded; coins110; Cooking remains unaccepted. The run and stopped checkpoint are preserved.
- The constructor-level native journal trace proved that incremental objective keys must use definition IDs, although full quest records contain ordinal tuple keys. Patched only incremental49/9+10 and added distinguishing regression checks; full original-writer self serialization stays unchanged. Live acceptance remains pending.
- Combined formatting and ordinary Rust suite pass:138 tests,0 failed,27 explicit private tests ignored by the ordinary run. Focused private Cooking, compiled community vectors and17-step checks pass separately. Release gateway SHA256cd0dc6ea1f2a84ba3ef6ad9cec8043562a62f560b808b6926cddf8d44548ed77.
- After independent review and verified shutdown, migrated the farm checkpoint into NEW lab/state/opening-cooking-character.json. SourceSHA256ab8588534d8e589299dbaed5a8a5eb5b81f52303b6e68d9852a94c90d6e67adf; initial newSHA256c2ddbc0a0bc3138658ecdf6cc9c5fb2cbe221ca84ebade0af6a8f56cb9cce13d. Only opening content hash changed to3e6f4228efc76460b8b4ff5c9ecefb36264e2caf308da7d0cc7fc3ea8e34229e; no future quest accepted. Native results branches are separate from this campaign.

## 2026-10-08 — Optional readable opening objectives prepared

- Native run154503 proves live green progress but exposes locale ID0 placeholders and ambiguous combined ingredient labels. Audited all51 objectives;24 receive explicit reconstructed labels in a separate private presentation package, covering all14 missing-ID goals, split ingredients and numbered fires/artifacts.
- Used unchanged pinned MIT FreeRealmsLocaleTools reader/AddEntry/writer and strict readback. All113286 stock entries/85932 original IDs remain intact;28 new noncolliding strings added. The paired locale replacement targets only a NEW disposable client/locale directory before launch.
- Final package lab/references/opening-journal-text-v2 contains a byte-identical29-file content clone plus optional journal-text.json. Opening SHA remains3e6f4228efc76460b8b4ff5c9ecefb36264e2caf308da7d0cc7fc3ea8e34229e; no migration/save write. OpeningJournal substitutes only full/Add text IDs; absent mapping preserves the complete original serializer oracle and live keys remain definition IDs.
- Added strict opt-in harness checks and provenance for --journal-text and the separately reviewed --character-select-title v3 private movie repair. Positive paired copy,23 negative preflight cases, all51-goal checkpoint/text proof, compiled original full-self oracle and mapped17-stage/restart integration pass; independent readers/reviewer repeat the proof and25 rejection cases without findings. Formatting/Python compilation pass. No release build, native input or active-run modification. Native UI acceptance remains pending. Details: notes/Research/Opening Journal Reconstructed Labels.md.

## 2026-10-08 16:14 UTC — Native Chef preparation and crafting start diagnosis

- Native run154503 restored the migrated campaign with110 coins, accepted Cooking101 through Mae, unlocked/selected Chef45, equipped Student Chef Shirt13597, and gathered four Rainleaf plus four Gummyfruit through counted resource interactions. Final quest counts[1,1,1,4,4,0]; all four recipe materials quantity4; no food/hat award or consumption occurred.
- Immediate native journal shows the three completed preparation goals green without restarting. Independent wire review confirms definition-ID updates and ordered four-count ingredient completion. Original zero-ID custom goal labels remain visibly broken in this old run; reviewed locale/title overlays are pending native rendering.
- Cooking table opens the original Strong Arm Stew offer. Actual Go emits39/5(instance46000001,group-1,game-1); the service rejected the native sentinel. Shared native writer disassembly confirms the same request for all three crafting services. Payload39/14 also uses the native sentinel; compatibility repair is being checked with phase/instance binding and existing transaction guards retained.
- Preserved pre-game checkpoint opening-cooking-before-native-game.json SHA256f184cea3915606855347c436f477fa694c648189b032b0ddd59287b631ddd107. No native cooking round or completion is claimed by screenshots named movie/round1; those capture an offer/failed launch only.
- Before the sentinel repair, combined formatting/ordinary suite passes141 tests with28 explicitly private checks ignored. Optional journal/title packages and actual score-row lifecycle have independent focused reviews; next release/native acceptance remains pending.

## 2026-10-08 — Shared native crafting header compatibility verified

- Original RequestStartGame009be7d0/shared writer009bdc60 and SendPayload009be6b0/constructor009b09d0 use current instance, group-1 and game-1 for all three crafting movies. Original Lua method table and independent Capstone bytes prove SendMessage00c08480 adds1 to the string length before sending, including its NUL.
- Narrowly admit the native game sentinel for39/5 only while Offered and39/14 only after Offered, requiring exact active instance/group. Existing authored IDs, S2C serialization, bounded payload/schema/order/replay and atomic economy guards remain unchanged. Added six compiled unchanged community C2S vectors and literal captured-Go plus negative regressions for Cooking, Smelting and Forging.
- Focused gates pass24 ordinary crafting checks,3 private Cooking gameplay checks,6 private Forging/Smelting gameplay checks and the full17-stage/restart mapped opening simulation using actual native C2S headers. Each older compatibility/fire-only fixture uses its intended versioned reference; native phase/statistic acceptance remains pending.
- Proof: lab/reports/native-crafting-client-header-proof.json, SHA256ac0a3948bcded427c85b3e2ff3ae39ac9e94f5c1fe1c163b30ef9971ad4e79a1. Original StatUpdate/StopLevel FR_event quote tolerance remains explicitly a live gate; no strings were guessed or sanitized. This agent performed no release build, native input or campaign/content mutation.

## 2026-10-08 16:22 UTC — Native crafting headers and live trace prepared

- Original x86 writer/constructor proof establishes both client39/5 RequestStartGame and39/14 SendPayload use[current instance,-1,-1], with NUL-inclusive payload length. All three crafting services now accept those two native sentinel boundaries only for the matching instance/group and valid phase. Existing parsing, ordered statistics, replay and atomic recipe checks remain. Six compiled unchanged community vectors match, including captured native Go.
- Combined formatting and ordinary Rust suite pass144 tests,0 failures,28 explicitly private checks ignored. All24 focused crafting tests, appropriate private Cooking/Smelting/Forging gameplay checks, and mapped17-step/restart integration pass separately. New release gateway SHA2561fa19774b64812c79d3b2d9a6fc90e43cf6b1814874ad81615b0041922e8808b.
- Optional --live-capture publishes the same recorded UDP format from a bounded nonblocking background writer. The relay does no disk I/O; final capture remains authoritative. Isolated known-native replay matches4780 frames/4759 CRCs/2703 applications and checks incomplete prefixes, partial JSONL, queue overflow, disk failure and stopped-writer final overwrite. Instructions: notes/Research/Live Native Packet Capture.md.
- New native campaign run162238 uses the reviewed journal-text-v2 content/locale pair, title-v3 single-operand movie repair, actual-native crafting headers and live capture. Opening hash/identity/gameplay progress are unchanged by the presentation packages. Native rendering, original cooking play, and v3 combat results acceptance remain pending.

## 2026-10-08 — Native combat result pages and character labels accepted

- Independently audited finalized synthetic one-hit win163518 and loss163829:1876/1882 datagram hashes,1855/1863 gateway CRCs, complete reassembly, exact GameOver→two score rows→StateRemove→ACK→actual Close→world ordering. Native score pages show4/3 defeated and total0, remain open59.547/59.063s, and return only after actual X sends41/109[0,1]. Both same-session journals preserve completed defense and pending Pappy talk.
- Verified exact prepared-fixture/source/identity/content hashes and whitelist changes. No repeated dungeon reward or item/quest changes. Loss returns health1000/coins70; win's new Farnum discovery2 explains coins70→75, and its actual new kill adds25 Brawler experience. All harness services stopped and live writer final counts match authoritative manifests. Native retry remains untested in these v3 runs.
- Recorded visible limitations: boss target-health bar overlaps loss score title; win's original local Goal1/1 row displays5000 while server/total points remain0; both runs retain22 unique asset404 paths. These do not prevent the narrow lifecycle acceptance and are not silently accepted as complete UI/assets/scoring.
- Native title-v3 screenshot now shows saved-name/Add a Character rows instead of1/0, then Go restores the same identity. Stock row width clips the long name's tail; full creator-option coverage remains separate. Proof lab/reports/native-results-v3-independent-proof.json SHA256352ff0de04b5e3a92e7009759c895c3ce3cd975dc4a7cd1c870ef8a0b9529bdb. Review: notes/Reviews/Native Combat Results v3.md.

## 2026-10-08 — Loss-results boss display repair prepared

- Original Sanctuary32/9 writer, native parser/dispatcher/boss-registry functions and authoritative win/loss wires prove false unregisters the surviving boss display without killing/removing its actor. Original Lua already hides Target on minigame end; exact later re-show timing remains untraced. Preserved evidence: lab/reports/native-loss-boss-display-proof.json.
- Added only loss-only living-boss unregister packets in the once-only results path, before scores. Reversing this ten-line addition reconstructs the original evidenced encounter source exactly. Actor state, terminal rules, economy, Close, win and Retry logic remain unchanged.
- New distinguishing regression reproduced the missing false packet before the fix;15 ordinary encounter checks plus2 explicit private original-content/compiled-writer checks now pass, formatting clean. It covers exact bytes/order, dead/nonboss exclusion, full-state preservation, duplicate suppression, restored loss and Retry registration. Patch proof lab/reports/loss-boss-display-patch-proof.json SHA256fcdc026b9ea8a66f2f4cb7463b76c6f6813cc4681c8beb6cbf61d479b6092f70. Independent reviewer repeated17 checks and verified exact source/native/wire evidence without blocking findings (loss-boss-display-independent-review.json SHA256cb42317573d8cafe014ee09284cc03d4827b99bd07da6da56bdabd60d615674f). No release/native input/save edits; fresh native loss/Close/Retry acceptance remains pending.

## 2026-10-08 16:56 UTC — Six native Cooking rounds complete; finish transition pending

- Native162238 verifies the paired journal labels and character-name overlay, then exposes the exact literal closing parenthesis in original FRServer_StatUpdate values. Narrow Cooking/Smelting compatibility removes at most one terminal parenthesis only from the last StatUpdate/StopLevel field; original header, NUL, schema, finite/range/order/replay and atomic economy gates remain. Forging's unused forms remain rejected. Independent original-writer review,27 focused checks,9 private gameplay checks and full17-step mapped integration pass.
- Formatting and full ordinary suite pass147 tests/0failures/28explicitprivateignored. Release gateway SHA256393ae59c5f3192e859fc6bf3000ee3cf840495a5e34bcfe59deff4fd4e4fbff8.
- Native164218 plays all six original Cooking rounds with normal mouse input. Saved per-round stats are[34,8,0,44],[0,15,0,6],[1,2,0,21],[0,1,0,120],[0,3,0,63],[0,0,10,41]; final time297 and bad/good/perfect35/29/10 agree with the original movie. Actual GameWon5199/quantity0 is accepted; four of each recipe material are consumed and exactly one food35 granted, quest910004 final count1, coins110 unchanged. No turn-in/hat reward yet.
- The completed movie remains visible. Actual topX then Leave emits no lifecycle request after FRServer_GameOver; this is a finish-presentation defect, not a missing gameplay completion. ExactPID shutdown finalizes8122datagrams and preserves immutable opening-cooking-after-native-win.json SHA2567e18b7b49b29a4eab2c24fa226a3c7e209ee8a85986adee5c0ef4b9646ac4db5. In-session finish/close, Mae turn-in and restart acceptance remain pending.

## 2026-10-08 17:13 UTC — Crafting results repair reviewed and rebuilt

- Original native/Lua evidence distinguishes automatic39/7 acknowledgment from the genuine four-byte26/42 results Close. Cooking/Smelting now present actual score/elapsed, a nonempty original total-score row and StateRemove; Forging preserves its delayed GameOver callback. Terminal economy remains until player Close, and Close emits no duplicate Remove. Ordinary in-game Quit39/6 remains a separate pending gate.
- Independent review reproduces30 crafting tests,9 private gameplay checks, the mapped17-stage restart flow,16 unchanged community writer vectors and251 original instruction bytes; no blocking findings. Proof: lab/reports/native-crafting-results-independent-review.json.
- Combined formatting and ordinary suite pass151 tests,0 failures,28 explicitly private checks ignored. Release SHA25639167b9d5c9f941d9caee555a972709fc21e7df7015b8b93c9e790064c09b431 also includes the independently reviewed loss boss-display cleanup; native acceptance of both changes remains pending.
- Started a new actual-client replay from the byte-identical pregame opening-cooking-finish-retest branch, preserving the earlier native win separately. Next gate: six rounds, visible actual results, genuine player Close, Mae turn-in and restart; no economy merging or fabricated completion.

## 2026-10-08 17:34 UTC — Native Cooking finish/turn-in accepted; inventory restart defect localized

- Native171251 completes all six original rounds, score5133/time101, shows original YOU WIN/TOTAL SCORE, and keeps terminalWon after automatic39/7. Actual playerClose26/42 after62.219seconds returns to the farm. Mae turns in Cooking101 once: wallet120,ChefXP25,onehat14794 and onefood35; repeat interaction grants nothing. Final4177datagrams/4156CRC and original whole-self/item writer comparisons pass. Proof: lab/runs/client-baseline-20261008T171251Z/analysis/native-cooking-finish-turnin-proof.json SHA2566ddb94aef86306fd26fd035f4471ecbd4672bab2b80fc3b7b7a852f16d4cff30.
- Immutable completed/turned-in snapshot SHA256d4a1df6ce4688bf81669172b293e4d186adef4bc5a1942668e2bac7988735341. Native172324 restores exact quest/economy/identity in the original self; observed inventory and gear categories are empty. Original startup35/37 requires a payload byte-length before definition count; current source omits that field, while runtime38/2 rewards correctly display. Narrow reader/writer-backed fix and regression are in progress.
- Stopped172324 safely with no stage5accepted; only character movement changed after restart. Preserved postrestart/beforefix snapshot SHA25604cc44149c720e16be74bb8963acfb5d22d7167f8a9710bd809b45b82ae144cf. Current position is nearCleetus. New catalogue visibility acceptance precedes mining/smithing.
- Static original Smelting/Forging geometry and stages5–17 play guides are saved with pinned community provenance. These do not replace native acceptance. Ordinary minigameQuit39/6 request is proved by native and three compiled community vectors; original response remains unresolved and no guessed production handler was added.

## 2026-10-08 18:15 UTC — Inventory restart and native Smelting accepted; save-lock failure isolated

- Repaired startup35/37 catalogue byte-length framing at the original serializer boundary. Exact311622-byte/1647-definition output matches the unchanged community writer. Independent original-reader/source/vector review and14focused checks pass. Native173814 visibly restores All Items, hats and Chef head gear; prior completed Cooking101/economy/identity remain exact. Ordinary suite152passed/0failed/29explicitprivateignored; releaseeabbceee32e1a44c63521e9b9a5988aabcc1271a7b431c8df92526538715b8c0.
- Gear Up now has actual Miner selection/five gathers/four original Smelting rounds. GameWon5497/time448 grants10bars once, consumes5ore/3coal, keepsWon after automaticACK and returns only after actual26/42 Close ~100s later. Final score-page pixels were not retained: acceptance is exact wire/held-state/actualClose/world return. Cleetus adds10coins/25MinerXP once, then separately accepts Your First Weapon and grants its supply bundle. See Reviews/Native Gear Up and Smelting173814.
- Forging139 accepts six normal native chains, two collected hammers and score6600. The Rust session ends with WindowsAccessDenied5 after frame10693; later client-local chains receive no server replies. This explains the later visible/saved mismatch without proving a gravity defect. Client exits0; finalized12525datagrams and immutable checkpointopening-after-smelting-forging-incomplete-173814.json SHA256f90e31dc2db996a43b0a625783bebd4f8da0a05f24ce65fc41cfff7a438a5cbe preserve the incomplete attempt and untouched recipe inputs.
- Native181038 restarts the same campaign: completedSmelting/130coins/Blacksmith and unused materials persist; only interruptedForging is cancelled, without output/consumption. Actual Stuff and journal show restored items and Blacksmith complete/Forge pending. Final1479datagrams; all services stopped for the repair build.
- A controlled Windows reader withoutFILE_SHARE_DELETE reproduces exactAccessDenied5 on the old atomic rename. A narrow same-rename retry (Windows5/32only, at most500ms waiting) passes the distinguishing transient-reader regression and permanent-denial preservation checks. Native logs do not identify the exact reader/file-op callstack; this is a reproduced compatible cause, not attribution to a user or process. Independent review/full-suite/release/nativeForge retry are in progress. No gravity/content/save-schema changes.

## 2026-10-08 18:28 UTC — Save retry reviewed; native Forging and hammer progression accepted

- Independent atomic-save review passes12focused checks; fullordinary153passed/0failed/29privateignored across32groups, formattingclean. Releasec642323aa75de78bd325c7e8e8a0b5530f66597544be1575aa679564130a8654 built13.90s. This adds only bounded Windows5/32 same-rename retries; no Forge/gravity/content/schema change.
- Native181038 and181715 each reproduce154349bytes of original wholePC serialization with active Your First Weapon and mapped labels. Final1810381479frames preserve entire gameplay except cancellation of incompleteForge; unused materials/coins130/Blacksmith/equipment/quests/identity remain exact. Final17381412525frames independently bound actual Smelting completion/turn-in and the later IO session failure. Proofs: native-inventory-smelting-forging-final-proof.json and native-forging-cancel-restart-final-proof.json in their respective analysis directories.
- Native181715 plays18normaldragchains, collects5hammers, completes original mold/delayed callback and shows YOUWIN/TOTALSCORE13100. AutomaticACK retainsWon; genuine26/42 Close returns to the farm. Exact one recipe consumes10Bars/5Gem/5Planks/5Fasteners and grantsHammer30280x1/id31, leavingBars10. ActualCleetus turn-in adds16coins/25BlacksmithXP once, wallet146; independent whole-state allowance passes. Checkpointopening-after-native-forging-turnin-181715.json SHA256e43e8d869861850db78277657188e82649caf0ed26783cd72265c2fa3c4cdc49. Proofanalysis-live/native-forging-results-turnin-proof.json SHAdf1973a8ecb2e5fc2ad23d727f8bdf3e1038726ff2ae42bc43c0604a0e4bc803. Terminalstate comes from recorded full-state log and observedWon, not a separately copied held save.
- Subsequent nativeCleetus interaction accepts stage7 without duplicate stage6 reward. Brawler selection and genuine Gear→Weapon forgedhammer equip advance[1,1,0]. Actual walking toward wolves is ongoing; no world kill/native-stage7 completion claim yet. Dynamic anvil's boulder occlusion is separately confirmed and a narrow versioned placement repair is being prepared without active content/save mutation.

## 2026-10-08 18:38 UTC — World knockout traced; narrow anvil placement migrated

- Native181715 first outdoor approach reaches a tree at[-2098.401,-52.621204,423.78705]. Three wolves pursue,315movement packets and original run/idle/attack1001 callbacks are recorded. Forty enemy hits×25damage within≤2.4814units knock out the idle character in17.47seconds. No native36/10 player attack request preceded knockout. Respawn restores1000HP at local opening checkpoint;146coins/items/quests/equipment remain intact and all5wolves remain180HP/count0. This verifies wire recovery, not a successful fight or native wolf skinning. Final6101datagrams/6080CRCs; allservicesstopped.
- The cyan/orange screenshot shapes are independently matched to original static fairy-flora at the blocked tree. Definition3156/model3284 correctly resolves wolf_generic.adr, original48bonewolf skeleton and all33animation slots; no guessed model/scale/material repair was made. Fresh actual wolf pixels remain required.
- Approved anvil move[-2084,-55.6,190]→[-2088,-54.79,200] removes the confirmed boulder overlap. Original-reader geometry, fullmodel clearance and exact one-actor/package delta are independently reviewed;44+17unsafe/rejection cases and byte-exact rollback pass. Migrated only a NEWsnapshotopening-anvil-character.json from immutablepostknockout3a3b159dc68447171ee095ff55f0771872b6532a906cb008f7d2c3ad23c21749. NewstateSHA0cc3496c99008ed8e9dd6b3a3a8a38de8fde50b9282c852ffae32a428e3bdcd3; onlyopening.content_sha256→4e6ac15cd19b5785a04ac7eb944bf0235f4a0592f4c42df755f89c47896b0ad1 changes. All29522other snapshotbytes/identity/character/gameplay are preserved. Independent10Rustchecks include full17mapped quest/restarts.
- Native183551 uses reviewed anvil-placement-v1 content/journal and unchangedc642release. Original wholePC154396bytes and identity/migration/persistence checks pass. Root is retrying worldcombat with normalinput from the saved opening checkpoint. Nativeanvil visibility/approach remains pending; relocation does not claim collision-parser support.

## 2026-10-08 18:55 UTC — Native five-wolf combat
Native183551 ordinary W4300ms then ten normal key1 holds defeats all five stage7 wolves, goal[1,1,5], HP525, coins146. Native chat records five distinct kills. Cleetus reward remains pending; whole starting-area acceptance remains open. Preserved checkpoint opening-after-native-five-wolves-183551.json SHA25668b2f561cba5745dddec374a3f75ed6fee51946f02db3587c771b2d43097cafd. Reviewer is finalizing packet/range/state proof; actual recording will be inspected for wolf visuals. Earlier183551 failed attack used unsupported automation tap commands, then corrected keys arrived after knockout/news; no gameplay fix was inferred from that test failure.


## 2026-10-08 19:17 UTC — Wolves rewarded, anvil native acceptance, Gloam retry
Native183551 Cleetus ordinary interaction completes Fighting off the Pack:18coins146→164,25BrawlerXP125→150, stage7completed/rewarded, worldstage0. Whole gameplay otherwise preserved. Immutable checkpoint opening-after-native-wolf-turnin-183551.json SHAc24b9d8dbabe68335b29cc1a1be2b2061786b95d05fe079b8753103788d7347d. Actual recording now independently inspected: correct canine models run/lunge/take100damage/disappear; root viewed the retained contact sheet. Review: notes/Reviews/Native Wolves and Anvil 183551.md. New anvil visibly clear of the boulder, approached on foot and interacted with normally; Brawler correctly receives Blacksmith requirement.
Pappy stage8 accepted[1,0]. Gloam approaches/hits within2.399973m; root paused26.819s before attackbatch, and525HP was exhausted22.328s afteracceptance. All eight later keyholds were blockedby respawnNews; no36/10/nohit/noquestreward, not evidenceofacombatdefect. RecoveryHP1000 preserves164coins/gear/quests. Private ordinary-input helper now binds exact wolves/Gloam/Mac stages/GUIDs, passes isolated/independentchecks and is ready for retry; it has not yet supplied native acceptance. Current character is following the known trail back.
Historical coverage ledger explicitly keeps the complete historical starting area open beyond17locallyboundstages: notes/Research/Opening Historical Coverage Ledger.md.


### 2026-10-08 19:33 UTC — native Gloam completion and restart

Native183551 completes The Gloam Council with the reviewed read-only-state combat helper sending ordinary W/A/D/key1 input. Three accepted100-damage attacks kill Gloam at1.10327m; three35-damage enemy attacks leave895HP. Native image retains the purple Gloam. Whole-state checkpoint b90b2e22ac9934c7fde7920b6e934faa7146557e441f9b74ab3abce707d18092 is immutable. Pappy grants20coins/25BrawlerXP once, yielding184coins/175XP; turn-in aff06523132d2e1977af97711af3211116dedd75d5b91bce202c1c2041edb3bc. Next ordinary interaction accepts stage9 Where's the Help? without another reward; pre-restart8c8cf8dac23ffbd04f0a45d69d38ad4192f8872eb4b621f8d30a2735b7925920.

Two earlier failures were test-control mistakes: idle time after acceptance and a wrong assumption that32m was outside aggro. Existing source starts pursuit within40m. Neither failure emitted a basic attack; the live client returned through ordinary knockout. No combat balance/AI repair was justified. The successful helper starts before entering aggro and uses normal inputs only.

Final183551 independent audit95f0fea3aeb25fc5f56bed2aeb22a2b2d2c26065db7c743391ae41e88e5493bc verifies15,829datagrams,15,808gatewayCRCs, complete reassembly, original PC/catalogue, kill/reward progression and deliberate shutdown. Native193318 restart proofc2fc63162da569262e503459d9716b42ff906dc11c39a4c3887f9132edbc665c preserves all gameplay exactly; only the original initial125 packet normalizes rotationZ by onef32ULP. Immutable restartfed38581bd35b78ed25fe841d5a6ff695d4b912512d48c20022ab6d72def247e. Native screenshot shows895HP, Brawler and equipped hammer. Continuing on foot from Pappy to Bartle; broader historical opening coverage remains incomplete.

### 2026-10-08 20:31 UTC — Native Bartle/artifacts/Mac and reviewed prompt build

Native193318 completed stage9 Bartle (+10coins/+25BrawlerXP), three ordered artifact interactions/removals and stage10 turn-in (+10/+25), then the outdoor Mac fight and stage11 turn-in (+25/+25). Ordinary aligned Mac retry took18.625s: ten W batches, five key1 holds, five100-damage hits and five45-damage incoming hits; HP1000→775. Earlier turning/time-budget/idle failures remain documented as test-control errors without a combat source change. Final balance229coins/250BrawlerXP, equipped Hammer31/43/7, stage12accepted[0,0]. Final authoritative193318 proofa0da5d589fe9819a1874a8456c4ef0700c4007ff1b3f58181ea4f9933f0a35c0 verifies11,531datagrams, complete reassembly, exact originalPC/catalogue, all historical live-prefix proofs and services stopped. Final checkpoint3581758cbc15966a6c6c890d3d383ee42aa4cb1e520c1add1361ec3cd2c9f164. Video reviewc4fcd7269455d21d47b7818f800b47b3fa5c389d70272b964a5f9d67e2e46add retains actual Mac approach, gray-hammer pose changes, damage/goal/removal; complete unobscured swing or death/corpse animation remains unproved.

Integrated exactly one AddNpc field write at template237 before UTF8-name splice, plus explanatory comment, preserving int133 and all other bytes. Original native trace proves this float feeds proximity acquisition; old zero falls back to global7 while the existing reconstructed server guard is12. Permanent entirely synthetic unchanged-community vectors cover full UTF8/fractional packets, preserved legacy integer, menus and filtering. Actual frozen-old RED, new GREEN and independent full candidate/source review pass. Formatting, nearest focused checks and full ordinary suite pass154/0 with29privateignored. Release SHA EA297301AFC5462BE7F40142D2A6B4709EB94AF6B3556DE721DFA407D9A89063; build log lab/reports/native-interaction-radius-release-v1.

Fresh203139 restart checkpointb7265056e72ab0047420f51dc000a9a4b70651e8b2d38a1245d95c826b52e72c preserves entire gameplay; one native initial125 rotation normalization differs by onef32ULP. OriginalPC154,289bytes/catalogue311,622bytes match unchanged compiled writers. All18 delivered opening actors match candidate vectors and carryfloat12.0; six consumed props/all dead enemies remain absent. Actual facingaway7.15m and positive-facing13.78m X give no menu. FacingBartle7.15m and10.96m X give intended menus and ordinary choices, with wholeGP/rewards unchanged. Restart proofe945f78684b9a702b5c06e0d9f708bec8e4f41dcb83f5716331c4308397ade93; independent visual/wire gate receipts follow in owned notes. Active save/content/identity unchanged. Walking reviewed road towardQueen; no whole-opening/historical completion claim.

### 2026-10-08 20:56 UTC — Native Queen stage12 checkpoint

Native203139 walked the main road, Sanctuary climb and both lilypad bridges using ordinary A/D/W. Two bounded helper stops were heading checks: tiny D84/D80 yaw changes near rampnode24, and an opposite-sign returned D149 yaw near802. Both were inspected in fresh screenshots and corrected with ordinary inputs; no W obstruction, jump, teleport, save edit or gameplay patch was demonstrated. Exact cause of the opposite-sign yaw is unproved. Route center804 was not walked into; the actual Queen interaction position[-1063.606,4.4188066,305.68225] is9.47m away and safely within the reviewed12m prompt.

Immutable pre-discovery d83e87cdddb1aa896565c40cdada5247b98e417c14f1a8609147ce7ae89573ee; post-discovery0314f40fa65915cf06b96d181bfc2ea65e5ca920c24c30cec40bb93aec74fd9c changes only discovery4/+5coins/+25BrawlerXP. Separate25m visit checkpointb970e9912eaa015eafee582f878e51400ce9eb9737c9b202997fbc76935f0315 recordsstage12[1,0]. Source-consistent discovery XP corrects an earlier preparation estimate that omitted it. Bounded wire proof3091b07ed49f81d79c9339b9a795581052585ba248773dd34a35400aada564cc preserves exact native wallet/XP and goal events.

Native Queen X menu/ordinarychoice completes stage12[1,1] and grants25coins/+25BrawlerXP; final259coins/300XP/Brawlerlevel4/775HP. Beforetalkb2e2eff3149e67adc4514d57086373440c9744d013be596bd310da53e342230b; afterturnin3a9d38078449b7dd95fdd8263a733b9d77e44b24ffe740d1e0a44deff07e2681. A second actual menu/choice produces the byte-identical3a9d… checkpoint, with no duplicate rewards or stage13 acceptance. Native screenshots retain QuestCompleted/level4 and Stuff wallet259. No palace interior is restored: this is the existing local zone0 visit/talk binding.

Prompt/Queen recording cleanly closed, then exact verified203139 clientPID1684 deliberately stopped. Harness exits0;7065datagrams, allclient/login/serverstopped, manifest binds newreleaseea297301afc5462be7f40142d2a6b4709eb94af6b3556de721dfa407d9a89063. Independent bounded prompt review9edb7e892ad2a5b1fbe5ce15d35d6d2ce725b4649986e509cbb8571ba7770a8b passes whole source/vector/state/pixel/wire gates; finalcapture and route reviews are being frozen separately. Currentcampaign stops at Queen with12/17localstages rewarded. Next is return toMae for stage13, then mining/repeatcrafting, job-center/classes/hat and finalrestart. Whole historical startingarea remains incomplete.

Final native203139 authoritative receipt1332451d1312374027ca8c116f23ac81206fa98735169a3079bc0b5faec1b8c2 passes all7065datagram/live-index/final-manifest hashes,7048gateway/7046CRCs, complete reassembly, closed live writer, preserved identity/locale/title files and actualEA297301release binding. Original wholePC154289/catalogue311622 remain exact unchanged-writer bytes. Retained startup/prompt/discovery/visit prefixes rederive identically. Queen choice6987 changes onlyTalkgoal/completion/+25coins/+25XP; nativewallet259/rank4/questremove match. Repeatedchoice7029 has no second reward/newquest; turn-in/repeat/finalsaved files byte-identical3a9d…. Allinventory/materials/gear/775HP/priorprogress preserved. Newreview: notes/Reviews/Native Queen and Prompt 203139.md. Route/video review is retained separately; mainCurrentProgress identifies12/17localstages and remaininggates.

Final route review passed: lab/reports/native-bartle-queen-route-203139-final-v1/proof.json SHA1f9b1bd09fcfe32c9d7f2fb1123168d2dcd89720403a48c73482bb01ff4f059d. It retains40terminal navigation logs,38route logs/36arrivals and490successful input matches; observedhandoffindices1–30/bridge subdivisions, no recordedSpace, both headingguards and normalcorrections. Root viewed original video bridge frames as well as live captures. Actual endpoint9.473425m fromQueen is accepted; exact804center remains explicitly unvisited. CleanvideoSHA6a25f2d5951dfd07b462764777b95194b1dd74bd9198a1cb0fdf65f7dc312310, observedclimb/bothleafbridges/Queenprompt and allstopped-service metadata are bound. Render/proxy preparation is not a universalcollider/pathfinding guarantee; finalnative wire/economy remains in1332451d… receipt. Owned route note is updated.

## 2026-10-08 23:16 UTC — DeepSeek worker tooling configured

Installed and enabled codex-on-crack 2.2.2+codex.20261002 (9dee6ca9aaa73e0008123eff872871e88b94a12a) and Codex Router 0.6.0 (444a72809d1609020516e14438c2075c5f4bda8c). the owner entered the key privately, chose DeepSeek V4.1 Flash, then explicitly requested max reasoning. Router operator selection and crack_builder both bind deepseek/deepseek-v4.1-flash/max. Existing gpt-6-astra lead/effort and unrelated configuration are preserved. Router doctor reports 88 checks/zero failures; crack doctor is static-ready with the role in sync. Source files are unchanged. AppData virtualization and a stale moved-venv LiteLLM launcher were diagnosed and repaired; the stable checkout is under .codex/tools/codex-router.

No paid installation probes or actual model turns were made. Local operator selection does not prove native delegation. Fully restart Codex before the first real worker task, then inspect actual child model/provider/effort and returned evidence. Private verification/undo receipts remain outside Git. Obsidian research note and task gate: notes/Research/DeepSeek Worker Setup.md. Game code/save/acceptance remain at the Queen checkpoint with 12/17 local stages rewarded.

## 2026-10-09 00:19 UTC — delegated test-first continuation

User explicitly authorized workers and continued implementation/testing. Workers' own matching session metadata records DeepSeek V4.1 Flash/max and actual tool/test/artifact work. Recovery guide and self-review were independently checked and corrected (seven findings); guide SHA256122dbb96e3e2a77355d1bd3680975fd9b3f9756ddb8c462187553358ab4f9a01, self-review f1fb9ce99a98addde461ca4bba26546c5079e3c550179ba107e7724867bf4081. Recovery inputs re-hashed, community readers/sample oracles re-run; full exports remain recorded rather than repeated. New notes: Verified Recovery Workflow, Recovery Workflow Evidence Review, Recovery Workflow Independent Review, Remaining Opening Preflight.

Crafting candidate recognizes exact15-byte native Quit39/6/current-instance/-1/-1 only for pre-win active runs, calls existing cancel without transaction, retains terminal/resultsClose/ACK behavior. Baseline154/0/29ignored; thirteen new regressions RED before production changes, then GREEN. Post-format suite179/0/29ignored includes13new library plus12 repeated integration tests. Pinned request provenance rechecked; C# writer not recompiled, historical server response unrecovered. Private ignored fire-extinguishing test failed on content selection and is being independently compared against pre-change production. No native Quit acceptance or release deployment claimed yet.

Canonical Queen source3a9d38078449b7dd95fdd8263a733b9d77e44b24ffe740d1e0a44deff07e2681 and prior releaseea297301afc5462be7f40142d2a6b4709eb94af6b3556de721dfa407d9a89063 reverified. Separate copied save/identity branch opening-continuation-20261008T235029Z launches bounded native235041. Recorded walk crosses both lilypad bridges and descends main road; guarded short-turn stops corrected normally, time-bound stops retained. A segment wrapper timeout also remains in evidence, followed by fresh pose/arrival recheck. Repeated foreground loss to ChatGPT prevents further input; asked whether user is actively using keyboard/mouse while background work continues. No quest advanced; source Queen checkpoint preserved.

## 2026-10-09 01:14 UTC — native developer teleport and stage13 preparation

Built the independently reviewed default-off developer teleport candidate: release e1b9f8def10e10a53b7dada29df0547af330edd187f42a5f8e2545485f1954d0, build exit0, old accepted binary backup unchanged ea297301…. Ordinary suite185pass/zero fail/35ignored; six private integration checks explicitly pass. Native005748 runs --dev-tools on the separate opening-developer-20261009T004212Z branch. Actual realm goto Mae sends writer-exact38/12 and receives native position reports, including normal terrain-height adjustment; final arrival4.004286676m fromMae, whole gameplay unchanged. Frozen1702-frame independent prefix audit passes28checks with complete reassembly; delivered unknown/ambiguous/list/gotomae controls are distinguished from earlier undelivered rapid inputs. Native frame/CRC closure and recording remain open until the run ends.

Root visibly opens Mae's menu and accepts stage13, then gathers4Rainleaf/4Gummyfruit through original interaction menus. Stage13counts[1,4,4,0], still12/17rewarded. Chat retains input focus after submission; mouse clicks on the visible interaction prompt and its radial Interact button avoid accidental chat typing/world walking. Native clustered-actor target competition motivates a scoped arrival-direction refinement before the next release. Developer relocation is not walked-terrain proof. Native Quit and Cooking victory/turn-in remain open. Progress/reviews are recorded in Obsidian.

## October8 Pacific — broader MMO delegation and native crafting closure

the owner explicitly requested100workers, kart/soccer/housing and the wider MMO, and explicitly selected RTK/Ponytail/Caveman/relevant game skills for workers. Updated project instructions; created notes/Design/MMO Worker Contract.md and notes/MMO Work Queue.md with100 scoped pieces, exclusive ownership and separate test/review/native gates. Saved config features.multi_agent_v2.max_concurrent_threads_per_session100 was verified after the owner changed it. The live session still rejects a sixth active worker with agent thread limit reached. No config or router source was changed by this audit; root reuses finished slots and preserves continuation before a Codex reload.

Mining17, harvesting17, fishing12 after lead corrections, kart11 and soccer10 tests independently rerun by lead:67passed/zero failed/zero ignored, no warnings. Log lab/reports/crafting-quit-native-005748-20261009T011957Z/lead-minigame-candidates-tests.log. Candidates recover original protocol/movie/data behavior with clear reconstructed rules and fidelity gaps. They are not registered in lib.rs, deployed or native accepted. Fishing initially ignored actor handles and accepted invalid geometry; worker repaired those pre-integration findings and corrected trace-bound/native-limit and inbound-codec wording. Kart worker reported crate-wide formatting of other new modules; ownership instructions now explicitly require scoped formatting. Housing, derby, activity/job/quest catalogues fill subsequent active slots. Future-tense child finals were interrupted and followed up; final status is not inferred from ongoing-intent text.

Refined teleport releaseec43e1de… retains default-off behavior; independent013155 bounded native review confirms targeted Mae/CookingTable prompts with whole gameplay preserved. Old005748 finalized9524frames/9507gateway/9505CRCs and26prior checks all pass. Root closed Cooking Playing Quit: request39/6 currentinstance46000004 frame009416 at1820.422s, remove39/19 frame009418 at1820.469s, ACK39/7 frame009420 at1820.531s. Playing snapshot differs from world-restored/final save only by cooking.active→null; ingredients/output/quest/economy preserved. Actual timed X/Leave batch succeeds within original3second confirmation timeout. Final recording frame10seconds before end shows restored farm/prompt; misleading earlier screenshot names are explicitly excluded. Reproducible root audit exits0, receipt final-quit-audit.json and review notes/Reviews/Native Crafting Quit005748.md. OfferCancel used26/42 plus staleinstance0 Quit; not liveOffered39/6 proof.

Fresh013155 offered46000005. the owner manually played six Cooking rounds and reported workswell. Final saved score4700/allroundstats, phasePlaying/GameStatsNone/completed1; clientexit0 occurred before final reward confirmation. No new stew or stage13 completion claimed. Both native runs/services stopped; immutable Queen3a9d… rollback retained, copied farm branch stage13[1,4,4,0] still12/17rewarded. Native completion/turn-in and stages14–17 remain open independently of broad MMO dispatch.

## October8 Pacific — actual100 dispatch and independent activity repairs

Post-relaunch capacity is proven by accepted worker receipts:6 initial reviewers +21 root pieces028–048 + coordinator100 +21 coordinator pieces007–027 +51 coordinator pieces049–099 =100 assignments. Root independently parsed72 unique coordinator ids and21 direct receipts, with no recorded failures. Lead MMO Dispatch Snapshot updated; this is total dispatch, not100 concurrent workers or native accepted features. Independent mining/kart/soccer/housing reviews repaired real compatibility/validation/persistence defects; worker GREEN logs remain candidates pending lead integration/native gates. Class/gear/cosmetic and housing harness artifacts returned with explicit recovery gaps.

Native040714 launch has incomplete finalization: process inspection finds no client, exec session ended1, final run receipt absent; logs record disconnect/idle timeout. No current recipe completion or new quest reward accepted. Copied branch and immutable Queen remain preserved. Root continues capture diagnosis and native opening; coordinator supervises remaining scoped workers.

Lead reran7 repaired activity targets:104passed/0failed/0ignored, exit0. Preserved log lab/reports/lead-reviewed-activities-20261009T0450.log. Allroot pieces028–048 now reported concrete candidates and were interrupted. Queue updated toactual assignments with review/integration/native gates explicit. Fresh bounded native relaunch started with hidden launcherPID11444, receiptincontinuationbranch; run054421 notyetaccepted.

## October8 Pacific — native14 checkpoint and lead economy repairs

Native054421 complete:9206frames/9188gateway/9186CRC, fullreassembly, allstopped. Cooking46000006 Won5199/198s/28stats consumesfour each2862/2900/2884/2875 andfood35+1 atGameWon, beforeClose; Maeonly+10coins/+25ChefXP. ActualresultsClose clearsactiveonly; Mae repeatunchanged. Miner14 selected, tenoremenus2739+10/879+10, eleventhnochange, Cleetus+10coins/+25MinerXP.14/17rewarded,279coins,stage15unaccepted. Independentreview corrected two report attribution/screenshot mistakes, preservingolderreceipts. Finalreceipt a04096df5f8716a4c1b9d67e44487777a59cb2bb6822377927f8703f6bcfda0c; preservednative14 snapshotbf3dc76b….

Lead independently reproduced5RED economy defects, then integrated piece081 pureallocator into optional InventoryState.next_instance_id savedwithitems. Default/legacywire bytesunchanged; allocation metadata appears onlyupon acceptedmint. grant_bundle stageswholeinventory; legacyquests complete-but-unrewarded remain retryable, ack onlyafterfullbundle; fullbag ordinaryrefusal.18regressionsGREENincludingrestart/retry/corrupt/exhausted counters/latebundle rollback. Wholepackage1365passed/0failed/57ignored; independent3privategoldenspass. Scopedformat only, no publish/commit. Newreleasea86615d22657f92571300dfc0b853c089d5f5eabdaace5d996ae2533b4824f3a buildexit0; no newnativeacceptanceyet. Preservedacceptedec43binary. Pendingrewardretrypolicy reconstructed, priorretired IDs absentfromlegacy snapshots cannotbe inferred; explicit migration mayfloorfrompreservedhistory. Opening/encounterfullbag still atomicviaexistinggateway stagingbutcurrentlysessionerror.

## October 9, 2026 Pacific: native local opening17/17

Economy releasea86615d2 completed native103605 stages15-17: actual Smelting5779, Forging55chains/fivehammers39300, Cleetus15, ordered jobtour16, native classes1/11/35/45 andChefhat16equip/turnin17. All17rewarded309coins; repeated15/17turnins byteidentical. Independentreview validates16135frames/16115CRCs/reassembly, correctrecipe/economypackets. Fresh111125native reload keepswholegameplay/hat16/counter52; Sunsparkduplicateagainunchanged. Explicitlegacycounterfloor36 from11preservedsnapshots; newingredientids36-38retired thennewclassgear39-51minted. Noothergameplaymigration. Historicalquestcoverage/terrainwalk/minigameintegration remainopen. Postcampaigncaveentrymissing; scopedordinaryreplayfix assignedbeforefullnewnativecombatpass.


## October 9, 2026 Pacific: replay score and first native Mining chain

Release6935fd25 is preserved before further edits. Expanded ordinary suite1410passed/0failed/57privateignored plus three explicit private goldens; independent Mining gateway and score source reviews pass. Native121834 completes cave instance5 with allfour normal melee kills/player700, goal5000 and total5000, actual Close/world return, no repeated economy/progression rewards. Independent capture audit verifies7440datagrams/7413CRCs, full reassembly, all17 stages/309coins/counter52 preserved.

Mining18/Miner14 opt-in board18000001 launches through the original movie; a real three-tile drag yields onechain/400points. Native pause and quit return to world, completed0/activeNone/economy unchanged. Both ore goals label Iron and regulars render blue: real visual bugs, not accepted as complete. Whole-round win/restart/world entry remain pending; rules reconstructed and default outputs empty. Review: notes/Reviews/Native Mining Score 121834.md. Source save copied unedited from114509; content36base files preserved with optional mining.json only.

Follow-up original-movie diagnosis: OnTileInfo second argument is type index, not category; captured400/600 indices refer to missing frames. Original601/602 both translate to Iron, representing different qualities, so duplicate Iron text is correct and no label rewrite is authorized. Rendering repair is scoped to the proven index error with regression-first verification.


## October 9 Pacific: Mining colours fixed, broader gate qualified

Frozen sourcee6e421d8 emits value-category with checked underflow. Regression RED3failed/GREEN3; independent differential4pass proves same seeded board/state/queues/allnon-index replies, exactoriginal sevenindices. Release7d5be63f built/preserved; native125556 originalmovie shows fivecolours/distinctironqualities, manualfourtilechain600 accepted. WholeMiningwin remains open. Driver handshake/actual1279x960 repaired; ACK/terminal/mup findings being fixed before batchplay. Native125556 uses copiedunedited Miner14 state a6adf508... and unchanged optionalMining121330 content.

Broad gate runs1280prefix+125remaining ordinarypasses/56ignored and threeprivatepasses. Fullcommand101 solelybecause SmartAppControl4551 prevented piece_097 executableCDC60A84... fromrunning; samebinaryfocusedretryalso101. Eightordinarytests/oneprivategateunexecuted. CodeIntegrity3033/3077/3118 captured; nosecuritysettings changed, nofullsuitepassclaim. HarvestingChef45 minimalrepair17+3+4andindependent4passes; gatewaygame111candidate nowassignedwithoutquestgather/recipecoupling orreleasechanges.


### Native Mining125556 progression blocker, October9 UTC

Frozen7d5 release passed native sprite rendering and six ordinary chains/3000points. Zero ore collected and the settled board has no legal moves; whole-round acceptance fails. Quit cleared the active run and returned to the world without economy/progression changes. Capture13834gateway+23login/13832CRC3/full reassembly is preserved; independent audit pending. Harvesting gateway remains a separate source candidate with seven focused targets passing root rerun, reward-definition/content and independent review gates open.

## Mining/Harvesting continuation checkpoint, October 9 UTC

Native Mining `client-baseline-20261009T134930Z`, frozen release `ef85c0d9...`, reached 41 normal mouse chains/25,400 points, zero ore and three reshuffles. This is diagnostic failure evidence, not whole-round acceptance. A captured ore-to-reshuffle transition is under read-only replay/original-order review. Root canceled using developer chat `realm exit`; top-X clicks did not quit, so no ordinary result Close/TRY AGAIN is claimed. World screenshot `parity-exit-result.png` was captured inside the recording before its stop. Final save SHA256 `fdf9ab62145e1b37f47dd3f634b6064ce306c7598fc775d8a23ba388eebbbfd3`: activeNone/completed0/next18000004. Native client and recorder stopped; parent manifest finalized. Analyzer: 14716 gateway +18 login datagrams, 14714 CRC3, complete reassembly, self154395 bytes SHA256 `09d96f77ecb376dba268ffaef953a6bd30bd816c927b2b8738900ccf8c161285`. Independent capture/conservation audit is pending [[Reviews/Native Mining134930]].

Harvesting tile-registration source `9435797d...` has 79 focused checks green and independent three-probe review accepted [[Reviews/Harvesting Tile Registration Review]]. This change is newer than frozen releaseEF85 and its1438-test broad receipt. Review demonstrated the guard only ran in the writer; runtime load guard is now assigned. [[Research/Harvesting Border Compatibility]] independently proves original fall loops exclude x0/y0; minimal11x11 empty top/left padding preserves10x9 playable cells. Border source repair is assigned. Existing immutable Harvest package `opening-harvesting-native-20261009T132512Z` remains unchanged and blocked; no native Harvest completion claimed. Original shape template remains missing, so padding is a compatible reconstruction rather than a recovered original diamond.

## Native Mining143211: verified wildcard move-check blocker

FrozenEF85 with independently reviewed mouseplanner4c41 accepted21 ordinarychains/15,200points, zeroore and one reshuffle. Driver refused its nextdrag because no legalchainexists. Root exhaustive three-tile path scan of `character-chain21-no-move.json` (SHA256 `e3885ebeaab33c9f15232f13a9e70333b4d8a8ccd24db8077b30d14927b1bd24`) found0validpaths/all196idle. Wildcard200at(3,3) borders403(4,3) andore602(3,4); current has_move incorrectly accepts any neighbour of a wildcard, including goal/mixedcolour tiles. A bounded legal-three-tile check is assigned test-first; no quotas, seeds, rewards, shapes or movement handlers are being adjusted.

Run `client-baseline-20261009T143211Z` ended via developerchat realmexit cancellation (after unpause), TRYAGAIN visible. Both recordings and verifiedclient stopped; actual world screenshot is **outside both recordings**. First recording stopped while paused after a failed exit; second segment records successful cancellation/TRYAGAIN. See `exit-qualification.json` and upcoming [[Reviews/Native Mining143211]]. Finalsave SHA256 `e8c40a2e6781d38041d5b55511fc838c2111d507e81a99b447a54622730f1ea2`, activeNone/completed0/next18000005. Analyzer8379gateway+27login,8377CRC3/fullreassembly/self15439509d96f77...; independent audit pending. No Miningwin/reward claimed.

Harvest source a977c466/gameplayd65f now has writer/runtime tile+border guards, 56 currentfocusedtests green, independent review [[Reviews/Harvesting Runtime and Border Review]]. New immutablepackage `opening-harvesting-native-20261009T142624Z` has45files:41identical/1changedHarvestcontent/3newmetadata, inventory1703unchanged. Regional111c08f264e... padded11x11/90playable/8kinds/4realoutputs. Root broadgate `harvesting-complete-lead-gate-20261009T143500Z` executed58ordinarypass/0fail/2ignored before Windows ApplicationControl4551 prevented cooking_gameplay; both explicit private gates also unexecuted. Same-hash normalCooking retry blocked. Fullsuite is NOTpassed for thisrevision; security settings unchanged. PriorEF85 broad1438pass remains evidence for thatolderrevision only. Miningmovefix will require a new rootgate/build.

Read-only originaltutorialfield review maps genuine41/2 byte20flagclear0; originalKart readiness now recorded [[Research/Go Kart Native Integration Readiness]]. Kartlaunch/finish/zone/reward bindings remain open; no kartnativecompletion claimed.


### Native minigame continuation 2026-10-09T15:40:41.943839+00:00

Mining wildcard move detection repair reviewed independently: 400 generated boards plus captured-board probes agree with legal-path enumerators; the prior wildcard/ore false positive is removed. Root verification: 1,407 ordinary tests passed, zero observed failures, 56 ignored across executed targets. Five targets (piece_059, 072, 074, 081, 099) were not executed because Windows Application Control blocked their binaries. Four explicit private checks passed (two encounter, one opening, one Mining snapshot); this is a qualified partial suite, not a full-suite pass.

Frozen release SHA-256 cb0feab5c15338fa0104c7a75d20e07ada6ecc61610b561f1af41142d6d8f601; run client-baseline-20261009T151307Z; native PID 9688; Harvesting v2 immutable package. Mining instance 18000005 is under native test. No Mining or Harvesting full-round victory, reward persistence, or restart acceptance claimed yet.

Evidence: lab/reports/mining-move-lead-gate-20261009T150000Z/; notes/Reviews/Mining Move Detection Repair Review.md; lab/runs/client-baseline-20261009T151307Z/.


### Native collection and control checkpoint 2026-10-09T15:58:57.491458+00:00

Current151307 Mining trace records84 guarded driver chains and one additional unattributed chain; observed state85chains/54200score/collected[2,0]/5reshuffles/Playingpaused. No quota win. Window changed from reviewed1279x960 to1536x1152 outside the guarded driver, so root suspended UI input and asked whether the user is playing. Snapshot name character-mining-before-quit-chain84.json carries actual85, explicitly qualified in dispatch snapshot. Quit has not succeeded; no world-return claim.

[[Research/Mining Native Collection Trace]] verifies the first +500/one-ore transition via source-path plus accepted-state inference (C2Smessage encrypted), scoped to gravity/refill; reshuffle destroys ore. [[Research/Mining Focused Ore Strategy]] found no sufficient strategy advantage in its five-chain model; no driver swapped. Model/capture parity is being checked.

Kart lap/time/finished fields are independently checked in [[Reviews/Kart Finish Independent Worker Review]]; no finish captured. The author wrote both lobby/finish notes and their separatechecker, so that checker is explicitly a same-worker check. Independent lobby and actual world/scene mapping remain in research.


### Kart protocol and final Mining checkpoint 2026-10-09T16:18:59.455993+00:00

The complete native Kart102/2 five-table codec passed an independent32-probe review and private2393-byte capture roundtrip. Root registered the module and converted its tests to the production crate import; these registration changes still need the next root gate. The builder's preceding full-suite log has1468 ordinary passes, zero failures and59 ignored across155 result groups; root independently parsed that log. All five earlier Application-Control-blocked targets executed normally in this later run, without changing security settings. This is evidence for the pre-registration/pre-clickable-powerup revision, not the currently changing source tree. Log: ../lab/reports/kart-lobby-native-codec-20261009T1602Z/full-suite-attempt.log, SHA256296499ef5e57a9f77b21c3b8d204cf755bbcd6ca97da04e678898048c5746b82.

The pinned community opcode31 serializer matches captured Kart scene sg_kt_track, Derby sg_dm_arena and Soccer sg_soccer. Native scene-transition codec work is assigned; no custom GZNE parser is needed for the original client to load these scenes. Captured bytes remain in the lab. Native vehicle/match integration is not accepted. See [[Research/Kart Community World Binding]], [[Reviews/Kart Native Lobby Codec Review]].

Native151307 ended at its one-hour launcher bound. Final Mining state is Lost:85chains/54200score/collected[2,0]/5reshuffles/completed0, wall_ms1800000. This is a failed whole-round acceptance; no ordinary Quit, Close or world return was established. The84 documented driver chains and one unattributed chain remain distinguished. Final capture analysis reports22043 gateway plus51 login datagrams,22041 CRC3, complete reassembly. Final save SHA2567a0fc38a28fcc94b191fcc1a0163cb77a9e801b8cbb34b9ce8cb013ff384a2da. Independent final capture/save/media audit is assigned. UI input remains paused pending the existing control question; code and reviews continue.

Mining clickable-powerup950-953 implementation is assigned with zero-powerup bit/RNG/save parity required. Original timed900-902 effects remain deferred rather than emitting inert tiles. No newly modified source is deployed; frozen CB0 remains preserved.


### Verified source continuation 2026-10-09T16:34:06.937296+00:00

Independent final151307 audit is accepted as failed-run evidence: all17 rewarded local opening stages and their discoveries/content hash are preserved, alongside inventory/profiles/identity/other gameplay. Character position/rotation changed normally. Mining ended Lost by wall cap; no win/normal Close/world return. Media acceptance is metadata/header/size only, and the extra85th chain remains unattributed. See [[Reviews/Mining Native 151307 Final Review]].

Root's native Kart lobby responder passed two actual failing-before/passing-after private integration tests, reproduced independently. It serves only exact102/1 through the full102/2 codec when dev-tools and optional bounded typed content are present; save state and catalogue file stay unchanged. Raw private catalogue was exported through the reviewed Rust codec with a2393-byte exact re-encode; it is an archived test catalogue, not retail data. Root registered reviewed native progress and zone31 codecs; combined root gate/release still pending. See [[Research/Kart Native Gateway Lobby Binding]], [[Reviews/Kart Progress and Gateway Catalogue Review]], [[Reviews/Native Zone Transition Codec Review]].

Independent Soccer adapter review found real width errors despite prior green tests: state byte read as i32, control-bit word read as f32. Corrected native-wire vectors and codec repairs are assigned. Soccer remains candidate-only with no native match route. Mining clickable review now verifies allfour actions/server-owned selection and zero-powerup bit/state parity, but shuffle same-value placement and draw/refill ordering need corrections before acceptance. No new source is deployed and no user save/content is edited.

### Independent wire acceptance 2026-10-09T16:55:18.959274+00:00

Soccer corrected wire widths passed14 independent probes; private250state/1216control messages re-encode exactly, malformed/server-only authority checks pass. Root registered soccer/soccer_match, with production test imports and removal of raw captured fixtures assigned before the combined gate. No native match claim. Housing recovery establishes127/2 C2S placement vs127/40/44 replies; unchanged community133-byte fixture serialization matches all9 replies, remaining94bytes and first-dword identity are open. Independent response review and deeper recovery assigned. Mining fixes include no invalid S2C action echo and DOWN/UP/RIGHT/LEFT selection order. No release/content/save/UI change.

### Root gate and stronger runtime oracle 2026-10-09T17:15:03.036189+00:00

Frozen038 source passes1514 ordinary tests/0fail/65ignored and11 explicit private checks; every source pin unchanged. Complete2917 Soccer messages round-trip through production modules. Root then identified an incorrect shuffle placement oracle: distinct tiles were assigned wrong destinations. Independent expected-board probes reproduce RED for distinct/equal cases, while tiny-queue refill counts and advertised RNG values pass. Release held; correction assigned. Private Mining feasibility used bottom13 instead of playable10, so its1500-chain zero-ore conclusion is invalid and preserved as superseded evidence. ActualACK harness repair assigned. Housing full94-byte tail independently reproduced with native cursor and unchanged community writer; private Rust candidate nonempty-action oracle exposed a missing dictionary key, under repair before production. No new native play or release claimed.


### Repaired Mining independent runtime acceptance 2026-10-09T17:28:55.185080+00:00

Source4abe6b07f8edf52798ce810cde752a58fb959c30fcb639675371a041e463eb46, tests507945e9d42d2568572c70bfa3f1c93b2c12ab825c7a56524502fce18359f40a accepted by three actual independent runtime probes, retained at ../lab/reports/mining-clickable-repair-verify-20261009T171910Z/. Post-shuffle boards match literal original AS2 oracle with distinct/two-equal/all-equal values; queue low lengths1/20/50 consume then refill exact20-value batches in order; zero-powerup behavior matches previous default. Older1514/11 root gate pins038 and is not this revision. No native new feature acceptance, no shared balance/save changes. Corrected bottom-row10 model qualifies prior invalid bottom13 failure conclusions; private40/50percent pacing trials take26–29minutes and remain inactive. DeepSeek/max workers continue concrete Housing codec review, Mining clickable model, Soccer direction and Kart race-entry contracts. Shared UI remains paused pending control clarification.


### Combined source gate and deeper idle finding 2026-10-09T17:37:22.902978+00:00

Root ../lab/reports/lead-minigame-gate-20261009T173220Z/receipt.json verifies1530 ordinary pass/0failed/66ignored across161 resultgroups,12 explicit private checks and releasebuild exit0; source pinsunchanged. Ordinary log SHA256 49cba57a0bf3bb0ed423c54ea4a2913578d88c898360159290e4f82b0df6e17c. Frozenbinary 59032cfa5bd959fa45272c940a8d7bc8911018cd63845525f31ae07b360cbdef preserved at %USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\osfr-gateway-after-clickables-housing-59032cfa.exe; not deployed/nativeaccepted. Housing production9091118f codec/root wrapper independently accepted16codec+51inner checks against44vectors, three nonempty compiledwriter oracles.

Private Mining run2 exposed trueprotocolgap: OnIdleMsg shares coordinates() chainminimum3, while original Grid.UpdateIdle/GameLevel.OnIdleMsg sendsanynonempty list. One/two tile ACKs thereforefail. Narrowrepair assignedpiece028 with independentkart_review; newgate required. Run1/2clicktargetgenerator additionally usedwrongeligibilitypools; these failures are notviability/balanceconclusions. Sharedbalance/save/content unchanged, UIpaused.


### Fresh combined gate and research corrections 2026-10-09T17:55:17.186517+00:00

Fresh174628 gate1536ordinary/0failed/66ignored across162resultgroups,12explicitprivate checks, releasebuild0; source bytehashes unchanged. Receipt SHA256 ff262eb9db1bab0d20cde4cd5dcb318002d9c11d60be9eac539fb0bbfb87c791. Ordinary log9b5a7be371f420251bb7cbbe53d8f61cbe3203643a7c1bbaa663ba44bb62fe78. Frozen lab/tools/osfr-gateway-after-short-idle-21233dc7.exe SHA25621233dc79cc031d301e726c81d8330310f1078209c51a5a7bed89c9a824c9edf. Mining3cc6652b/Housing9091118f. Independent18-check short-idle review accepts one/two coordinate pairs and full rollback; chain minimum remains3. No shared balance/save/content activation or native input.

Lead doc correction introduced CRCRLF causing144 invalid bare-CR errors; byte-only replacement restored source and fresh gate passes. Failed rerun accidentally overwrote old173220 ordinary.log/source-before/source-after and appended failed exit. Old receipt/release/frozen590 unchanged; explicit disclosure ../lab/reports/lead-minigame-gate-20261009T173220Z/historical-log-overwrite-disclosure.json. Latest174628 evidence is intact. Do not use old overwritten log as original success evidence.

Mining corrected private clickable run3 on byte-identical3cc harness:250ms baseline1127chains/[15,14]/LevelComplete. Reconstructed4sec/action10/25/50percent trials accepted170/88/61clicks,0refusals, but Lost with[3,6]/[0,0]/[0,0]. They establish input-model parity, not feasible balance/native play. Runs1/2 remain invalid-driver receipts. Completion handshake is separately pending.

Soccer direction semantics independently accepted; private adapter being finished. Housing selection note corrected preceding7/8 vs all8membership, observedcooccurrence notserverauthority, explicit namespaces. Kartentry-v2 fixes stale layout claims:102/2allfive tables and opcode31tail fully decoded; gameplay integration still open. See current research/review notes.


### Private playability and direct Soccer review 2026-10-09T18:05:13.985122+00:00

Private baseline Mining natural1127chain completion drives ordinaryNextLevel→OnGameWonMsg/Won/completed_count1→GameOver→Close/activeNone. Corrected even-quota study:40%4+4=48chains192s,50%4+4=46chains184s,40%6+6=85chains340s; all normalhandshakes. Oldoddquota3/5trials refusedsourceper_reward divisibility and remain historical. Selected6+6 is feasible amongthree tested only, reconstructed4sec model, notnative timing. Baselinehash25370 preserved. Candidate24568d64 differs onlypercent10→40/quotas15→6,12→6. Packageopening-mining-starter-20261009T180011Z preserves46/47basefiles, changesMiningonly, adds2metadata. No launch/save/economy changes. Independent Mining replay pending.

Soccerreviewworker repeatedlyreturned intention-onlyfinals without executedprobes and was interrupted. Lead directly authored/executed13 additionalindependent tests in ../lab/reports/soccer-state-adapter-lead-review-20261009T180219Z/:13pass0failed0ignored, candidate11d291 and productionpinsunchanged. Actualshot/passballmotion verified; fullsnapshotsunchanged allrefusals; advisoryvectors/word do not move player, clocklag/spike noauthorityclockchanges. Corrected misleadingclaim: authority bounds resultingcharge*max_power, not max_power scale itself. Builder14GREEN retained; postimplementation12-test API-removalE0432RED only provesmissingAPI, not behavioral test-first evidence. Privateadapter remainsunregistered/nativeuntested. See [[Reviews/Soccer State Input Adapter Review]].


### Independently accepted private starter Mining checkpoint 2026-10-09T18:08:24.689609+00:00

Independentselectedcandidate review ../lab/reports/mining-starting-round-independent-20261009T1806Z/ executed85chains/12fallACKs/340modeledseconds,17checksalltrue,exit0. Pre-close6/6ore,9reshuffles,59200score,546000remainingms,340000wallms. PlainNextLevel→Won/OnGameWonMsg/completed1; GameOverend_senttrue; duplicateGameOverunchanged; CloseactiveNone/completed1. Author539-entrytranscriptmatchesexactly. Only3JSONleaveschanged; byte-identical24568d64candidate, baseline25370unchanged. Rootpreparedpackageacceptance sidecar is ../lab/reports/mining-playable-package-preflight-20261009T180011Z/independent-acceptance.json. Packagecreationmanifestpreservedhistorically. No native/reward/save/release/production changes from thischeck. Allcurrentdelegatedworkersfinishedandinterrupted. Rootnotes CurrentProgress/Tasks/Reviews updated; nextnativeMining/HarvestgatesremainopenpendingUIcontrol.


### Community Rust SOE transport assessment 2026-10-09T18:18:14.682367+00:00

User-suppliedsoe-protocol docs assessed against published2.3.0 and our existingtransport. Officialcrate checksum1e650ba4… matched; sourcecommitf1c01ad8. Private default/core source-identical tests76unit+8fragment/fuzz-style+1realUDP+1doc=86passed0failed. Manifest-onlyprivatecopy removal ofTokiodev dependency; originalallfeaturesblockedbyGNUdlltool/assemblerenvironment. NoFreeRealmscapture/nativecompatibilityclaim. Existingone-sessiongateway makesmultiplexer a useful futurecandidate. Selectiveretransmitopenissue andMITLICENSE/Cargo vsGPLREADME discrepancy recorded. Obsidianassessment notes/Research/SOE Protocol Rust Crate Assessment.md; evidence %USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\soe-protocol-assessment-20261009T181451Z/receipt.json. No productiondependency/source/save/release change; globalmemorynotupdated.


### Edenfps community reference review 2026-10-09T18:27:52.165960+00:00

Downloaded/inventoried full FR ADR Toolkit5acfdf8d and Sanctuary-PS3d1c747f6 plus PC2009b1182d67 into private lab. Full toolkit same pin;21 prior canonical source hashes match despite checkout CRLF. Fresh selected community-reader probe passes: Chatdy4256vertices/6317triangles/92bones/32slots; warpstone998vertices/978triangles;6original/copy inputs unchanged. Sanctuary skim finds version-specific serializers, fishing wire experiments and PS3 loading diagnostics; housing entry is commented out, Kart zero-blob experiment and hardcoded fishing catch are not accepted gameplay. No native/build/production changes. Evidence ../lab/reports/edenfps-reference-review-20261009T182429Z/receipt.json; Obsidian [[Research/Edenfps Toolkit and Sanctuary Branch Review]]. Explicit user request fulfilled by SOE dated evidence memory note at %USERPROFILE%\.codex\memories\extensions\ad_hoc\notes\2026-10-09T18-27-52Z-soe-protocol-reference.md; no registry files edited.


### Reward toast and deferred-bank investigation 2026-10-09T19:03:18.874782+00:00

User supplied3donor files preserved/hash-inventoried in ../lab/reports/reward-bank-research-20261009T185311Z. Native50 usesu16base/u8subtype; bundlebody1coins/5XP/13kind. Realx86 getters/readers provefields; templatebranchoneifonlyonepositive/twoifboth. Bank50/6addsXPword2/coinsword3;50/7clearsnativebankandsumsrowword2, callsShowXpCoinBankReward(stars,coins). Lua/GFXconfirmmembershippanelratherthanordinarytoast. Itemhelperkind16opensMysteryChestpanel; chestworldinteractionnotestablished. Final36executedCPU/Lua/donorchecks pass;3officialJPEXSexports104ASfiles. Donorlocalehasher85932IDs/firstlinesmatch,14multilinefulltexts truncated; firstfailedall-fulltextassumptionretained. Writeup [[Research/Reward Toast and Deferred Bank Findings]] ready for user sharing. No native input, production source/save/balance changes. Receipt %USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\reward-bank-research-20261009T185311Z\receipt.json.

### Mystery Chest Rust and original-client acceptance 2026-10-09T19:54Z

Four donor C# files preserved/hash-verified. Native42/16Check,42/15Use,38/30confirmation and50/1kind16item bundle resolved. Optional Rust loot bindings validate Type28 and key78137; staged consume+grant rollback, fresh-Check/replay/session guards and existing save-before-send path. No retail Type28 definition found; private reserved910100 reconstructs original chest name/icon/category408 and fixed Kangaroo11584 reward. Seven supplied IDs are reward definitions. Actual original x86 reader accepts111bodybytes/one entry/icon7665/name378; decompiler removed real entry loop, so assembly/392-instruction execution is retained.

Whole suite1549pass/0fail/70ignored, all4private chest gates separately green. Corrected16 baseline replay restores byte-identical original gameplay and fails missing confirmation; run after integration, initialwrong14RED qualified. Postgate change only corrects inaccurate codec documentation;25focused+4private rerun green. Native73d8b311... frozen. Firstnative194318Cancel2/2 unchanged, OkayKangaroo panel and inventory1/1/1, coins5/counter14:1623datagrams/1602gatewayCRCs/fullreassembly. Freshnative194943 inventory persists1/1/1, whole gameplay identical, no repeat reward:1531datagrams/1512gatewayCRCs/fullreassembly. Both bounded clients/services closed. Source/user saves preserved; test branch only. No mounting, explorer-chest or historical drop-table completeness claim. See [[Research/Mystery Chest Restoration]], updated Tasks/CurrentProgress, receipt ../lab/reports/mystery-chest-intake-20261009T192906Z/receipt.json.

### Delegated readiness and native checks, October9

Audited100unique workitems; lead checked93subsequent pieces' module/test/checker/note paths. Corrected initial audit's zero-testable claim, piece081 alias/callsite omission and overbroad native-successor counts. Focused receipts independently parsed: Housing121/0/3ignored +1private44bodycorpus; Fishing30/0; Kartcore64/0/3ignored +3privategateway; Soccer21/0/1ignored. Housingroutingprobe127 returnsnooutcome and canonicalSnapshot lackshousing. Racingonly102lobbyconnected; Fishing138 and Soccer76nativeflowsabsent. Findings returned internally to originaltasks, Housing025/026/028 and Racer041; fixesnotclaimed.

Originalclient73d8: Mining200750Lost13manual/unattributedchains5400/[0,0]/completed0; noquota/win/ordinaryClose. Harvest202011 nativeJobsChef/offer/board; unresolvedgoalnames/icons, noSkyendpointchainaccepted and Leaveattemptsfailed. DeveloperrealmexitTRYAGAIN/worldreturn thenExitGame; inventoryexactlyunchangedcoins309, onlyChefselection/instanceallocationchanged. Restart202622 wholegameplayequal; cancellationpersistenceonly. Finalcaptures5817/2664/1409gatewaydatagrams;5815/2662/1407CRC3/fullreassembly; allclients/login/gatewaystopped. Root verifierpasses100rows/93artifactsets/3nativecaptures/noeconomychange. Nofreshfullsuiteorproductionchangesclaimed. Evidence [[Reviews/Delegated Native Acceptance Audit]], lab/reports/delegated-native-audit-20261009T200750Z/lead-receipt.json and durableHarvestscreenshots.


### Five-worker repair dispatch, October9 21:17UTC

the owner reduced concurrency to five. Saved config, AGENTS and worker contract now agree; config backup retained outside repository. Five actual crack_builder assignments returned receipts and each status was checked once as running: type8_repair, housing_repair, kart_repair, fishing_repair, soccer_repair. No descendant spawning. Separate source/test/note/private-evidence ownership; lead reserves shared integration and native control. Baseline296Rust source/test fingerprints preserved. Lead registered existing fishing/housing/kart_racing modules; explicit library gate24passed/0failed/0ignored. Initial missing-manifest attempt failed before compilation and is preserved separately. No native-ready claim, release rebuild or deployment. See notes/Reviews/Five Worker Repair Round.md and lab/reports/five-worker-repair-20261009T211727Z/.

### Kart integration and native entry repair, October9 22:24UTC

See [[Reviews/Kart Playable Integration]]. Kart entry, solo three-lap authority, content-priced rewards and canonical save/reconnect/Close are integrated. Full Rust suite1651passed/0failed/75ignored; two additional private protocol finish/reward/reload tests pass. Registered local track31 price50coins/25RacerXP and3m trajectory gate volumes are reconstructed, not recovered historical values. Native whole-race acceptance remains pending.

Recovered16original geometry chunks and40model/material dependencies with CRC/length/SHA receipts; asset delivery supports raw.gcnk and original&names.221554 native rerun with recovered assets still stopped before33/9. Lead traced and independently recovered missing35/20 local actor mask0x401 from original racing capture15451/15496; return44446 resets1. Entry/return now send these flags using actualcharacterGUID. Missing-definition and scene-id worker hypotheses remain unproven.222147 native run is testing this fix. Source/content/capture receipts are under lab/reports/kart-playable-v2/v3/v4-20261009. No whole-race/native-payout claim yet; no usersave or originalarchive reset.

2026-10-09 kart integration follow-up: original41/114 duplicate ActivityIdclear, owned41/2 afterLoaded10, exact41/107readiness.8adapter/private tests pass. Corrected launcherbuildtarget osfr-gateway; v6 repeatedoldsourceddb1 therefore no entryfix proof. Nativewholefinishstillpending.

### Kart lane, boost and native capture review, October9 23:49UTC

Reconstructed oriented gate bands replace the private captured-path spheres; a second ten-metre driving line passes all93gates/three laps. Go rebases the remaining countdown safely at early server clocks; timeout clears the owned racing flag and returns to the original world pose without rewards. Registered starter track31 payout remains50coins/25KartDriverXP, with atomic banks/toast/rank/save/replay tests. All42focused checks pass, including4private gameplay gates. Fresh complete suite1663passed/0failed/77ignored. Separate asset-delivery regression failed then passed6Pythonchecks: caret-containing goldcup decal filenames now retain CRC validation. Native texture delivery is being rechecked in v12.

Nativev11 finalized normally:47,611frame hashes/47,592gateway post-session CRCs verified. Recovered first-application-byte/user-compression framing adds38,736unsequenced33/6updates (30compressed) that the prior reliable-only reader skipped. Runtime read-only probe proves racer/controller/owned record exist; original33/9 confirms the GUID is the player's, so rebinding it to1 is correct. No real checkpoint hits or full race occurred during tap-only testing. Timeout returned the original pose and preserved the complete inventory and XP map. Gate actor instances are present in16GCNKchunks; earlier absence claims were scan-scope errors. Exact retail trigger volumes and balances remain unrecovered.

Recovered C33/22 owned-u64 BoostAwardRequest and empty S33/23 BoostAwardResponse; native consumer adds one type5 entry to a three-slot boost queue. Registered replay-guarded server-clock refill windows (reconstructed20seconds), isolated from finish/economy awards. Field defaults preserve old snapshots. A parallel private-test save-name collision was repaired; missing-env and failed-collision logs are retained. v11 frozen87de7f91 binary did not contain boost/caret fixes. New candidate49317256 runs in a fresh disposable v12 package; whole native driving/results/reward/Close/restart gate remains open. User held-key help is pending because Sky exposes taps/chords only.
# Kart native boost acceptance, October10 00:02UTC

Finalized v12 run `client-baseline-20261009T234659Z` on release `49317256...`: all15,108 frame hashes and15,086 gateway CRCs verified;11,272 direct C33/6 frames remain stationary/control0. Four owned boost requests produced three empty responses and visible HUD rockets. Native gold-cup decal AssetReceived confirms the caret filename repair. Inventory/XP unchanged; no checkpoint/lap/result/reward acceptance. Normal Exit Game stopped every owned service. Evidence and source/log fingerprints: `../lab/reports/kart-playable-v12-20261009/lead-receipt.json`.

Independent boost review: fresh1 service,1 malformed-bundle and4 private gameplay checks pass. Lead corrected the review's RED interpretation: UnsupportedSubtype was returned before implementation, rather than a foreign GUID receiving a grant. Refill uses fixed20-second elapsed epochs, not a strict minimum delay between acknowledgements. Existing42 focused/1663 ordinary/6 Python gates remain accepted; no Rust logic changed after them. Native held-key drive/finish/reward/Close/restart and normal NPC/menu entry remain open. A bounded restart is being prepared for interrupted-race recovery and human driving; no user campaign save modified.
