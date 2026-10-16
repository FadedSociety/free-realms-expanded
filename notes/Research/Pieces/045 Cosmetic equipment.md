# 045 Cosmetic equipment

Work item 045: appearance/slot/ownership validation for cosmetic equipment, plus actual native
equip and restart persistence. This note records what piece 045 proves offline and what still
belongs to the lead's native gate.

"Cosmetic" is the local queue label. The recovered definitions are simply wearable rows with a
`Slot` and a `CategoryId`; the read-only wiki corpus
(`lab/references/freerealms-wiki/wiki.sqlite`) lists a `Cosmetics` title but holds no archived body
for it, so the term is not corroborated by that corpus. The piece therefore validates the recovered
appearance rows, not a reconstructed naming convention.

## ORIGINAL evidence

1. **The supplied client logs its own applied equip.** `lab/decompiled/client/FreeRealms/pseudocode.c`
   line 1099052 prints
   `"EquipItem: Slot=%d, ModelName=%s, TextureAlias=%s, TintAlias=%s, TintIdOverride=%d, Effect=%d."`
   That order is exactly what `src/inventory.rs::write_attachment` serializes, so the attachment
   bytes the piece parses are the client's own appearance fields.
2. **Recovered slot/category pairs.** `ClientItemDefinitions.json` (community starter import,
   AGPL-3.0, revision `9e082025391177d5a33ee18c714ddd06b457a695`) carries `Slot`, `CategoryId`,
   `ModelName`, `TextureAlias`, `TintAlias` and `Icon.TintId` on each wearable row. The pinned
   category name ids resolve in the supplied locale export to the labels below.
3. **UI slot labels.** `lab/decompiled/client-ui/ScriptsBase.lua` line 9005 `EquipmentSlotLabels`
   names the UI rows through locale ids `16359 Head`, `16368 Hands`, `16360 Chest`, `16361 Legs`,
   `16369 Feet`, `16370 Weapon`, `EquipScreen_Ring`, `EquipScreen_Necklace`, the five runes and
   `5381 Costume`. The exact index alignment between that UI table and the item `Slot` field is
   **not** proved here, so the piece does not assert it.
4. **Prepared content.** `lab/references/opening-anvil-placement-v1/content/inventory.json` is the
   running content: the captured `PlayerCode.SendPlayerUpdateItemDefinitions` payload with
   community starter rows merged in (its own provenance block records 298 creator-added ids).
5. **Saved native snapshots.** `lab/state/opening-*.json` version-two snapshots carry the equipped
   instance rows the native client accepted.
6. **Community equip handler.** `Sanctuary.Gateway/Handlers/BaseInventoryPacket/
   InventoryPacketEquipByGuidHandler.cs` resolves the wire tint as
   `clientItem.Tint == 0 ? clientItemDefinition.Icon.TintId : clientItem.Tint` and fills the
   attachment from the definition. Production's `write_attachment` implements the same rule.

## Recovered appearance table

Row counts re-derived from the pinned catalogue by `scripts/pieces/piece_045.py`; the same pairs and
counts are declared in `src/restoration/piece_045.rs` and cross-checked on every run.

| Slot | CategoryId | Category (locale) | NameId | Recovered rows |
|---|---|---|---|---|
| 1 | 101 | Hats | 23319 | 7522 |
| 2 | 102 | Gloves | 23320 | 3189 |
| 3 | 103 | Shirts & Dresses | 23321 | 8254 |
| 4 | 104 | Pants | 23322 | 5873 |
| 5 | 105 | Shoes | 2482 | 5867 |
| 6 | 106 | Miscellaneous | 23324 | 2301 |
| 17 | 201 | Costumes | 5273 | 93 |

Two facts stay visible instead of being smoothed over: slot 17 also holds 541 `Shirts & Dresses`
and 55 `Hats` costume overrides, and slots 1/3/5/6 each carry a large secondary `CategoryId 6`
(`Clothing`, name id 958) group. Weapons/tools (7), jewelry (8), shards (12/13) and runes are other
work items; the piece refuses them as cosmetic slots.

## RECONSTRUCTED policy

These rules are local validation policy over the recovered shapes, not recovered server code:

* an appearance must be for a recovered cosmetic slot;
* the model must be a non-empty `.adr` mesh whose only placeholders are `<race>`/`<gender>`
  (25,483 recovered rows carry both, 6,366 only `<race>`, 6 none);
* the texture alias must be a non-empty concrete name, the tint alias non-empty, the tint
  non-negative, and no field may exceed the content loader's 4,096-byte text bound;
* an equip request is only replayed when it is the recovered `42/3` (by instance) or `42/8`
  (by item record) shape; anything else reports `NotEquipRequest`;
* ownership, profile slot and class stay production's rules (`src/inventory.rs::equip`), which
  refuse an unowned instance, a definition slot that differs from the requested slot, and a class
  the profile does not allow. The community handler does not check the definition slot; production
  does, and the piece keeps that stricter behaviour;
* restart equality means the saved equipment bytes and the applied attachment bytes for every
  profile are identical after a save/load round trip.

