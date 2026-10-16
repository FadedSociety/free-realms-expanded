# Integration Round 4

Date: 2026-10-13

## Status

The W59 modules are registered and the selected integration targets pass. The private startup oracle and release gateway/login builds pass. The crate-wide test command remains blocked by `tests/piece_012.rs`; an individually run target sweep found one stale assertion in `tests/piece_050.rs`. Merchant 66/43 and typed NPC quest delivery remain evidence/content gaps described below.

## Integration changes

- Registered `game_rules`, `npc_interaction`, `progression_extras`, `recipes`, and `harvest_nodes` in `src/lib.rs`. The test-only `#[path]` copies were removed; tests import the library modules. The W56 source-tag adapter was removed.
- Added `SourceTag::LegacyData` with the `legacy-data` serde spelling and evidence handling. Legacy provenance remains distinct from reconstructed policy.
- `Gameplay` loads the bundled NPC placement/region data when the selected gameplay content directory does not contain those files. `NpcVisibility` emits `35/2` records through `startup_world::build_add_npc`; `Gateway` updates the view after Loaded and movement events.
- Regional `npc_id` values are stable runtime GUIDs and content keys. Extra placement rows use their stable `spawn_guid`; only regional actors have a proven current interaction-content key.
- The 23 startup actors are excluded from dynamic additions using typed snapshot actor IDs. In template fallback mode the corresponding captured ready templates (indices 10 through 32) are parsed for those IDs. The 23 startup packets themselves remain on their existing byte-preserving startup path.
- Mapped regional NPC requests route 26/8 to 26/9 and 26/10 into dialogue, quest-engine offers when a typed quest catalogue is available, or the vendor list. Active Coin Store 165/4 and 165/5 packets go through `NpcInteraction` stock/range checks and `CoinShop` validation.
- The character-list row now places Base64 UTF-8 of the character display name in `Payload`. A login test compares the complete list bytes using the identity already held by the account store.
- Login account-info values are obtained through `GameRuleSet` from account membership and slot policy. The local account schema has no admin flag, so admin is false (R).

## Recovered client facts

- `PlayerUpdatePacketAddNpc_ReadBody` at `00921a10` establishes the AddNpc field sequence and types. The typed startup snapshot parses the original 23 actors from ready messages 10–32. This establishes packet shape and captured values, not the historical correctness of new regional positions.
- NPC request 26/8 is built at `008b6bc0` and sent by `0092efe0`; its inner shape is a 4-byte family/subtype, 8-byte target, and one bool. The 26/9 menu readers include `00aa2360`, `00aa2220`, `00aa1600`, and `00aa04d0`. The 26/10 constructor/sender are `00a36f80` and `00a39090`.
- ShowDialog 26/3 reader `00a9ef10` consumes a 93-byte base plus 20 bytes per choice. The 49/1 reader `00c7b7a0` consumes six dwords, a bool, a reward bundle, and a fixed 14-byte tail; the empty reward bundle base is 69 bytes. These readers establish field order, not the semantic values used by the new local builders.
- Merchant 66/42 reader `00b29cd0` and nested record reader `00b33020` establish row layout. The 66/43 serializer at `00b3ab20` writes two dwords, three dwords, a length-prefixed byte string, and three trailing dwords. Native field meanings and transaction/result behavior remain unresolved in the reviewed NPC interaction note.
- Coin Store buy/sell are separate family 165 packets: subtype 4 is 28 bytes and subtype 5 is 20 bytes. The existing `CoinShop` validators process those transactions; vendor authorization and numeric result choices are reconstructed policy.
- `ClientAccountManager::OnAccountInfo` at `00719330` consumes membership, maximum characters, and admin values. `GameRuleSet::account_info_callback_bytes` supplies those callback values; the six-byte login payload retains the existing wire layout.
- The character-list readers inspected for this task are `00fa7730`, `00fa6e30`, `00fa61e0`, and `00fa4370`. `ReadEntityRecord` at `00fa4370` reads the row's `Payload` string. The local fix places the encoded display name there; native GUI acceptance remains pending.

## Reconstructed policy (R) and limits

- Visibility uses the existing local player-interest policy: 100 m 3D radius and the nearest 64 actors. The source pattern is in `src/bin/osfr-gateway.rs`; the radius and cap are local choices, not recovered retail values.
- `npc_spawns_extra.json` has no zone field. Each row is assigned to the zone of its nearest positioned regional NPC. In the inspected dataset, 3,265 extra rows map against 1,095 regional anchors; nearest-anchor distance is 0.33 m minimum, 48.85 m median, 435.75 m at p95, and 926.8 m maximum. Long-distance assignments may be wrong.
- Capture-matched extra rows are skipped. Captured startup actors are excluded by GUID so a nearby region row cannot duplicate those actors.
- Missing regional model/name IDs use zero-valued AddNpc fields (R). The selected 12 m interaction radius is the existing server-side interaction policy; it is not the client's prompt-acquisition rule.
- `NpcInteraction` uses regional content keys for menus and vendor authorization. Extra `spawn_guid` rows lack a validated mapping to regional interaction records and are not interactive yet.
- Vendor stock comes from `data/content/vendors.json`; its item mappings include partial wiki evidence joined to client item definitions and `stores.json` reference prices. They are not native captured NPC price quotes. The 66/42 semantic field values are local opaque-field policy (R).
- When a typed root `ContentDb` with all required files is present, the gateway constructs the `quest_engine::QuestEngine` for NPC offers. The checked-in regional content is split into directories and does not provide that aggregate `npcs.json`/`quests.json` set, so current data cannot initialize this engine. The 49/1 builder uses the recovered empty 69-byte bundle shell with zero-filled unresolved fields (R); this is not native-tested quest presentation.
- The new NPC quest engine is session-only: its state is not part of `GameplayState`, and the gateway does not yet route the client quest-accept 49/2 request into it. Offers are therefore not a persistent end-to-end quest flow.
- Talk 26/3 uses a reader-shaped, zero-filled local shell with available content string IDs. Its dialogue presentation is speculative. 66/43 is parsed only behind an open vendor session; it is not translated to CoinShop because item, quantity, and merchant fields have not been identified in client evidence.
- `CoinShop` currently owns an inventory snapshot. Gameplay copies shop inventory back after a shop transaction, but does not refresh the shop's private copy after unrelated inventory/reward changes. The shop/gameplay inventory can diverge within a session; a narrow CoinShop inventory-sync API is needed for safe mixed gameplay.

