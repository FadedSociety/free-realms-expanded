# Integration Round 5 (W62)

Date: 2026-10-13. Status: code and automated gates pass; native in-game acceptance remains for the lead.

## Implemented

- **Dialogue:** NPC Talk now sends a 26/3 with one localized Goodbye response. Selecting 26/6 clears the active talk state and replies with the empty 26/4 EndDialog; an inbound 26/4 also clears talk state. Closing dialogue does not close an active vendor shop. Byte tests cover the 26/3 row, 26/6 response and 26/4 packet.
- **Regional quests:** `QuestEngine::from_split_content` loads the split quest, NPC and job files with size/schema bounds and cross-reference checks. NPC menus combine explicit `quest_ids_given` with quest definitions whose `giver_npc_id` matches that NPC's content key. Accept/decline 49/2 and abandon 26/0x17 enter the quest engine. `GameplayState` persists and restores that state. Quest title IDs are preserved as raw u32 bits, including values above `i32::MAX`.
- **Vendors:** vendor menus include Shop for vendor-role NPCs, and 66/42 uses `vendors.json` runtime stock. Purchases/sales use Coin Store 165/4 and 165/5, updating the persisted inventory and wallet. 66/43 is bounded, open-shop-gated, parsed and logged only.
- **Movement:** opcode 125 is decoded and validated before character position is committed. Teleports, relocations and zone changes create one-use discontinuity tokens; stale reports are ignored, rejects snap back with the existing location message, and soft overages are clamped. `movement` is public.
- **Recipes:** learned numeric IDs are in the character gameplay save. Login sends 0x3f/op5 ReplaceAll; consuming a learned pattern can send op1 Add. Cooking, smelting and forging start through `begin_recipe` with selected recipe key, job rank, learned status and current inventory.
- **Spawn filter:** load-time filtering skips the three extra `knocker_m_boss.adr` rows because the preserved client catalog has no `knocker_m_boss.dme`. The authored rows remain in content.
- **Save migration:** gameplay snapshots now write schema 7. Older schemas deserialize with defaults for NPC quest and learned-recipe state; migration tests preserve existing character fields and instance counters.

## Recovered evidence and reconstructed choices

Recovered client facts used here:

- Native dialogue reader `00a9ef10` consumes a 93-byte base plus 20 bytes per response; response 26/6 has one i32 (`009f5890`), and 26/4 EndDialog has no body (`009f5760`). The native capture scan in `Research/NPC Interaction.md` found no 26/3 frame. Locale export row 8075 is `Goodbye`. The original Lua quest end screen uses response action tuple `(300,17,3)` (`Research/Pieces/071 NPC dialogue branching.md`, `QuestHandler:AddDialogResponseNodes`).
- Quest offer 49/1 is read at `00c7bb60` and handled at `00a94be0`; 49/2 carries an i32 quest ID and u8 accept flag (`00a97d70`, `00a97e10`, constructor `00a86400`); QuestAdd 49/3 uses `00c7cc80`/`00c7ccd0`; abandon is command 26/0x17 (`00c09f60`, `00c0a500`). See `Research/Quest Engine.md` and `Build Guides/Gameplay/Quests and Objectives.md`.
- Merchant dispatcher `00b48a10` routes 66/42 list parsing through `00b40a60`; 66/43 is a distinct request with unresolved fields and transaction semantics. Coin Store 165/4 and 165/5 have the documented buy/sell layouts. See `Research/NPC Interaction.md`.
- Opcode 125 is a 36-byte message read at `008e5940`. State bit 4 is the quarter-speed selector (`00934a20`, `0090e0f4`, `008b3180`); other state bits are not assigned movement modes. See `Research/Movement Validation.md`.
- Recipe message family 0x3f dispatch is at `009471a0` to `00a12740`/`00c8d170`. Recipe pattern Param1 is the numeric recipe ID (`0103cc80`, `0103ace0`, `0103bbf0`); pattern items support IDs 70 and 11. No baseline recipe packet was recovered; wire tests are synthetic. See `Research/Recipe Content.md`.
- The client asset catalog lists `knocker_m_boss.adr` without a `knocker_m_boss.dme`; native round 4 records that these spawns fail NPC creation in `Reviews/Native Round4 NPCs 035300.md`.

Reconstructed server choices, marked `(R)` in code where applicable:

