# Opening playthrough

Status: active implementation and recovery. Requested by the owner after the first local gameplay milestone. This note is the acceptance contract and links implementation/review evidence; it is not a completion claim.

## Requested experience

Start a new character, play the starting area and its quest/discovery/cosmetic/class progression, enter a complete dungeon, defeat its boss, receive rewards, return to the world and retain progress after restarting. Use the supplied original Windows client with Rust-owned server behavior. Preserve original inputs and existing milestone saves.

## Acceptance gates

- [ ] Audit and enumerate the selected original starting area, its quests/discoveries and the beginner dungeon. Separate confirmed content from unknown coverage.
- [x] Create a named/customized character through a player-facing flow; customization and fresh progression persist (supported appearance, run072258).
- [x] Enter the starting area with appropriate initial inventory/class state rather than the old all-items research character (run072258).
- [ ] Complete the entire enumerated starting quest chain through player actions, including dialogue, objectives, turn-in and rewards.
- [ ] Discover the enumerated exploration/collection content and retain records without repeat farming.
- [ ] Unlock/use appropriate classes and earn/equip cosmetics through progression.
- [x] Travel into a real dungeon scene; progress through enemies and its boss with damage, defeat/recovery and completion state (reconstructed cave411, native run081219).
- [x] Receive dungeon rewards once per supported completion and return safely to the starting area (native run081219; independent wire/video audit).
- [ ] Restart Rust and the client; restore the new character, appearance, class, quest/discovery records and rewards.
- [ ] Independent review of transitions, reward duplication, persistence and unsupported-content claims.
- [ ] Full actual-client playthrough recorded from creation through dungeon return; resolve blocking failures before handoff.

## Work and reviews

- [[Research/Character Creation Audit]] — login/creation/appearance compatibility evidence.
- [[Research/Starting Area Content Audit]] — original quest/discovery catalog and recovery gaps.
- [[Research/Starting Dungeon Audit]] — dungeon scene, classes, encounters, boss and completion evidence.
- [[Research/Gameplay Coverage]] — previous supported behavior and known limits.

Maintain dated progress/review entries here and in MODLOG. A synthetic test or server-only simulation does not satisfy a native playthrough gate. Missing original rules may need explicit reconstruction, with provenance and limitations recorded rather than implied fidelity.

## Progress journal

2026-10-08 UTC: began the three parallel content/protocol audits and traced the existing gateway/gameplay/save seams. Existing baseline is preserved separately. No opening-playthrough gate is accepted yet.

2026-10-08 06:50 UTC: recovered 17 locally bound steps/51 objectives, 27 friendly/resource actors and four local discovery records around the original later-era Briarwood/Farnum/Cobblestone story. Added fresh seven-item inventory, class selection/unlocks, typed dungeon/world encounters, scene transitions and unified restart persistence. The packet-driven integration completes all 17 steps and the cave, restarts between steps and after the win, and checks reward/discovery duplication. This is a server-only simulation, not the required native playthrough. Independent review found and prompted fixes for finite wolves spawning before their prerequisite goals and missing Medic/Archer starter definitions/Archer equipment slot.

The original creator now renders and reaches naming after community-manifest-verified dressing-room terrain and the missing starting-outfit table were installed in the disposable copy. Native submission then stalled; the actual captured request proved that this client uses default server ID0. The local service incorrectly accepted only1. A focused replay now creates/saves/reloads that actual request with the corrected0/1 routing. Native submission/world entry must be retested.

[[Research/Wiki Research Database]] is complete: Scrapling0.4.15 acquisition of 3,250 preserved pages, 3,996 chunks and actual sqlite-vec384-dimensional vectors. Exact wiki revisions corrected the wolf count to five and weapon/wolf rewards to16/18 coins. Missing hammer/material definitions and original crafting minigames remain explicit gaps; local resource/craft interactions are not original minigames. Native quest journal replacement/incremental updates are being recovered before the native opening acceptance run.

2026-10-08 07:05 UTC: native creator now completed race/gender, naming, appearance and confirmation for test character Bennett Eaglebeach (Human male, Shaggy hair, Pearl skin, Skull face paint, Red Hoodie). Actual Rust login logs show create5 succeeded and login7 selected; identity and fresh ten-item snapshot persist. Run `client-baseline-20261008T065152Z` then exited cleanly before world entry: original head texture08 is absent from supplied packs, causing `Failed first profile`. This is an asset blocker, not accepted gameplay or a diagnosed access violation. Recovering the exact CRC-matching texture from public community packs; no appearance substitution.

Native journal now replaces unrelated research quests and uses opcode49 incremental updates; independently compiled supplied serializer matches the entire patched self payload. Basic-only Brawler/Archer/Medic hotbars match community packets; original skill effects remain unimplemented. Added native XP/rank updates and fixed inactive starter equips changing the visible job. Full Rust suite passes107 ordinary tests; focused private packet simulations/oracles also pass. The native playthrough remains outstanding.

[[Research/Native Asset HTTP Delivery]] records the verified8-byte native `.z` header and zlib stream. Loopback delivery now indexes103,097 supplied raw assets that match the original manifest CRCs, checks source hashes/CRC and bounds, and preserves delivery proofs. A physical-pixel DPI correction to the private Universal Modder input helper makes clicks agree with GPU captures; originals/tools are preserved and future inputs are recorded per run.

