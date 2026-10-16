# Startup Catalogues

Research snapshot: 2026-10-13. Scope: W06 pure packet builders for startup catalogue and status indices 00, 04-06, 35-37, 40, 42, and 44. This is capture compatibility work, not recovered production content or a complete startup implementation.

## Artifacts and result

- Builder: `prototype/osfr-tools/src/startup_catalogues.rs`.
- Isolated test target: `prototype/osfr-tools/tests/startup_catalogues.rs` (`#[path]` includes the private builder module; no shared module registration).
- Private oracle and synthetic checks: 7 tests total, 6 passed and the capture oracle is ignored by default. With `OSFR_GATEWAY_REFERENCE` set to the private reference directory, the oracle passes and byte-compares all ten requested packets.
- Command and output record: [`catalogues-20261013T204520Z/command-log.md`](../../../lab/reports/catalogues-20261013T204520Z/command-log.md).
- Captures remain at `../../../lab/references/rust-gateway-reference`; the inventory hashes and packet size metadata are in [`capture-inventory.json`](../../../lab/reports/startup-builders-20261013T191103Z/capture-inventory.json). No packet captures were copied into this note or the repository.

The captured inner message lengths are: index 00 13 bytes; 04 19; 05 263,901; 06 64; 35 44; 36 and 37 each 97; 40 39,247; 42 2; and 44 52. Including each seven-byte tunnel header, those match the startup inventory totals.

## Recovered layouts and provenance

| Index | Builder and input | Client facts recovered from native map | Capture-specific or open fields |
|---|---|---|---|
| 05, 35/37 | `build_item_definitions`; ordered serialized definition records and IDs | Dispatcher `0092f460`; bounded length/count batch reader `00a3db70`; fixed fields `008e3760`; extension reader `00919e10` calls the fixed reader and consumes four further dwords; cache `00a3c290` looks up by ID and replaces an existing entry. | Keep source order and duplicate records for wire parity. The private catalogue has 1,383 records. The existing `Inventory::definitions_packet()` calls `inventory_v2_fixes::encode_definition_batch_v2` (`inventory.rs:319-336`, `inventory_v2_fixes.rs:34-90`), which sorts and deduplicates by ID; its result has 1,329 unique IDs and cannot reproduce this captured record stream. The standalone builder therefore reuses the same framing rules but accepts the captured ordered serialized records. Definition semantics remain governed by the Items and Inventory guide. |
| 40, 165/1 | `build_coin_store_item_lists`; separate static and dynamic rows | Handler `00c75fc0`; record reader `00c71510` (label was speculative, then checked with `nmq dis`) reads three dwords and three bytes after each four-byte key. Two counted lists; 19 bytes per entry. | Names/meaning of all three dwords and three flag bytes remain `(R)`; preserve the bytes. Capture lists have 1,834 and 231 rows. |
| 35, 36/5 | `build_ability_set_definition`; set ID and tagged entries | Dispatcher `00a35cc0` case 5; packet constructor `00a2ccf0`; parser `00a32fe0`; serializer `008fa5c0`; slot reader `008e6760`. The captured set contains eight kind-0 empty slots. | Other tagged records are represented as opaque serialized tails; their field meanings and value policy remain `(R)`. The ordinary test varies set ID and entry count. |
| 36-37, 38/16 | `build_effect_tag`; ID and length-delimited blob | Client update dispatcher `009392c0` case 16; bounded blob reader `008d8870`; application path `00a19f00`/`00a19e70` keeps records in an ID-keyed map. | Blob internals remain opaque content bytes. Captures contain 85-byte blobs with IDs 2 and 4. |
| 44, 38/7 | `build_stat_update`; target GUID and typed stat rows | Handler `0091e1a0`; row parser `0090a540`; counted-list reader `008fc4c0`; row reader `008e7640` (speculative label checked with `nmq dis`). Rows are stat ID, value kind, value; kind 0 is `i32`, kind 1 is `f32`; other kinds and NaN floats are rejected by the client reader. | Meanings of stat IDs and captured values are not established as server policy. Synthetic test covers both value kinds and changes GUID and rows. |
| 00, 41/132 | `build_overworld_combat_flag`; typed combat bool | Encounter dispatcher `00aa36c0`; parser path `00a9fb50`/`00a9ed60` and bounded reader `008d6690`; setter `009058d0` receives the bool. The body has two preceding dwords. | The two dwords remain named capture-valued `(R)` constants (both zero in the capture). Only the final bool is exposed as state. |
| 04, opcode 144 | `build_client_game_settings`; no input | Constructor `0089bc20` establishes packet identity. No client body reader was mapped. | All five body values are fixed named capture-valued `(R)` constants: three dwords, one byte, and one float bit pattern. |
| 06, opcode 193, observed 193/2 | `build_unidentified_193_2`; no input | Client dispatcher `009471a0` case `0xc1` routes to confirmed `AnnouncementsSubsystem_HandlePacket` at `00a40aa0`. `nmq command 193` returns `CommandObject_DrawCard`, a separate serialized-command ID, not this packet. | The observed capture body has not been correlated to the announcement sub-reader; retain its exact 62-byte post-opcode tail as named `(R)` bytes. The native route alone does not identify this capture's meaning. |
| 42, opcode 14 | `build_opcode_14_marker`; no input | Client dispatcher `009471a0` has case `0xe`; disassembly shows it writes `NetInfo.log` and sets a flag at `+0x24`. | Purpose/startup semantics remain unresolved. `nmq packet '^14$'` returns community rows for multiple packet families, including `PacketZoneDoneSendingInitialData`; that catalogue is not a native mapping for this captured empty marker. Preserve only captured opcode 14. |

`(R)` means reconstructed or capture-retained, not recovered server policy. Packet outer framing is tunnel ID 5, flag byte 1, little-endian inner length. Build Guides consulted: Gameplay/Items and Inventory, Gameplay/Combat and Abilities, Gameplay/Coin Store and Purchases, and Network/Gateway and Transport.

## Verification

Ordinary synthetic tests vary item records/count, coin-list entries, ability ID/count, effect ID/blob, stat GUID/rows/types, and the combat bool. The ignored byte oracle reads the private capture and `rust-gameplay/inventory.json`, constructs the typed inputs, and compares complete tunnel packets byte-for-byte for all ten indices. Oracle success establishes parity with these captures only; it does not establish original production catalogue values, rules, or live-client acceptance.

Commands, native-map queries, exact outcomes, the test-first red baseline, and the final test commands are recorded in the linked report. The private Rust toolchain was selected by `scripts/cargo.ps1` with a lab-only target directory. No native game client was launched and no shared gateway or game module was changed.

## Open gaps

- Coin-store field meanings and ability/effect payload semantics remain unresolved.
- Item records arrive as already serialized bytes; this task did not implement a content source that generates the 1,383-record capture catalogue from recovered server rules.
- Stat names/IDs, original values, the two combat dwords, settings values, and the meaning of indices 06 and 42 are not established.
- Index 06's `0xc1` announcement-family route needs exact subtype/reader correlation before it can be treated as a semantic builder.
- These builders are not registered in the shared startup gateway. The capture oracle is not a substitute for native runtime acceptance or the remaining startup-message builders.