- Goodbye's response tuple reuses the recovered end-screen `(300,17,3)` action with locale label 8075; the tuple meaning is not independently decoded from native bytes. No original 26/3 opening-flow packet was available, so the 26/3 response row is synthetic; 26/4 bytes are native-reader-backed.
- Quest membership, player level, active-quest limit and reward display defaults are local policy. NPC/quest IDs are joined by exact content key; this extends incomplete NPC `quest_ids_given` rows. From the current split files, 403 quest IDs have explicit NPC links, 33 more are recovered by the matching typed giver ID, and 215/651 quest definitions have no matching NPC content key and cannot appear in an NPC menu. Opening quest behavior remains on its existing path.
- Coin Store prices, stock eligibility and transaction state use the existing local CoinShop policy. 66/43 remains non-transactional because its client field meanings are unresolved.
- Movement speed, clamp/reject thresholds, discontinuity radius/timeout and base speed are server policy; only the bit-4 scale is client-derived. The validator replays all available captures without rejects or clamps.
- Recipe rank curve, missing recipe wire presentation defaults and learning-on-pattern-consumption are local policy. Opening saves seed IDs 11, 70 and local Copper sample 2002 to preserve the existing tutorial flow; this is not a recovered retail known-recipe state.
- Only the proven broken Knocker model is filtered. A same-stem `.dme` rule is not applied to every ADR because many catalogued ADRs do not use a same-stem DME; their actual base bindings are not established by this task.

## Verification

All Rust commands ran from `project/prototype/osfr-tools` through `project/scripts/cargo.ps1`, with `CARGO_TARGET_DIR=lab/build/W62` except the release build. Scoped formatting used the private `rustfmt.exe --edition 2024` on changed Rust files only.

Focused commands: `test --test npc_interaction --test quest_engine` passed 13 and 9 tests; `test --test recipes_wire --test world --test content_npc_spawns --test game_store --test save_schema --test cooking_gameplay --test smelting_gameplay --test forging_gameplay` passed every target listed below. The final `test --all-targets --quiet` run exited 0.

| Target | Result |
| --- | --- |
| `cargo test --all-targets --quiet` | Exit 0; all test targets passed. |
| `npc_interaction` | 13 passed. |
| `quest_engine` | 9 passed. |
| `movement` | 10 passed; corpus replay processed 14,982 updates across 99 captures, with 0 rejects and 0 clamps. |
| `gateway` | 9 passed, 2 private tests ignored in the ordinary full-suite run. |
| `recipes_wire` | 5 passed. |
| `world` | 8 passed. |
| `content_npc_spawns` | 1 passed. |
| `game_store` / `save_schema` | 2 / 5 passed. |
| `cooking_gameplay` / `smelting_gameplay` / `forging_gameplay` | 1 / 1 / 1 passed; 2 / 1 / 3 private-content tests ignored. |

The required private startup oracle passed with `OSFR_GATEWAY_REFERENCE=lab/references/rust-gateway-reference` and `OSFR_GATEWAY_CAPTURE=lab/runs/client-baseline-20261008T020205Z`: `cargo test --test gateway '--' '--ignored'` ran 2 tests, 2 passed, 0 failed, 9 filtered out. PowerShell requires the separator to be quoted when forwarding it through the wrapper.

The default-target release command `cargo build --release --bin osfr-gateway --bin osfr-login` passed. It produced `target/release/osfr-gateway.exe` (12,722,890 bytes) and `target/release/osfr-login.exe` (12,029,571 bytes).

The initial full-suite run found four stale save-schema expectations (old versions 5/6); tests now assert `GAMEPLAY_SCHEMA_VERSION`, and the final all-target rerun passed.

## Native test plan for lead

1. Talk to a regional NPC, select Goodbye, and confirm the dialogue camera closes; verify any client close request observed during the flow is accepted.
2. Take a quest from a regional giver, accept it, relog and verify it remains in the journal; also exercise decline and abandon.
3. Open a vendor, inspect 66/42 stock, buy with 165/4, sell with 165/5, relog and verify wallet and inventory persistence.
4. Teleport or relocate, then walk normally with validation enabled; confirm stale transition reports are ignored and ordinary movement is not rejected or clamped.
5. Learn a recipe, relog and verify ReplaceAll restoration, then craft with an eligible rank and inventory.

