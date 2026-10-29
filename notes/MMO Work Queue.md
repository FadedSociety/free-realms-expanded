# MMO work queue

**October9 lead readiness audit:** see [[Reviews/Delegated Native Acceptance Audit]] and [[Reviews/Delegated Task Readiness]] for current status of all100 assignments. Earlier rows below are dispatch/handoff history. Artifact-complete is not native-playable: housing/racing/fishing/soccer lack connected flows; Mining/Harvest have partial native tests and open whole-round gates. Findings were returned to original tasks. Do not use old row labels as a current acceptance certificate.

Updated October8 evening Pacific. **All100 assignments were accepted after relaunch**, including coordinator100. Receipts are independently counted in [[Reviews/Lead MMO Dispatch Snapshot]]. This is total dispatch, not100 simultaneously running or100 playable features. Workers return concrete code/tests/provenance; lead review, shared integration and native acceptance remain separate.

All workers follow [[Design/MMO Worker Contract]]. Each queued item receives an exclusive code/test or research path before dispatch. The lead alone changes shared integration files and controls the native client. Each item must pass focused tests, independent review and applicable native acceptance. Existing opening progress and preserved checkpoints remain separate from this backlog.

For items 001–006, owned Rust modules are `prototype/osfr-tools/src/{mining,harvesting,fishing,kart_racing,soccer,housing}.rs`, matching `tests/*.rs`, and the corresponding restoration research note. Every worker has private timestamped evidence output under the lab. Items007–099 have exclusive assignments in `notes/WorkItems/###.md`: one Rust module/test, one optional corpus checker/test, one note and private lab evidence. Root owns001–006 and028–048; coordinator100 owns007–027 and049–099.

