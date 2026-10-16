# Self Data Layout

Research snapshot: 2026-10-13. Scope: login message index 07, inner opcode 12 (`SendSelfToClient`), from the private gateway reference. Offsets below are relative to the opcode body unless marked packet-absolute. Native client reads establish wire shape; they do not establish server policy.

## Builder result

`prototype/osfr-tools/src/startup_selfdata.rs` parses and rebuilds identity, appearance, transform, name, account fields, profiles, equipment, profile abilities, collections and rewards, inventory items, gender, both quest lists and nested objectives, the post-quest record families, action bars and slots, action/tutorial keys, and effect records. Inventory uses `inventory::InventoryState`, `inventory::InventoryItem`, and `inventory::EquippedItem`; quest progress overlays the shared `quests::QuestState` through the local binding already used by the gateway.

Profile map keys come from the first wire dword: `00a19da0` calls `00a18990`, whose instructions load the key from `*param_2`. `00911d60` reads the profile record, including both keyed pair-lists; `008fa5c0` reads a count-prefixed ability vector; and `008ca050` reads its following zero-key-terminated rows. The eight-element allocation at `00ae1f70` is capacity, not the serialized count. The 12-byte second pair-list matches the equipment spans in `profiles.json` and `inventory.json` and uses `inventory::EquippedItem`.

Unknown fixed fields remain explicit named byte arrays: reward-bundle fields, quest row prefixes/tails, recipe/variable/pre-action record payloads, profile ability dwords with unresolved meanings, 51-byte action-slot payloads, and 81-byte effect payloads. The final 1,962-byte tail is still exact opaque data because its first count is 268 while `00913850`'s element reader `008e9c10` requires at least 24 bytes per row. No server policy is inferred from these preserved fields.

The required Cargo target passed nine synthetic tests, with the private oracle ignored by default. Running that oracle separately with `OSFR_GATEWAY_REFERENCE` passed and rebuilt the 203,850-byte capture exactly. The same results were confirmed by the isolated rustc harness; command details are in `lab/reports/selfdata2-20261013T215535Z/command-log.md`.

## Envelope and reader order

The capture is 203,850 bytes including the seven-byte tunnel header. Its inner opcode-12 body is 203,837 bytes. Packet offsets are body offsets plus 13. Dispatch reads the opcode at `009471a0`; `PacketSendSelfToClientCtor` at `0089b240` stores opcode 12 and the inner length.

| Order | Body offset | Decoded wire section | Native evidence |
|---:|---:|---|---|
| 1-5 | 0 onward | Unknown identity `u64` (R), GUID `u64`, model `u32`; appearance strings/colors/IDs; position and rotation `float4`; name record; account fields including coins. | Top-level reader `00944fe0`; transform helper `008e2410`; name helper `008e7290`; gateway prefix layout. Unknown identity is object `+0x3c0`, GUID `+0x3c8`, model `+0x35520`. |
| 6 | 281 | Profile count 16; rows begin at 285. The first wire dword is the keyed-map key. The second keyed pair-list in each profile is decoded as equipment `(slot,item_id,category)` triples. | `00921460`, profile reader `00911d60`, map insertion `00a19da0` -> `00a18990` (disassembly confirms the key is `*param_2`); list readers `008fc8c0`/`008fc960`. The equipment identification is independently corroborated by the opening `profiles.json` and `inventory.json` spans: the equipment offset lands exactly on the second 12-byte pair-list. Example packet-absolute span: profile 1 begins at 298 and its equipment list begins at 697. |
| 7 | 8697 | Active profile `u32`. | `00944fe0`, lookup `008bf5f0`; lookup consumes no additional bytes. |
| 8 | 8701 | Profile-type count; ten two-dword rows end at 8785. | `00944fe0`, insertion `00909f00`. Pair meanings remain unresolved. |
| 9 | 8785 | 253 collection records end at 114935. Each row is seven dwords, reward bundle `008e7930`, then keyed 33-byte rows via `008fb360`/`008dacc0`. Capture totals: 511 reward entries across types 1/3/6/10/12 and 1,815 keyed rows. | `009099d0`, `008e7930`, `008fb360`, `008dacc0`; speculative/unknown labels were checked with `nmq dis`. The builder preserves each full bounded collection row. |
| 10 | 114935 | Inventory count 1,386; item rows end at 157815. Native item writer-compatible rows are 30 bytes without rental data and 46 with it. | `0090a630` calls `008fa4b0`, then reads the trailing activation byte at `ClientItem +0x31`. Rust writer shape is cross-checked against `inventory.rs::write_item`; `inventory.json` proves the full count-plus-items span is 42,880 bytes. `stored_count` is not in this self-data wire shape; the parsed server field defaults to zero (R). |
| 11 | 157815 | Gender `u32`, value 1 or 2 in this capture. | Direct read in `00944fe0`; gateway gender patch validates the same boundary. |
| 12 | 157819 | First quest list: 25 records plus a 17-byte tail, ending 165789. Second list: 101 records plus a four-byte tail, ending 189311. Nested objective lists use `0093c840`/`0093ca50` and reward bundles. | Top-level `00940c60`/`00940e60`; row readers `0093c840`, `0093ca50`, `008fd770`, objective helper `0092b050`, reward helper `008e7930`. `008fd770` bytes were checked with `nmq dis`. Quest/objective keys, rewards and row boundaries are typed; unresolved fixed prefixes/tails remain bytes. |
| 13 | 189311 | Counted keyed/string records end at 189344. | `008e73b0`. |
| 14 | 189344 | 17 recipe records, 55 fixed 32-byte component rows, end at 191822. | `0090b930`, `0090a720`. |
| 15 | 191822 | 28 variable records with nested arrays, end at 200051. Then signed pet-source marker `-1` and `u64`, ending 200063. | `00912cf0`, then `00b5b470`; nested sizes were checked against bytes. |
| 16 | 200063 | 34 pre-action records end at 201317. | `008e74c0`; each fixed prefix, counted byte string and two-byte tail is a separate typed record. |
| 17 | 201317 | One action bar: key, scalar, nested count, four keyed 51-byte action-slot payloads. Bar ends at 201553. | `00a16d50`, `0091a9b0`, `008fce50`, `008dae30`. The `0091a9b0` scalar precedes the counted slot list; each slot row is key `u32` plus 51 bytes. |
| 18 | 201553 | One unnamed byte; then 55 dword action keys ending at 201778; one tutorial key ending at 201786. | `00909fd0` and `008e8230`; tutorial insertion thunk `00a12820` to `00c8df70`. The flag meaning is not recovered. |
| 19 | 201786 | One effect record: key `u32` plus 81-byte payload, ending at 201875. | `00944fe0` -> `00a19e70` -> `00a18cc0` -> `008e93e0`; `00a18cc0` adds one dword, one bool, and two dwords after the 68-byte complex record. Payload field meanings remain mostly unnamed. |
| 20 | 201875 | Remaining 1,962 bytes, preserved as `post_effect_tail`. The first dword reads as 268, but `00913850`'s element reader `008e9c10` requires at least 24 bytes per row and fewer than 1,962 bytes remain. The nested boundary therefore cannot be valid as currently decoded. | `00913850`, `008e9c10`, `008e7640`, `0093f950`, `0090a890`, `0091b530`; disassembly is saved under the selfdata2 report. This is an unresolved capture/native-boundary mismatch, not a guessed section parse. |