Native client control was excluded by the worker contract, so these acceptance steps have not been performed here. Regional quest objective progression and 49/13 and 49/14 turn-in routing also remain outside this integration slice.

# W63 - native-round-five fixes

Date: 2026-10-13. Status: code and requested automated gates pass; native UI acceptance remains open.

## Implemented

- **Developer teleport:** `realm goto <name|guid>` combines opening actors with the content NPC visibility index (regional `legacy_data` rows and renderable extra spawns). Name lookup is case-insensitive, prefers an exact match, then accepts only a unique prefix. GUID lookup is supported; ambiguous replies show at most ten names and GUIDs. `realm goto list [filter]` accepts a case-insensitive substring filter. Duplicate runtime GUIDs keep the opening descriptor. The developer command arrives 2 m from the target facing it, updates content NPC visibility immediately, and returns the relocation so the existing gateway adds the one-use developer-teleport discontinuity token. Name matching, 2 m placement, candidate deduplication, and missing-name fallback are reconstructed developer-command policy (R); no collision or floor query is performed.
- **Dialogue close choice:** the 26/3 choice row is now `[300, 17, 8075, 0, 0]` (little-endian dwords). The display string occupies dword 3; the two image parameters are zero. Dwords 1 and 2 preserve the Lua tuple values but their response/opaque meanings are not established from a native packet, so that mapping remains reconstructed (R).

## Recovered evidence and limits

- Native reader `00a9ef10` dispatches the dialogue response list through `00a9d760`, whose 20-byte row reader is `00a9b480`. Native display routine `009f5ae0` resolves row dword 3 as text and passes dwords 5 and 4 to `SetChoiceText`; `lab/decompiled/client-ui/ScriptsBase.lua:12511-12513` confirms the order as `(bgImageSetId, imageId, text, 0)`. The string resolver at `0097bca0` uses `Global.Text.<id>`. Locale export `lab/reports/locale-id-to-text.json` maps 8075 to `Goodbye` and 17 to `Chef Tools`. Choice activation at `009f6aa0` sends row dword 1 in 26/6. These facts identify why W62's `[8075,300,17,3,0]` rendered Chef Tools with an image.
- The original quest-end Lua tuple `(Ui.GetString("questThanks"),300,17,3)` is recorded in `Research/Pieces/071 NPC dialogue branching.md`. The native row reader establishes the display and image slots, but no captured packet establishes how the tuple's remaining values map into the server's first two row dwords.
- Native-map inspection commands, from the project root: `python -I scripts/native_map/nmq.py dis 00a9ef10 384`; `python -I scripts/native_map/nmq.py show 00a9ef10`; `python -I scripts/native_map/nmq.py show 009f5ae0`; `python -I scripts/native_map/nmq.py show 009f6aa0`; `python -I scripts/native_map/nmq.py show 0097bca0`. These query `lab/native-map/run/map.sqlite`; no disassembly output was copied into the repository.
- A scan of 14 `.pcap`/`.pcapng` captures under `lab/references/OpenSourceFreeRealms/ofrserver/Packets` found no 26/3 signature in TCP/UDP application payloads. The 1,661 binary frames from `lab/runs/client-baseline-20261014T050310Z/frames` also contain no 26/3 row. The previous scan in `Research/NPC Interaction.md` covered 137,912 application frames with the same result. The current dialogue byte test is therefore synthetic, not a native packet parity test.
- Content NPC names and positions come from the checked-in content rows: regional `legacy_data` placements in `data/content/npcs/*.json` and eligible rows in `data/content/npc_spawns_extra.json`. `NpcVisibility::teleport_targets` exposes those same runtime rows, honoring its startup exclusions. The integration test confirms Samantha by name/GUID and Gus Galewind by GUID, then checks the immediate 35/2 visibility addition.
- The gateway already creates `ExpectedDiscontinuity::developer_teleport` for an `Outcome.relocation`; W63 preserves and exercises that path. The 2 m arrival's collision clearance and floor placement have not been checked in the native client.

## Verification

Rust commands ran from `project/prototype/osfr-tools` through `project/scripts/cargo.ps1`. Test artifacts used `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W63`; the release build used the default `prototype/osfr-tools/target` directory. Formatting used private `rustfmt.exe --edition 2024` on `src/dev_teleport.rs`, `src/gameplay.rs`, `src/world.rs`, `src/npc_interaction.rs`, `tests/dev_teleport.rs`, `tests/npc_interaction.rs`, `tests/world.rs`, and dependent `tests/piece_018.rs` call sites only.