## Commands and results

All Rust commands used `project/scripts/cargo.ps1` from `project/prototype/osfr-tools`. Formatting used the private toolchain `rustfmt.exe` with `--edition 2024 --config skip_children=true` on only the edited Rust files; no crate-wide formatting ran.

| Test target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| `content` | 8 | 0 | 0 |
| `content_npc_placement` | 1 | 0 | 0 |
| `content_npc_spawns` | 1 | 0 | 0 |
| `content_npcs` | 1 | 0 | 0 |
| `game_rules` | 7 | 0 | 1 |
| `gateway` (normal) | 8 | 0 | 2 |
| `harvest_nodes` | 1 | 0 | 0 |
| `login` | 11 | 0 | 2 |
| `npc_interaction` | 9 | 0 | 0 |
| `npc_interaction_vendors` | 1 | 0 | 0 |
| `progression_extras` | 6 | 0 | 0 |
| `progression_extras_content` | 1 | 0 | 0 |
| `recipes` | 5 | 0 | 0 |
| `recipes_wire` | 4 | 0 | 0 |
| `world` | 8 | 0 | 0 |

Additional gates:

- `cargo.ps1 check --lib`: passed; existing dead-code warnings only.
- `cargo.ps1 test --lib`: 27 passed, 0 failed, 0 ignored.
- Private startup oracle, with `OSFR_GATEWAY_REFERENCE=lab/references/rust-gateway-reference` and `OSFR_GATEWAY_CAPTURE=lab/runs/client-baseline-20261008T020205Z`, run as `cargo.ps1 test --test gateway -- --ignored`: 2 passed. The startup test compared the 46 original replies byte-for-byte; the captured-client oracle returned the expected 53 total replies.
- `cargo.ps1 build --release --bin osfr-gateway --bin osfr-login`: passed in the default target directory.
- `cargo.ps1 test --no-fail-fast`: blocked at compile time by `tests/piece_012.rs:82` and `:127`, where two `CookingTransaction` literals omit the newly required `experience_points` field. That file was outside W59 ownership and was left unchanged.
- To exercise the rest of the integration suite despite that compile blocker, 212 other integration targets were run individually. 211 targets exited successfully. `piece_050` had 9 passed and 1 failed: `production_check_name_preserves_the_saved_native_appearance` still expects the old `selection_reply` bytes at its list step. The new character-list bytes are asserted in `tests/login.rs`; `tests/piece_050.rs` was outside W59's assigned test-import edits and was left unchanged.
- `piece_012` was the one integration target not individually runnable because its test source does not compile. The full-suite gate is therefore not green.

The visibility test in `tests/world.rs` loads the checked-in NPC data, confirms regional and extra AddNpc packets, enforces the 64-addition bound, checks unique GUIDs, and verifies that excluded startup GUIDs do not reappear.

## Native test plan

1. Select a character and inspect the character-list row's `Payload`; confirm the UI shows the character name, not a list status value. Compare the client request/reply capture with the unit-tested bytes.
2. Enter the opening area and compare its 23 startup NPC packets with the private startup capture. Confirm those actors do not receive a second dynamic AddNpc. Walk across the 100 m interest boundary and confirm adds/removes track the visible set without exceeding 64 NPCs.
3. Walk within interaction range of a mapped regional NPC, request its menu, select Talk, and inspect the visible dialogue. The 26/3 semantic values require native review.
4. Talk to a quest giver with the typed quest catalogue enabled. Capture 26/8, 26/9, 26/10, and 49/1; verify the offer and then acceptance/reward behavior. Current split regional data does not enable this catalogue by itself.
5. Open a mapped vendor, inspect 66/42 stock, then buy and sell through the Coin Store. Capture 165/4 and 165/5 and verify balance/inventory changes. Capture 66/43 with all fields before any transaction translation is added.

## Open gaps

- The crate-wide suite needs its external `piece_012` test literals updated for the concurrent `experience_points` field. The stale `piece_050` list expectation also needs review against the intended `Payload` fix.
- The active checked-in NPC dataset cannot initialize a typed NPC quest engine; 49/1 and 26/3 semantic values have not passed native acceptance.
- NPC quest acceptance/state persistence and CoinShop inventory synchronization are not wired through the whole session lifecycle.
- Merchant 66/43 cannot safely map to CoinShop transactions until the native fields are identified. Extra placement actors do not yet have validated content-key mappings.
- The private startup gate proves the untouched startup sequence, not new NPC visibility, vendor behavior, quest presentation, or the character-select GUI.
