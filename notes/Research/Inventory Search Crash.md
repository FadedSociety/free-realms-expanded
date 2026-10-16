# Inventory search crash and v2 spec reconciliation

Updated 2026-10-13. Static diagnosis and Rust-side encoders only; no native client was launched.

## Finding

The native search path contains an unguarded definition-cache dereference, and the exact inventory catalogue sent by this server omits definitions for three owned items. That is a direct, high-confidence cause for the observed access violation when the client builds the search index. The old crashed run has no dump or fault address, so the missing fault instruction cannot be claimed as runtime proof.

The prior Piece 082 preflight only projected rows that matched the entered text. It found all ten reconstructed “Wheeler” matches resolved, but did not model the native search engine's earlier pass over the owned-item list. The engine indexes candidates before filtering visible rows, so a missing definition outside the query matches still reaches the bad dereference.

## Native search path and data read

Evidence is from the accepted native map and byte checks under ../lab/reports/inventory-v2-20261013T192406Z/.

1. 00c09a80 is the InventoryScreen.SearchInventoryItems handler registered in the native Lua module. Its body reads a Lua string, converts it to wide text, and calls 0098faf0.
2. 0098faf0 tail-jumps to 00a6e7b0. The search refresh at 00a6e910 sends the stored text through this same row rebuild.
3. Before applying the row predicate, 00a6e7b0 calls 00990d80 to build search candidates. The accepted pseudocode and disassembly show 00990d80 walking the owned item records and calling 00b0d810 for each record.
4. The accepted label for 00b0d810 is marked unknown and its generic label text describes a string lookup; do not rely on that text. Its saved instruction bytes show it reading the input record's id at +0x04, following the manager pointer at client +0x20c, matching the id at the cached-node field +0xb4, and returning zero if no node matches (00b0d844). In this call context it is the definition-cache lookup.
5. At 00990e12, the caller invokes 00b0d810; at 00990e17, it immediately executes mov eax,[eax+4]. There is no null test between them. Any owned record with no cached definition can therefore raise an access violation while search builds its candidate list.
6. The row predicate at 00a6d9b0 is separate and runs after candidate creation. It resolves the definition, looks up the definition's category through the category manager at client +0x1bc, applies the category's +0x14 in-bag flag, and checks available count. Missing categories make rows invisible; this predicate has guards and is not the unguarded dereference above.

This pass also resolves the guide's earlier open question about whether the item search engine is on this path. 00990d80 calls the ItemSearchEngine constructor at 00c2cfa0, then 00c30450 to tokenize the query and rebuild its index, then 00c30a70 to score candidates; 00c30450 calls 00c2fb00/00c2f490 for query tokens and 00c30150 for indexed text, while 00c30a70 calls 00c2fc80 to compare fields. This is static call-chain evidence, not a runtime trace.

The indexer requests configured wide-string fields through a virtual getter and the scorer lowercases and compares those strings. Definitions carry name and description locale ids in their serialized record (raw offsets +0x04 and +0x08; runtime fields +0x08 and +0x0c); the client resolves those ids through its locale table. The exact configured search-field list and weights, including whether both name and description are indexed, are not fully recovered. Category display strings come from the 44/2 category NameId.

## Missing server data in the captured startup inputs

Input hashes and the reproducible counts are in ../lab/reports/inventory-v2-20261013T192406Z/input-audit.json.

| Owned instance | Definition id | Finding |
|---:|---:|---|
| 1145 | 7960 | No definition record in the 35/37 catalogue |
| 4594 | 78398 | No definition record in the 35/37 catalogue |
| 4595 | 78399 | No definition record in the 35/37 catalogue |

The input has 1,386 owned instances, 1,383 definition records and 1,329 unique definition ids. The 35/37 encoder in Inventory::definitions_packet (inventory.rs:294) correctly writes a byte-length prefix and the catalogue count, but simply writes the available map; it does not require every id in the self-body item list to be present. validate_content_state (inventory.rs:404-433) permits the three missing definitions when they match the initial default state. That exception is why the malformed dependency can reach the client.