All offsets and counts are from `0007-login.bin` and the bounded-reader walk in `lab/reports/selfdata2-20261013T215535Z/probe_sections_corrected.py`. No player name or chat value is included in this note.

## Inventory, equipment, and quest state

`InventoryState.coins` serializes at the existing account-prefix coin field. `InventoryState.items` serializes through the same field order as `inventory.rs::write_item`: three signed IDs/attributes, stack and consumed counts, signed last-cast time, optional rental fields, ability count, and activation byte. `stored_count` is not present in this message. Equipment is stored in profile rows, not after gender: profile item records are the second keyed pair-list within each `ClientPcProfile`, with each row matching the server's 12-byte equipment serializer. The builder splits that list from the profile's other opaque bytes and re-serializes it from `InventoryState.equipment`.

The server's local `quests.json` journal proof starts at packet offset 157832 (body offset 157819), length 7,970. Its one binding points to quest 1617, objective 0, at journal-relative offset 4,112 (packet offset 161944/body offset 161931). The builder exposes that count through `quests::QuestState` and patches only the bound signed dword in the serialized typed records. The quest ID/objective binding is local reconstructed content (R), not a client-named identity. The wire does not establish `accepted`, `completed`, or `rewarded`; those flags are preserved by the caller's state but are not serialized by this binding.

## Opaque data and limits

- Collection rows, reward bundles/entries, and keyed 33-byte rows are parsed and rebuilt with exact count and conditional widths; their unnamed fixed payloads remain bytes.
- Both quest lists and nested objective/reward rows are parsed and rebuilt with exact boundaries; fixed row prefixes/tails remain bytes except the reviewed local objective counter binding.
- Keyed strings, recipe rows and components, variable rows and their three nested arrays, pet marker, and pre-action rows are parsed as separate typed records; unresolved fixed fields remain bytes.
- Action slots are 51-byte records. Their keys are typed; field payloads remain exact bytes.
- Effect records are bounded to the 81 bytes consumed by the native reader. The later list at body offset 201875 remains opaque because its declared count conflicts with the minimum native element size.
- The private byte oracle and nine synthetic checks pass in the isolated W18 harness. The oracle comparison is exact. No native game client was launched.

## Provenance

- Original client facts: address-level readers listed above, inspected through `python -I scripts/native_map/nmq.py dis <address> <size>` and saved under `lab/reports/selfdata2-20261013T215535Z/`.
- Capture boundaries: `lab/references/rust-gateway-reference/0007-login.bin`, outer/inner lengths, and `probe_sections_corrected.py` structural walk.
- Server state reuse: `prototype/osfr-tools/src/inventory.rs`, `prototype/osfr-tools/src/quests.rs`, and read-only gateway patch paths.
- Local quest binding and profile equipment span cross-checks: `lab/references/rust-opening/quests.json`, `profiles.json`, and `inventory.json`. Those files support this restoration's local server state; they are not claims about retail server policy.
- No AGPL source was copied. No raw game assets or decompiled source were written into the project repository.