| ID | Piece | Status / worker | Required evidence or acceptance |
|---|---|---|---|
| 001 | Mining original board/session | Reviewed repair; lead19+13 tests GREEN; integration/native pending | Recover Type8 board/messages; valid and invalid moves, progress, cancel, restart, bounded reward proposal. |
| 002 | Harvesting original game/profile | 17-test candidate; independent review resumed | Recover Farnum111 configuration; share Type8 engine with001; distinct goals/drop semantics. |
| 003 | Fishing | Reviewed codec; catch/reward authority/native pending | Recover Type21 cast/catch lifecycle; validated input, cancel/restart and result proposal. |
| 004 | Go-kart racing | Reviewed repair; lead13+12 tests GREEN; physics/integration/native pending | Track/checkpoint/lap order, timing and session validation; distinguish client vehicle physics from server scoring. |
| 005 | Soccer | Reviewed replay reducer; lead11+10 tests GREEN; authoritative physics/native pending | Recover team/ball/goal/timer lifecycle; bounded authoritative match and restart handling. |
| 006 | Housing placement | Reviewed repair; lead26 tests GREEN; wire/integration/native pending | Property ownership, furniture instance accounting, finite transforms, move/remove and persistence. |
| 007 | Demolition derby | Candidate reported; lead review/integration/native pending | Recover original arena/vehicle/damage/scoring data; prevent duplicate finish/reward. |
| 008 | Chess | Candidate reported; lead review/integration/native pending | Recover launch/result contract; legal moves, check/mate/stalemate and reconnect. |
| 009 | Checkers | Candidate reported; lead review/integration/native pending | Recover rules/config; compulsory captures, promotion, end state and replay protection. |
| 010 | Trading card game | Candidate reported; lead review/integration/native pending | Recover available decks/cards/protocol and coverage gaps before implementing a bounded evidenced rule slice. |
| 011 | Pet training activities | Candidate reported; lead review/integration/native pending | Recover tricks/commands/rounds and validated completion; no invented client animation claims. |
| 012 | Cooking recipes/content | Candidate reported; lead review/integration/native pending | Inventory recovered recipes, materials/results/locale; check atomic repeated recipe transactions. |
| 013 | Smelting recipes/content | Candidate reported; lead review/integration/native pending | Recover furnace definitions and materials; verify original movie payload and once-only outputs. |
| 014 | Forging recipes/content | Candidate reported; lead review/integration/native pending | Recover anvil patterns/equipment recipes; correct profile and slot restrictions. |
| 015 | Minigame launch catalogue | Candidate reported; lead review/integration/native pending | Map all783 recovered entries/22types/20groups to source, client mode and actual implementation coverage. |
| 016 | Minigame loss and retry | Candidate reported; lead review/integration/native pending | Original loss/retry packets and fresh instance IDs; no reward on loss or stale finish. |
| 017 | Minigame disconnect recovery | Candidate reported; lead review/integration/native pending | Recover semantics; conserve materials across disconnect, restart and abandoned sessions. |
| 018 | Minigame native test routes | Candidate reported; lead review/integration/native pending | Evidence-backed developer entry points for supported games; gates remain default-off. |
| 019 | Activity leaderboards | Candidate reported; lead review/integration/native pending | Recover score direction/ties/reward rules; persisted records without trusting arbitrary client scores. |
| 020 | Multiplayer activity grouping | Candidate reported; lead review/integration/native pending | Recover group/instance membership; join/leave/disconnect and invalid membership rejection. |
| 021 | Property acquisition | Candidate reported; lead review/integration/native pending | Recover property definitions and purchase/grant semantics; once-only ownership and currency accounting. |
| 022 | House loading and visiting | Candidate reported; lead review/integration/native pending | Recover zone/instance/entrance packets; owners and guests enter the correct persisted instance. |
| 023 | House permissions | Candidate reported; lead review/integration/native pending | Recover access/edit roles; owner/guest/blocked behavior and unauthorized edit rejection. |
| 024 | Furniture catalogue | Candidate reported; lead review/integration/native pending | Map asset/item/model/locale relationships and missing entries; validate stock versus reconstructed inventory. |
| 025 | Furniture placement client wire | Candidate reported; lead review/integration/native pending | Recover edit/commit/update/delete requests and server responses; native placement acceptance after006. |
| 026 | Housing inventory transactions | Candidate reported; lead review/integration/native pending | Placement/removal conserve owned furniture instances; rollback on persistence failure. |
| 027 | Housing fixtures and limits | Candidate reported; lead review/integration/native pending | Recover snap/rotation/build bounds/item limits; report unrecovered collision semantics. |
| 028 | Housing save migration | Candidate reported; lead review/integration/native pending | Versioned restore and invalid/old data handling; no loss of owned items or property. |
| 029 | Housing decoration discovery | Candidate reported; lead review/integration/native pending | Recover interactive furniture/decoration behavior; correct instance selection and safe effects. |
| 030 | House native acceptance | Candidate reported; lead review/integration/native pending | Enter, place, move, remove, visit, restart and verify through actual client once dependent pieces integrate. |
| 031 | Job catalogue | Candidate reported; lead review/integration/native pending | Recover every job/profile/localization/unlock mapping and explicit historical coverage. |
| 032 | Brawler progression | Candidate reported; lead review/integration/native pending | Original skills/XP/level/equipment definitions; validate unlock and reward boundaries. |
| 033 | Warrior progression | Candidate reported; lead review/integration/native pending | Recover job/skill definitions; ownership and level restrictions, not copied Brawler assumptions. |
| 034 | Ninja progression | Candidate reported; lead review/integration/native pending | Recover skill/equipment/cooldown rules and bounded server validation. |
| 035 | Archer progression | Candidate reported; lead review/integration/native pending | Recover ranged actions/ammo/projectile behavior; range and line-of-sight evidence. |
| 036 | Wizard progression | Candidate reported; lead review/integration/native pending | Recover spells/resources/effects; reject invalid targets and repeated casts. |
| 037 | Medic progression | Candidate reported; lead review/integration/native pending | Recover healing/revive/resource definitions; correct friendly/dead target rules. |
| 038 | Adventurer progression | Candidate reported; lead review/integration/native pending | Recover exploration/collection unlock rules; preserve profile-specific progress. |
| 039 | Gathering job progression | Candidate reported; lead review/integration/native pending | Original Miner and Farmer XP/rank/unlock definitions; integrate001/002 without duplicate grants. |
| 040 | Crafting job progression | Candidate reported; lead review/integration/native pending | Chef and Blacksmith recipe/rank unlocks; preserve current accepted crafting behavior. |
| 041 | Racer and derby progression | Candidate reported; lead review/integration/native pending | Recover job/rank/vehicle unlock definitions; link verified game outcomes to progression. |
| 042 | Soccer and postman progression | Candidate reported; lead review/integration/native pending | Recover job XP/unlocks/tutorial definitions and source coverage before content binding. |
| 043 | Pet trainer and card progression | Candidate reported; lead review/integration/native pending | Recover profile/unlock data; link only validated activity outcomes. |
| 044 | Class switching wire | Candidate reported; lead review/integration/native pending | Recover switch/update packets; equipment/health/skills preserved and invalid profile rejected. |
| 045 | Cosmetic equipment | Candidate reported; lead review/integration/native pending | Appearance/slot/ownership validation; actual native equip and restart persistence. |
| 046 | Consumables and buffs | Candidate reported; lead review/integration/native pending | Native use request, quantity/effect/cooldown and icons; once-only consumption and expiration. |
| 047 | Gear attributes | Candidate reported; lead review/integration/native pending | Recover item stats and profile restrictions; no fabricated balance values hidden as historical. |
| 048 | Ability bar | Candidate reported; lead review/integration/native pending | Original action binding/hotbar packets; persist only owned available actions. |
| 049 | Character appearance coverage | Dispatched: coordinator piece_049; consult live ledger for completion | Recover remaining supported race/sex/hair/skin/outfit combinations and invalid asset exclusions. |
| 050 | Character naming/select lifecycle | Dispatched: coordinator piece_050; consult live ledger for completion | Duplicate/invalid name rules and saved appearance; complete native select/delete behavior only with evidence. |
| 051 | Enemy catalogue | Dispatched: coordinator piece_051; consult live ledger for completion | Original models/factions/skills/spawn data with explicit missing bindings. |
| 052 | Enemy patrol and idle | Dispatched: coordinator piece_052; consult live ledger for completion | Recover movement/animation contract; bounded patrol and actual native idle/run transitions. |
| 053 | Aggro and leash | Dispatched: coordinator piece_053; consult live ledger for completion | Target acquisition, pursuit, disengage/reset and invalid/out-of-zone targets. |
| 054 | Melee attacks | Dispatched: coordinator piece_054; consult live ledger for completion | Preserve reviewed3m melee rule; original timing/animation evidence and far-hit rejection. |
| 055 | Ranged enemy attacks | Dispatched: coordinator piece_055; consult live ledger for completion | Recover cast/projectile packets, hit timing/range and source-defined exceptions. |
| 056 | Combat status effects | Dispatched: coordinator piece_056; consult live ledger for completion | Source-defined stacking/duration/immunity and restart/disconnect behavior. |
| 057 | Loot and enemy rewards | Dispatched: coordinator piece_057; consult live ledger for completion | Original drop tables where present; once-only kill/loot grants and eligibility. |
| 058 | Death and revive | Dispatched: coordinator piece_058; consult live ledger for completion | Native KO/revive/location/health handling; no reward exploit through repeated terminal messages. |
| 059 | Dungeon definitions | Dispatched: coordinator piece_059; consult live ledger for completion | Inventory recovered maps/encounters/entrances/difficulty with historical coverage. |
| 060 | Dungeon instance lifecycle | Dispatched: coordinator piece_060; consult live ledger for completion | Membership, spawn/reset, completion, exit and restart/disconnect invariants. |
| 061 | Dungeon waves and gates | Dispatched: coordinator piece_061; consult live ledger for completion | Original wave/trigger/door evidence; ordered completion and no bypass from arbitrary client events. |
| 062 | Boss behaviors | Dispatched: coordinator piece_062; consult live ledger for completion | Recover one original boss's phases/abilities; evidence-backed timing and actual animation acceptance. |
| 063 | Dungeon party rewards | Dispatched: coordinator piece_063; consult live ledger for completion | Group eligibility, shared loot/completion and duplicated/replayed finish rejection. |
| 064 | Dungeon collision/reachability | Dispatched: coordinator piece_064; consult live ledger for completion | Community model/terrain readers and real client traversal; distinguish render bounds from collision proof. |
| 065 | Dungeon native replay | Dispatched: coordinator piece_065; consult live ledger for completion | Full fresh entry, combat, boss, rewards, exit and reconnect using original client after integration. |
| 066 | Historical quest catalogue | Dispatched: coordinator piece_066; consult live ledger for completion | Recover quest/objective/prerequisite/NPC/reward sources; missing original bindings remain explicit. |
| 067 | Quest prerequisite engine | Dispatched: coordinator piece_067; consult live ledger for completion | Source-defined requirements, branch availability and invalid acceptance rejection. |
| 068 | Quest event routing | Dispatched: coordinator piece_068; consult live ledger for completion | Typed talk/visit/kill/gather/craft/game events; matching original identifiers and no unrelated kill credit. |
| 069 | Quest turn-in transactions | Dispatched: coordinator piece_069; consult live ledger for completion | Atomic quest completion/item/currency/XP; repeat, crash and stale request cases. |
| 070 | Quest journal wire | Dispatched: coordinator piece_070; consult live ledger for completion | Replace remaining captured fields with writer-verified serialization; native labels/progress/removal. |
| 071 | NPC dialogue branching | Dispatched: coordinator piece_071; consult live ledger for completion | Original dialogue/menu/choice definitions and quest phase handling. |
| 072 | Starting area historical coverage | Dispatched: coordinator piece_072; consult live ledger for completion | Compare working17-stage local chain with original tutorials/quests; identify and restore evidenced omissions. |
| 073 | Discoveries and exploration | Dispatched: coordinator piece_073; consult live ledger for completion | Recover trigger locations/radii/rewards; once-only events and native visit validation. |
| 074 | Collections and collectibles | Dispatched: coordinator piece_074; consult live ledger for completion | Original set/item/location/reward definitions; collection completion and restart persistence. |
| 075 | Postman delivery routes | Dispatched: coordinator piece_075; consult live ledger for completion | Original route/order/time/NPC interactions; validated delivery and completion. |
| 076 | Regional NPC and spawn content | Dispatched: coordinator piece_076; consult live ledger for completion | Source-backed actor placements/zone membership; cross-zone identifier and asset validation. |
| 077 | Warpstones and travel | Dispatched: coordinator piece_077; consult live ledger for completion | Original unlock/destination/request packets; locked destination and malformed request rejection. |
| 078 | Zone transitions | Dispatched: coordinator piece_078; consult live ledger for completion | Correct unload/load/spawn and persisted location; no stale prior-zone actions. |
| 079 | World terrain readers | Dispatched: coordinator piece_079; consult live ledger for completion | Run community tools on preserved assets; demonstrable format gaps only before custom readers. |
| 080 | World collision data | Dispatched: coordinator piece_080; consult live ledger for completion | Recover native/community collision representation and compare against actual movement samples. |
| 081 | Inventory instance lifecycle | Dispatched: coordinator piece_081; consult live ledger for completion | Existing item instances/counts/stack limits; atomic add/remove/merge and no ID reuse. |
| 082 | Inventory search crash | Dispatched: coordinator piece_082; consult live ledger for completion | Preserve fault evidence; controlled native reproduction and narrow root-cause fix before acceptance. |
| 083 | Shops and purchases | Dispatched: coordinator piece_083; consult live ledger for completion | Source catalogue/prices/request packets; stock/ownership/currency validation and rollback. |
| 084 | Selling and buyback | Dispatched: coordinator piece_084; consult live ledger for completion | Original vendor rules and transaction semantics; no duplication across replay/restart. |
| 085 | Player trading | Dispatched: coordinator piece_085; consult live ledger for completion | Recover offer/accept/cancel wire; atomic two-inventory transfer with disconnect and stale offer checks. |
| 086 | Currency and economy audit | Dispatched: coordinator piece_086; consult live ledger for completion | Enumerate actual currency types and sinks/sources; invariant checks across all reward producers. |
| 087 | Friends and presence | Dispatched: coordinator piece_087; consult live ledger for completion | Original roster/online status/requests; identity and permission validation. |
| 088 | Chat channels | Dispatched: coordinator piece_088; consult live ledger for completion | Original local/party/private channel routing; bounded message parsing and correct recipients. |
| 089 | Party lifecycle | Dispatched: coordinator piece_089; consult live ledger for completion | Invite/accept/leader/leave/disconnect; membership invariants and native panels. |
| 090 | Guild lifecycle | Dispatched: coordinator piece_090; consult live ledger for completion | Recover supported guild protocol/data before bounded create/member/role persistence implementation. |
| 091 | Pets and companions | Dispatched: coordinator piece_091; consult live ledger for completion | Recover ownership/spawn/follow/interaction definitions; native behavior and preserved pet state. |
| 092 | Mounts and vehicles | Dispatched: coordinator piece_092; consult live ledger for completion | Recover mount/dismount/speed/ownership requests; valid state transitions and native visuals. |
| 093 | Emotes and social actions | Dispatched: coordinator piece_093; consult live ledger for completion | Original action/animation IDs and restrictions; actual client acceptance with no guessed motion. |
| 094 | Reliable transport stress | Dispatched: coordinator piece_094; consult live ledger for completion | Loss/reorder/rollover/session replacement under several clients; bounded queues and exact delivery. |
| 095 | Multi-character persistence | Dispatched: coordinator piece_095; consult live ledger for completion | Remove single-character assumptions with reversible migration; isolation and restart proofs. |
| 096 | Typed world initialization | Dispatched: coordinator piece_096; consult live ledger for completion | Replace captured templates incrementally with unchanged-writer vectors and exact native compatibility. |
| 097 | Asset reference validation | Dispatched: coordinator piece_097; consult live ledger for completion | Catalogue references resolve to preserved assets/locale/model; report missing or conflicting entries. |
| 098 | Rust rendering/client feasibility | Dispatched: coordinator piece_098; consult live ledger for completion | Small evidence-backed terrain/model/animation loading slice; separate future replacement client from native compatibility. |
| 099 | Multi-client integration harness | Dispatched: coordinator piece_099; consult live ledger for completion | Deterministic launch/log/state comparison; party/trade/activity cases without concurrent shared UI control. |
| 100 | Restoration acceptance dashboard | Dispatch complete; supervising049–099 | Obsidian ledger links source, tests, review and native gates for each piece; never promote code-only work to playable. |