The definition rows' positive NameIds all resolve in the pinned locale export. Category NameIds also resolve. One positive DescriptionId (6685) does not; that is a display-data gap, not the null lookup demonstrated above. No placeholder or deletion of the three owned items is justified by the available evidence. The pinned item-definition catalogs did not provide records for 78398 or 78399; the OpenSourceFreeRealms material inspected contains their self-body instance references but no definition record. The existing audit also found a possible community record for 7960, but not a byte-verified native definition suitable for insertion.

### Category observations

The current 44/2 packet has 71 category rows and 74 group associations. All 71 rows set the serialized bool at row offset +0x14 (the in-bag flag identified by the guide); the existing writer's ordering is byte-verified against its pinned serializer oracle. However, definitions reference categories absent from the table: category 0 (53 rows), 127 (6), 148 (1), and 149 (3). Group associations also reference absent category ids 58, 60, and 401-407. These gaps can hide rows. Category 0 may be an uncategorized sentinel; that interpretation is unverified, so its absence should not be treated as equivalent to the other missing categories. None of these gaps explains the unguarded definition-cache dereference, and no missing category rows should be fabricated.

## Rust server checklist comparison

The checklist is notes/Build Guides/Gameplay/Items and Inventory.md:236-258. “Recovered” below refers to client packet/layout facts; (R) is reconstructed server policy.

| # | Guide requirement | inventory.rs / item_categories.rs comparison |
|---:|---|---|
| 1 | Own a complete catalogue with definition fields and locale ids | Partial. Raw serialized rows preserve fields, but typed metadata covers only id, slot, class, tint, stack size, models, effect and wield type. No locale/reference validation; three owned definition records are absent. |
| 2 | Send 44/1 and 44/2 before complete 35/37, then self id 12 | Partial. 44/2 and length-prefixed 35/37 exist. No active 44/1 item-class packet was found in the Rust source. 35/37 omits the three owned definitions. Gateway wraps the 35/37 template at gateway.rs:817-820; categories are emitted through Gameplay initialization paths. |
| 3 | Every definition category exists in 44/2 and is in-bag | Mismatch. The category writer preserves the bool but does not reconcile it with definitions. The four missing category ids/counts are above; nine group category references also dangle. |
| 4 | Unique durable instance ids, merge by definition+tint, enforce stack | Implemented as reconstructed policy. InventoryState validates unique positive ids; instance_ids allocates from the persisted counter; grants merge matching non-rental definition/tint stacks and enforce maximum stack. |
| 5 | Persist count, consumed, stored, last cast, rental, equipment, item bar and storage | Partial. Count, consumed count, last_cast_time, rentals and equipment are stored. There is no stored-count field, item-bar assignment state, or storage usage/capacity. |
| 6 | Equip/unequip by supported requests; validate ownership, active profile, member, class, rank and gender; reply and persist | Partial. Handlers cover 42/2, 42/3 and 42/8, and validate ownership, requested slot, profile slot and allowed class. They do not implement 42/10-14, active-profile/member/rank/gender checks or required-slot policy. |
| 7 | Respect required slots and minimum ranks | Mismatch. Chest/Legs required status and Star/Moon Rune minimum ranks are not enforced. |
| 8 | Send 38/2 add, 38/3 update, 38/4 delete and 38/19 coin update | Partial. Add, delete, coin and update packets exist. The recovered 38/3 layout reads seven i32 values, but the guide marks their semantic names as reconstructed (R): under that mapping the current field five is ability_count instead of stored_count, and the final 64 bits are rental expiry instead of last-cast time. The assigned v2 module encodes that layout and mapping. |
| 9 | Answer 35/34 with one definition or include it in 35/37 | Packet shape is consistent with the native request class: the client writer sends the id plus an empty length-prefixed payload; the server replies with id plus length-prefixed definition. A missing id currently gets no reply, so the three gaps remain unresolved. |
| 10 | Persist item-bar assignments, send 38/25 and handle 36/10 activations | Not implemented. activate_enabled exists on an item but assignments and activations are not handled. |
| 11 | Handle immediate-use checks and atomic consumption/rewards | Not implemented in this inventory layer. |
| 12 | Support 42/18, 38/31/32, storage moves and stored count | Not implemented. |
| 13 | Persist and enforce rental expiration | Partial. Rental fields are serialized and validated; expiry enforcement is absent. Under the guide's reconstructed 38/3 field mapping (R), the current encoder also sends expiry in the last-cast field. |
| 14 | Own loot and prize-wheel results | Not implemented in these modules; separate reward/minigame work. |
| 15 | Send membership activation and VIP rank state | Not implemented in these modules. |
| 16 | Preserve item-derived rename and boost entitlements | Partial. Item definitions and grants exist, but derived entitlement logic is not in this module; missing definitions also prevent lookup for those ids. |
| 17 | Define style-card categories and support 42/10 as a set | Not implemented. |
| 18 | Route fixtures and pet gear through their dedicated systems | Outside these modules; no cross-system item route here. |
| 19 | Validate all name, description, requirement and category locale ids | Mismatch. No load-time locale check. Current captured data has no positive NameId or category-name gaps, but DescriptionId 6685 is absent. |