2026-10-08 07:25 UTC: completed actual second TESTcreation as Capo Indigocauldron (Human male, Long hair, Gold skin, Skull paint, Blue Tee Shirt). Preserved Bennett identity/save unchanged. Community-public SkinTone/FaceType tables match launcher manifest XXH64 and remove unsupported skin7/8 choices; all male caucasian1..6 raw textures match original manifest CRCs. Native run072258 reached World Ready and Initialized-Client using new releaseSHA5063442e993cae0b7af2d69223b6b6885f03a5c24d5713c40311d8ad64bcb52d. Fresh10items, firstdiscovery1,5coins/25AdventurerXP persisted; native Jobs switchedBrawler43 with changed equipment. Quest interaction/dungeon acceptance still in progress. Independent HTTP review passed path/CRC/envelope/cache checks and found a cleanup metric-snapshot race; root applied a metrics lock for future runs. The already-running instance predates that correction.

2026-10-08 07:58 UTC: native run073831 completed Hero's Journal through player interaction and turn-in (saved counts [1,1], rewarded true), accepted The Last Best Hope, zoned into the original sg_npx_cavern_01 scene and registered all three Gloamling defeats. The user reported stationary enemies, missing attacks and overly long apparent hit range; the missing boss ADR was also visible in logs. The character then lost with the boss at 500 HP. This is failed combat acceptance, not a completed dungeon.

Investigated original readers and pinned community writers before patching. Native opcode125 rotation is a horizontal direction (X/Z mask; atan2(X,Z)), despite the community Quaternion type name. The NPC template already used MovementType2; its Speed0 and animation IDs -1, movement State0 and absent opcode35/8 animation messages were inadequate runtime defaults. Added Speed3, Walk2/Run3/Stand1, heading with State1, bounded pursuit at three units per second, explicit run/stand/attack animation messages, a one-second enemy attack cooldown and melee limits of three player units and 2.5 enemy units. Independent review caught f32 rounding at the stopping boundary and running animations that continued after leaving aggro range; actual-coordinate regressions cover both fixes. Guide interaction remains twelve units. These combat rules are reconstructed.

Recovered the exact gloam_m_boss.adr from the public native CDN, raw CRC3361679076, SHA-256 ac842259cb0d62648e8c4083df6e1edf7d381d8d6681185ecd8a7ff830c477ba. Its 3,577-byte wire response matches the original manifest. Core dependencies are verified; spawn/despawn clips and six terrain textures remain absent. Release498fb4ee launched run075754 from the saved loss checkpoint. [[Research/Native NPC Motion and Animation]] contains native code and serializer proof; [[Research/Native Creator Delivery Proof]] contains full serializer comparisons.

2026-10-08 08:20 UTC: [[Reviews/Native Combat Motion Retest075754]] independently confirms visible boss approach and attack poses, all 256 State1 movements with horizontal headings, and 100 run/stand/attack messages. Both retries still damaged the player roughly one second after loading, before voluntary control. Added a connection-local preparation gate: the cave waits for nearby Darkthorne's Start option or the first valid attack. Reload and retry disarm without erasing saved progress. Nine focused tests and the private encounter and 17-step packet simulations pass; independent review found no blocking issue. Run081219 visibly renders Start and retains full health while waiting. Combat and normal action-bar controls are still under native test; the dungeon gate remains unchecked.

2026-10-08 08:44 UTC: [[Reviews/Native Cave Completion 081219]] independently accepts the actual campaign cave fight and return: eleven hits defeat three Gloamlings and the boss; player survives at280 HP; fifty coins and one food item are granted once; Exit restores zone0/full health and updates The Last Best Hope to [1,0]. Recorded preparation waits up to152 seconds cause no damage or movement. Measured successful melee distances are1.970–2.434 units. Unit tests separately cover distant rejection.

Normal action-bar testing found two further errors: MiniGameType1 disabled native combat targeting, and the server expected a job ID where the client sends bar category1. The original basic requests self GUID1. Corrected combat type4 and bar1/slot0 handling with server-side selection of one nearby live enemy; original multi-target sweeps remain unrecovered. A separate synthetic controls branch in native run083946 wins with key1 and660 HP. It retains the already-rewarded flag and grants nothing again; the real campaign save is byte-identical throughout that branch. Full starting-area testing and native crafting remain pending. [[Research/Original Smelting]] records the integrated original movie/protocol and explicit temporary harvest drops.

2026-10-08 09:29 UTC: native campaign run090058 follows Darkthorne's recovered magic-door story transition to Farnum's Farmstead. Pappy's actual interaction completes The Last Best Hope and grants25 coins; discovery2 grants5 coins, bringing the wallet to100. Saving Farnum's Farm is accepted with all four objectives pending. The typed journal renders the completed defense and pending Pappy objective after restart in084544 and, after the StateID411 cleanup correction, without restarting in the separate091836 win branch. The original results modal remains unresolved.

Farm play exposed missing well/fire visual dependencies despite valid terrain placement. Verified original alternate models3519/204 and a third-fire height correction are prepared in a new private content version, with explicit provenance and persistent fire removal. This is not yet native farm acceptance. The same version integrates original Smelting77 and Forging139 with server-owned completion and atomic materials/rewards. Campaign migration will preserve all existing progression and identity in a new save; the original content/save remain available.