## Dispatch order and shared dependencies

Next available slots:031 job catalogue,066 quest catalogue,079 world readers and097 asset reference validation. Catalogue/research pieces can proceed without live UI. Integration follows reviewed module interfaces. Items002 and001 share the Type8 engine; only001 writes its generic engine. Item025 depends on006/022 evidence;030 depends on reviewed021–029. Combat/dungeon gates depend on recovered source bindings; a plausible reconstruction cannot close historical coverage. Activity outcomes pass through the lead-owned atomic economy layer.

## Lead continuation checkpoint

Developer teleport refinement has code review and bounded native acceptance, release `ec43e1de…`. Ordinary suite187 passed,0 failed,36 ignored; seven private developer teleport integration checks pass when run explicitly. Original Queen checkpoint remains immutable. The stage13 branch has gathered four Rainleaf/four Gummyfruit and the prior accepted quest materials. The owner manually played all six Cooking rounds and reported that it works well. This fresh run exited before final GameStats/victory confirmation persisted: no new stew grant or stage13 completion is established. Continue with a fresh game instance and actual completion/turn-in; do not fabricate final packets or edit rewards.

Active workers received required skill directives through collaboration messages and project instructions. Soccer repeatedly ended early with future-tense final text and was explicitly interrupted/restarted from its existing evidence. Its continued work is not accepted until actual artifacts and tests are returned. Housing started after fishing first returned; derby and activity catalogue filled subsequent finished slots. Lead fishing review requires actor binding, finite geometry validation and corrected provenance/integration instructions before registration. After a Codex reload, inspect worker status and attempt the sixth spawn before declaring that the100 limit is effective. Dispatch remaining workers from this ledger with exclusive paths and clear tests.