## v2 code and integration

New, currently unregistered helpers are in prototype/osfr-tools/src/inventory_v2_fixes.rs:

- encode_definition_batch_v2 writes the existing 35/37 framing and refuses to encode a catalogue if any currently owned definition id is absent. This guard is reconstructed policy (R); it prevents the unsafe packet but cannot supply the three unavailable definitions.
- encode_item_update_v2 writes the client-recovered 38/3 sequence: item id, count, item-bar slot, consumed count, stored count, and 64-bit last-cast time.

The assigned tests are prototype/osfr-tools/tests/inventory_v2_fixes.rs; they compare the current 35/37 behavior against fail-closed validation and print the old and corrected 38/3 bytes.

Lead integration steps:

1. Register pub mod inventory_v2_fixes; in prototype/osfr-tools/src/lib.rs.
2. Replace the body/signature of Inventory::definitions_packet (inventory.rs:294-306) with encode_definition_batch_v2, passing the definition byte records and every definition_id in the active InventoryState. Return an error when coverage is incomplete. First add byte-verified definition rows for 7960, 78398 and 78399; otherwise startup intentionally fails closed.
3. Propagate that error through Gameplay::definitions_packet (gameplay.rs:534-538) and the index-5 template replacement in Gateway::templates (gateway.rs:817-820). Preserve the 35/37 length-prefix bytes and send reference data before the client can use inventory.
4. Replace the free item_update encoder (inventory.rs:918-936) and its grant/consume call sites with encode_item_update_v2. The current InventoryItem has no stored_count, and its last-cast field is only i32; wire integration therefore needs a state/snapshot change to preserve the 38/3 layout using the guide's semantic field interpretation (R). Defaulting legacy snapshots to stored count zero is reconstructed policy (R).
5. Keep ItemCategories::initialization unchanged: its 44/2 field order and bytes already match the pinned serializer. Recover or correct category source rows before claiming all owned rows are visible. A native Wheeler rerun is still required after the missing definitions are resolved.

## Verification and remaining gate

Focused command:

    & scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','inventory_v2_fixes','--','--nocapture')

Result: 3 passed, 0 failed. The only compiler warning is the existing unused Lot::instance_index in housing.rs; the test target emitted no warning for the new module.

The former crash run has no dump, stack or fault address. The native code and exact missing-definition inputs establish an unguarded crash hazard and a strong static explanation, but only a controlled native search run after complete definitions are available can close the crash gate.

## Integration

Task W09, 2026-10-13. The inventory module now owns the startup guard and 38/3 encoding. This supersedes the earlier fail-whole-catalogue suggestion above: the three unavailable rows remain saved and are withheld individually; no definition records are fabricated and no gateway change is needed for this policy.

### Implemented