## Implementation

* `prototype/osfr-tools/src/restoration/piece_045.rs` - `COSMETIC_SLOTS`, `Appearance`,
  `parse_profile_attachments`, `parse_equip_reply`, `parse_visible_reply`, `validate_appearance`,
  `inspect`, `replay_equip`, `capture_profiles`, `assert_profiles_identical`. The module includes
  `src/inventory.rs` by `#[path]` so the owned test target compiles before lead registration.
* `prototype/osfr-tools/tests/piece_045.rs` - 8 hermetic vectors plus one ignored private gate.
* `scripts/pieces/piece_045.py` - independent checker over the pinned catalogue, the prepared
  content and a saved snapshot.
* `tests/pieces/test_piece_045.py` - 10 Python tests.

## Evidence and results

Private evidence directory: `lab/reports/piece-045-20261009T041210Z/`.

RED (test written before the module existed):

```
scripts\cargo.ps1 test --test piece_045
error: couldn't find file `tests\..\src\restoration\piece_045.rs`
```

`red-rust-test.log`. GREEN:

```
scripts\cargo.ps1 test --test piece_045
test result: ok. 8 passed; 0 failed; 1 ignored
```

`green-rust-test.log`. Private gate over the preserved content and
`lab/state/opening-after-native-restart-203139.json`:

```
$env:OSFR_PIECE_045_CONTENT=...\opening-anvil-placement-v1\content
$env:OSFR_PIECE_045_SNAPSHOT=...\lab\state\opening-after-native-restart-203139.json
$env:OSFR_PIECE_045_EVIDENCE_DIR=...\lab\reports\piece-045-20261009T041210Z
cargo test --test piece_045 -- --ignored
test result: ok. 1 passed; 0 failed
```

`private-gate-rust.log` and `rust-replay.json`. That gate found 21 equipped cosmetic rows across the
saved snapshot (profiles 1, 14, 16, 43, 45), all with valid appearance; 21 accepted `42/3` replays
whose `38/5` and `35/6` attachments agree and whose equipment bytes are unchanged; 21 wrong-slot
replays refused byte-stably; and identical equipment/attachment bytes for all 16 profiles after a
save/load round trip.

Python checker (`python-tests.log`, `checker-run.log`, `findings.json`, `provenance.json`,
`run.log`):

```
10 tests OK
{"attachment_rows": 686, "checked_definitions": 851, "equipped_cosmetics": 21,
 "hard_findings": [], "missing_in_catalog": 32, "model_mismatch": [],
 "non_mesh_model_rows": 6, "slot_table_errors": 0, "texture_mismatch": [],
 "tint_alias_mismatch": [], "wire_tint_differs": 316}
```

Interpretation of those numbers, kept deliberately separate:

* **Verified match.** The module's seven slot/category pairs and their row counts equal the pinned
  catalogue. All 686 prepared definitions that carry a mesh match the catalogue's `ModelName`,
  `TextureAlias` and `TintAlias` exactly (zero model, texture or tint-alias mismatches).
* **Creator additions.** 32 ids (78037-78066, 78090-78091) have no catalogue row; the prepared
  content provenance records those as locally added definitions.
* **Non-mesh rows.** 77484-77489 carry `SkinTone 1..6` in the model column, so they are tokens, not
  attachments; their icon texture is reported instead of compared as an alias.
* **Informational deltas.** 8 slot remaps (ids in `findings.json`), 185 class remaps (102 to class
  0 from catalogue class 1, 64 to class 0 from no catalogue class, 19 to the shared costume class
  173), and 316 definitions whose wire `default_tint` differs from the community `Icon.TintId`.
  These are provenance differences between the captured wire records and the community starter
  rows, not appearance mismatches.

## Boundaries and remaining gates

* **Native equip and native restart persistence are not proved here.** No native client was driven,
  no UI was touched and no release was changed. The lead must equip a cosmetic through the real
  client, restart server and client, and confirm the item and its appearance are restored. The
  `rust-replay.json` samples give the exact expected `Slot/ModelName/TextureAlias/TintAlias/Tint`
  rows to compare against that run.
* Armor/weapon attributes, shard/weapon composite effects, jewelry and rune slots are other work
  items; this piece only refuses to call them cosmetic.
* The piece grants no item, coin or experience and changes no economy state.

## Lead integration

1. Register `pub mod piece_045;` and replace the `#[path]` include in the module with
   `use crate::inventory::{Inventory, InventoryState};` (drop the re-export shim and the
   `osfr_inventory` module in the test).
2. Optional follow-up: reuse `inspect` from the gateway before emitting an equip reply, or call
   `replay_equip` against a captured native `42/3`/`42/8` frame to bind the wire directly.

## Skills applied

RTK (normal commands first, condensed output), Ponytail (reuse `src/inventory.rs` and the existing
snapshot format; no new dependency, no parallel store), Caveman (concise updates), test-first RED
then GREEN, and the game-research/reverse-engineering workflow for reading the decompiled client
and the community catalogue as evidence rather than guessing. Native automation stayed with the
lead.