| Command | Result |
| --- | --- |
| `test --test dev_teleport --test npc_interaction --test world --test gateway` | Passed: dev_teleport 1 passed / 7 ignored; npc_interaction 13 passed; world 8 passed; gateway 9 passed / 2 ignored. |
| `test --test gateway '--' '--ignored'` with `OSFR_GATEWAY_REFERENCE=lab/references/rust-gateway-reference` and `OSFR_GATEWAY_CAPTURE=lab/runs/client-baseline-20261008T020205Z` | Passed: 2 passed, 0 failed, 9 filtered out. PowerShell requires quoting the argument separator through the wrapper. |
| `test` | Passed with exit code 0 across the full suite. The movement baseline replay processed all recorded cases in 128.83 seconds and passed; the suite's declared private-data tests remained ignored. An initial compile found three stale `OpeningActor` callers in `tests/piece_018.rs`; those callers now convert to `dev_teleport::Target`, and the final run passed. |
| `build --release --bin osfr-gateway --bin osfr-login` | Passed. Default-target artifacts: `prototype/osfr-tools/target/release/osfr-gateway.exe` (12,729,618 bytes) and `osfr-login.exe` (12,036,613 bytes). |

Native client control was excluded by the worker contract. Remaining acceptance gaps are native confirmation of the Goodbye label/no-image rendering, a native 26/3 packet fixture, and collision/floor validation for content NPC teleport arrivals.

# W64 - regional quest offers, empty turn-ins and zone-less NPCs

Date: 2026-10-14. Status: implementation and requested automated gates pass; native UI acceptance remains open.

## Implemented

- **Eligibility:** regional quest eligibility now comes from unlocked profile IDs mapped through `jobs.json` to job IDs. XP is converted with the profile rank formula already used by `Profiles` (base rank 1, +1 per 100 XP, cap 20). Membership and configured quest limits come from `GameRuleSet`; unconfigured accounts use its active-member default and the existing local limit fallback. Eligibility refreshes when rules change, after opening/kart/quest XP and before NPC actions.
- **Samantha's offer:** the radial menu exposes her eligible quest; selecting the row routes through `QuestEngine::offer` and sends 49/1. The client 49/2 accept/decline decoder now accepts the native 9-byte request, and accepting creates the active zero-objective quest. Talking to the giver starts the turn-in. Empty-reward quests serialize a 99-byte 49/13 prompt, and 49/14 completes the quest through the existing idempotent reward transaction.
- **Zone-less regional NPCs:** all 50 positioned `legacy_data` NPC rows without a zone now spawn in `OPEN_WORLD_ZONE` (zone 0), use the same zone for interaction range checks, and appear in `realm goto` targets. The regression exercises Roosey through teleport and merchant interaction.

## Root cause and provenance

- `data/content/quests/adventurer.json` identifies local content key 3000021 with giver 9000000189 and empty prerequisite, objective and reward arrays. The matching row in `data/content/npcs/cobblestone_village.json` binds that key to the `quest_giver` role. The key is a reconstructed content ID, not a recovered retail server quest ID.
- Before eligibility refresh, `CharacterEligibility::default()` left both quest limits at zero. `QuestEngine::offer_denial` compares the occupied count against that limit, so an empty journal (`0 >= 0`) denied the quest and the NPC menu filtered it out through `can_offer`. `GameRuleSet` now supplies membership and optional limits; a missing limit retains the 100-slot local fallback.
- The regression reads only `gameplay.profiles` from `lab/reports/round5b-native-20261014T053834Z/character.json`, then loads that profile state with the current reference profile definitions and bundled regional content. It asserts the unlocked Adventurer profile maps to rank 1 before the offer flow.
- Native quest offer reader/handler provenance: `00c7bb60` and `00a94be0`; client accept/decline sender and constructor: `00a97d70`, `00a97e10`, and `00a86400`. These support the 49/1 -> 49/2 -> 49/3 offer path; the offer display's remaining body fields are still reconstructed by the existing 49/1 codec.
- Native 49/13 reader `00c7b990` uses common prefix reader `00c7acd0`, consumes five dwords, a reward bundle through `008e7930`, and a trailing float; `00c7bbd0` validates the bounded record. Constructor `00c7c030` stores the quest ID plus four scalar fields and the bundle/float; `00a95420` shows the end screen. Client 49/14 sender/constructor are `00a97ef0` and `00a864c0`.
- The 49/13 byte count follows the native reader shape: six-byte 49/subtype prefix + five dwords + 69-byte empty bundle body + trailing float = 99 bytes. The reader layout is recovered; this task's zero defaults for the four scalar fields, empty bundle values and float are reconstructed for the content's empty-reward quest.
- The zone-less placement rows and names come from the reviewed split NPC content. The reported baseline observation is in run `client-baseline-20261014T053834Z`; no native client was launched by this worker.