- `Inventory::definitions_packet` uses the v2 35/37 encoder and checks coverage for every owned item that the client-visible state will include.
- Reconstructed policy (R): `Inventory::load` logs unavailable owned definition ids. It retains those instances in `InventoryState` for saves, while `initialization_patch`, `profile_equipment`, and `profile_attachments` omit them and clear matching equipment links in the client view. The default owned rows 1145/7960, 4594/78398 and 4595/78399 are covered by a focused test. Saved equipment links to those default rows remain intact.
- 38/3 emits item id, count, bar slot, consumed count, stored count and an eight-byte last-cast value. `ability_count` and rental expiry stay in saved/self-body data and do not replace these update fields.
- `InventoryItem` now persists `stored_count` and an `i64` last-cast value. Reconstructed migration policy (R): absent legacy `stored_count` becomes zero. Game snapshot schema advances from 3 to 4; schema 3 snapshots remain readable and migrate on their next save. Character-only schema stays at 2.

### Provenance boundary

- Recovered 38/3 widths/order: byte-checked `008d8350` reads the 16-bit family/opcode, five 32-bit values, then eight bytes into the client object's `+0x20/+0x24` fields. `008b25e0` constructs opcode 3 and initializes those two dwords. Re-run commands: `python -I scripts/native_map/nmq.py dis 008d8350 320` and `python -I scripts/native_map/nmq.py dis 008b25e0 128`.
- Recovered initial self-body time input differs in width: `008fa4b0` calls `008cb5a0`; byte-checked `008cb5a0` reads one i32 and combines it with the client tick into a 64-bit runtime field. The existing 12 self-body writer therefore keeps its i32 field width and rejects out-of-range values rather than truncating them. Encoding a persisted 64-bit update value back into the self-body's time-delta form remains open.
- The semantic assignment of 38/3 fields five and six as stored count and last-cast time follows the Build Guide's reconstructed interpretation (R); only the field widths/order are recovered. The meanings of the last-cast time domain and bar-slot policy remain unverified.
- The missing-definition ids, instance ids and catalogue counts come from `../lab/reports/inventory-v2-20261013T192406Z/input-audit.json`, summarized above. The search hazard comes from the cited call/disassembly chain (`00990d80`, `00b0d810`, `00990e12` and unguarded dereference `00990e17`). It remains static evidence; no fault address or controlled native search rerun exists.

### Checks and remaining work

- Private-toolchain formatting succeeded with `rustup run stable rustfmt` on the six owned Rust files.
- Focused tests on `inventory`, `inventory_v2_fixes`, `save_schema` and `character_store`: 30 passed, 0 failed, 2 ignored. Existing warning: unused `Lot::instance_index` in `housing.rs`.
- Passing PowerShell command from `prototype/osfr-tools`: `$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W09'; & '..\..\scripts\cargo.ps1' test --test inventory --test inventory_v2_fixes --test save_schema --test character_store`. Cargo artifacts are under `../lab/build/W09`.
- The exact requested command including `--test game_store` exits 1 because unowned `tests/game_store.rs:38` still expects schema version 3; the save now correctly writes version 4. Character-store tests passed before Cargo stopped. Update that assertion within the lead's integration scope.
- Failing requested PowerShell command from `prototype/osfr-tools`: `$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W09'; & '..\..\scripts\cargo.ps1' test --test inventory --test inventory_v2_fixes --test save_schema --test game_store --test character_store`. Cargo stopped at `tests/game_store.rs:38` (`left: 4`, `right: 3`).
- Other test modules contain `InventoryItem` struct literals and need the new field/type updates if the complete integration-test suite is built (`economy_review`, `housing_service`, `piece_045`, `piece_081`, `startup_selfdata`, `type8_harvesting_gateway`). They are outside W09's owned paths and were not changed or run.
- Remaining guide mismatches: complete definition/locale metadata, 44/1 classes, missing category rows and group ids, broader equip/profile/member/rank/gender validation, unresolved 35/34 responses for missing ids, item bar and immediate use, storage, rental expiry, loot/rewards, member/VIP state, derived entitlements, style cards, and fixture/pet routing. The item-bar/storage pieces of checklist item 5 also remain absent. No category rows were fabricated. Native search rerun remains the gate for crash closure.