## Reconstructed policy and remaining limits

- The XP-to-rank curve and cap, avatar level fixed at 1 (the save has no separate avatar-level field), active-member default, unconfigured 100 quest limit, and zero defaults in the empty 49/13 are `(R)`, not recovered historical values. The local `GameRuleSet` default and profile curve are documented in `Research/Game Rules and Membership.md` and `src/profiles.rs`.
- Mapping zone-less `legacy_data` placements into shared `OPEN_WORLD_ZONE` is `(R)`. Native collision, floor placement and prompt radius were not evaluated here.
- `QuestMessage::encode_exact` continues to leave 49/13 contextual. Gameplay emits the reconstructed packet only when the quest has no coin, item or job-XP reward. Reward-bearing 49/13 serialization and its preview bundle remain unresolved.
- `GameRuleSet` changes refresh eligibility through `Gameplay::set_game_rules`, but gateway/account configuration was outside this task's file scope. Existing account membership is not yet wired into this setter, so current unconfigured gameplay uses the documented revival default.
- The 49/13 shape and generated bytes have unit coverage but no native client acceptance in this task. The worker did not launch or control the shared game window.

## Verification

Rust commands ran from `project/prototype/osfr-tools` through `project/scripts/cargo.ps1`. Test builds used `CARGO_TARGET_DIR=lab/build/W64`; release builds used the default target directory. Private `rustfmt.exe --edition 2024` formatted only the edited Rust source and test files; crate-wide formatting was not run.

Read-only native-map queries, also from `project/prototype/osfr-tools`, were `python -I ..\..\scripts\native_map\nmq.py dis 00a97d70 128`, `python -I ..\..\scripts\native_map\nmq.py dis 00a91630 128`, `python -I ..\..\scripts\native_map\nmq.py dis 00a86400 128`, `python -I ..\..\scripts\native_map\nmq.py dis 00c7bbd0 256`, `python -I ..\..\scripts\native_map\nmq.py dis 00c7c030 320`, and `python -I ..\..\scripts\native_map\nmq.py show 00c7b990 00c7c030 008e7930 0091c110`. The database is `lab/native-map/run/map.sqlite`; results corroborate the packet reader/constructor shapes summarized above. Raw decompilation was not written into the project repository.

| Command | Result |
| --- | --- |
| `test --lib native_save_samantha_menu_offers_empty_quest_and_talk_back_ends_it` | Passed: 1 passed, 0 failed. The test covers rank-1 native-save profile loading, Samantha's radial 49/1 offer, 49/2 acceptance, zero-objective talk-back 49/13, and 49/14 completion. |
| `test --test quest_engine --test npc_interaction --test world --test dev_teleport --test gateway` | Passed: quest_engine 10; npc_interaction 14; world 9; dev_teleport 1 passed / 7 ignored; gateway 9 passed / 2 ignored. |
| `test --test gateway '--' '--ignored'` with `OSFR_GATEWAY_REFERENCE=lab/references/rust-gateway-reference` and `OSFR_GATEWAY_CAPTURE=lab/runs/client-baseline-20261008T020205Z` | Passed: 2 passed, 0 failed, 9 filtered out. |
| `test --all-targets --quiet` | Exit 0; all runnable targets passed. Movement replay passed 10 tests in 130.49 seconds. Tests requiring other private content/captures remained ignored under their declared gates. |
| `build --release --bin osfr-gateway --bin osfr-login` | Passed on the default target directory. `target/release/osfr-gateway.exe` is 12,794,517 bytes; `target/release/osfr-login.exe` is 12,094,346 bytes. |

Native client acceptance for Samantha's radial offer, reconstructed 49/13 defaults, and zone-less NPC interaction remains open to the lead.
