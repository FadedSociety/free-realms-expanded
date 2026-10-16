# Original inventory food and potion use

Read-only recovery on2026-10-08. This prepares a separate native consumable milestone after the selected opening chain; it does not claim that current food or potion use works. No source, campaign, content, movie or capture analysis was changed, and no native input was sent.

Private proof: `../../../lab/reports/original-inventory-use-v1/proof.json`, SHA-256 `262d05bd51b55719ad93e27ddb988f7a22eebf9821ce8d1458ac1895cdacb9cf`. Reproducer `../../../lab/reports/trace-original-inventory-use.py` writes a new private report directory only. It binds27 original native function extracts,467 original machine instructions, six Lua ranges, four exact native method-table bindings, three cached wiki revisions, the immutable five-wolf checkpoint, current Rust handling and pinned community files. All read sources retained their hashes.

Independent review passed with no semantic finding: `../../../lab/reports/original-inventory-use-independent-review.json`, SHA-256 `9ba4e0ed8ef9bbe8549a10e01af647b52e44f1172a7069eca383d3c1261db551`. The reviewer independently verified all26 source hashes,27 native extracts,467 original PE instructions, six Lua extracts, four method tables, seven metadata rows, three pinned wiki revisions and exact earned Stew15/Corn11 ownership. Native consumption and numerical effects remain pending.

## Normal controls and requests

Original Lua `ScriptsBase.lua:8816-8831` distinguishes consumable equipment from clothing. The consumables equipment view has map18 and four slots; its Equip action calls `Ui.AssignToItemActionBar(slot-18, ownedItemID)`. The dock selector has its own `EquipItem` event and calls the same function with `button-5`. The safest initial native test uses the dock’s consumable selector rather than inventory text search, which has a separate known crash.

The native table binds `AssignToItemActionBar` to `00c23ed0`; it constructs `InventoryPacketItemActionBarAssign` via `00c14720` and sends through `00c22ad0` / writer`00c20a30`. Original base`00a10400` supplies main42, writer`00a1ada0` writes two16-bit headers. The request is exactly:

| Request | Little-endian inner fields | Length |
|---|---|---|
|42/6 assign consumable |u16 main42, u16 sub6, i32 zero-based slot, i32 owned item instanceID |12B |
|36/10 activate |u16 main36, u16 sub10, i32 barID2, i32 slot, i32 target mode; mode0 adds u64 targetGUID; mode1 adds Vector4 |24B for mode0;32B for mode1 |
|42/16 immediate-use check |u16 main42, u16 sub16, i32 owned item instanceID |8B |
|42/15 immediate-use confirmation |u16 main42, u16 sub15, i32 owned item instanceID |8B |

The last two are a distinct confirmation workflow, not a substitute for ordinary food activation. `InventoryHandler`’s `useItem` event calls `EquipScreen.CheckImmediateActivationItem`; its success callback opens an Okay/Cancel utility window, then calls `UseImmediateActivationItem` after Okay. Original constructors/writers and native table prove both8B forms. Do not send them speculatively for Stew.

Normal food/potion activation is dock `activateItem` → `Ui.ClickedAbility(button-5,2)` (`ScriptsBase.lua:30492-30494`); `QuickItem.ActivateItem` uses the same bar2 call. Native binding`00c173c0` forwards the optional bar argument through`008a7e40`, bar2 lookup`00a12840`, slot gating`00c44160`, cast selection`00a36a70`/`00a35600`, then common ability send`00a35150`/`00a35000`. Constructor`00a2e0e0` and writer`00a342e0` establish the schema above. Native target selection depends on server-supplied slot/ability targeting metadata. The example selfGUID1 vector in the proof is explicitly synthetic; a food-specific actual request has not yet been captured.

Original client slot gating can stop delivery for an empty slot, disabled status, recast timer, insufficient mana or no-cast player status (including knockout). Slot presentation comes from `BaseClient.ItemActionBar`; the dock reads icon, name, enabled state, quantity and cooldown. Pinned Sanctuary identifies the corresponding server update as38/25, `ActionBarData` plus `ActionBarSlot`. Some targeting/cooldown fields are still named Unknown in that community source. They should be recovered/verified before production support, rather than assuming any visible icon is usable.

## Effects and candidates

Community-first sources are Sanctuary revision`4f028dcc274f628b65ef665fd59b184f5f3d3e3e` and FreeRealms-Legacy revision`9e082025391177d5a33ee18c714ddd06b457a695`, bothAGPL-3.0. No upstream application was run or implementation copied. The existing unmodified FreeRealmsLocaleTools reader produced `locale-id-to-text.json`; no new format reader was written.

| Item | Catalog metadata | Verified original text / limitation |
|---|---|---|
|Strong Arm Stew35 |SingleUse, class9, category10, activatable ability3264 |Original locale3172 says eating increases health and player size. Amount, whether maximum/current health changes, duration, size multiplier, stacking and cooldown are unrecovered. |
|Wheeler on the Cob1195 |SingleUse, activatable ability232 |Original description is flavor text about real corn; it does not establish healing. |
|Small Health Potion11002 |SingleUse, class9, category9, ability4696 |Original locale422916 says health restoration over5seconds. A separate2173 definition has the same displayed name but ability267 and is absent from the current prepared catalog; names alone cannot select the ability. |
|Medium Health Potion3021 |SingleUse, class9, category9, ability4697 |Same generic5second description. |
|Wheeler Health Potion65820 |SingleUse, class9, category9, ability4696 |Original locale4532 says healing on consumption; no amount. |

The immutable `opening-after-native-five-wolves-183551.json` owns one Corn1195 instance11 and one Stew35 instance15. Neither that checkpoint nor the fresh seven-item initial inventory owns a health potion. Catalogued potions are not automatically awarded or accessible shop stock. Stew is the strongest existing earned candidate for native assignment/activation; a potion test needs a deliberate separate fixture/grant with its provenance recorded.

Cached wiki evidence is secondary, and its revision era is explicit:

- Medium Health Potion revision10651,2013-02-15:16%health/second for3seconds plus2%instant.
- Wheeler Health Potion revision10857,2013-02-15:11%/second for3seconds plus2%instant.
- Small Health Potion revision44062,2021-01-20:11%/second for3seconds plus2%instant.

Their raw wikitext and clean-text hashes are bound in the private proof. These three-second claims differ from the original generic five-second tooltip; no local original ability effect database currently resolves that discrepancy. Do not silently convert either source into production numerical rules. The cached pages do not provide Strong Arm Stew’s missing values.

Pinned Sanctuary demonstrates a useful owned-slot → owned-item → ability lookup and consumption protocol, but its default single-use fallback consumes an item without restoring original Stew3264 or health4696/4697 numerical behavior. Its food visual abilities use modern fixed durations/cooldowns and do not prove historical rules. Consuming the item alone would be an incomplete restoration.

## Current gap and smallest acceptance gate

Current Rust intentionally no-ops42/6 and ignores bar2 in36/10; `Inventory::consume` is an internal atomic recipe/economy primitive. There is no normal client consumable workflow, saved actionbar binding, native slot refresh or health/size/potion effect implementation. Current saved candidate item records have `activate_enabled=false`. Follow-up original38/25 recovery now proves that the item field is independent of the native slot enable/cast pathway; no item-field mutation is needed for a request probe.

Proposed next evidence milestone, before any effect numbers are chosen:

1. Use a new branch of an immutable checkpoint with earned Stew; retain the original reward checkpoint unchanged. Recover the original38/25 slot targeting/cooldown field semantics and native parser from the pinned serializer plus native reader.
2. Implement only the proven owned-item assignment/slot presentation with exact packet framing and bounded slot IDs; no consumption yet. Root uses the ordinary dock selector, captures actual42/6 and the enabled rendered slot, then clicks the slot to capture the actual bar2/target36/10 form.
3. Keep any effect implementation blocked on an explicit sourced rule table or a clearly labelled reconstruction decision. A successful consume-only click is not faithful health/size/potion acceptance.
4. Once rules are established, test normal use, one-item decrement, health/size/buff display and expiry, cooldown, full-health policy, invalid target/ownership, retransmission/replay, knockout, bundle rollback and restart. Verify exactly one effect/consumption and no changes to quest rewards or other inventory.

This is an implementation candidate and acceptance plan, not permission to mutate the active run, inject a packet or award a potion.

## Follow-up: original toolbar fields and private probe

Read-only proof `../../../lab/reports/original-item-toolbar-v1/proof.json`, SHA-256 `eb1fe14305fb2e769beda8486bf8cf5414c5c24c9f617e68f64f5361fd47c24e`, binds22 original functions,358 PE instructions and four newly compiled unchanged community vectors. The original native reader consumes63B38/25:4header+8bar/slot+51slot. Native `008dae30` treats the community's `Unknown7` bits as a float, not an integer distance. Slot serverEnabled is at absolute offset41 and is independently gated by empty state/status/mana/recast; it does not read `InventoryItem.activate_enabled`.

The actual supplied reference self has a disabled Medium Health Potion in bar2/slot0, with selectors1/2/zero numeric parameter. Native `00a35600` explicitly forces self for selector1+target2 (and1+4). This resolves a safe client-control template while leaving original Stew-specific tuning unknown. The reference's59-potion toolbar quantity is inherited presentation, not ownership in the new campaign.

Unintegrated private source and detailed integration/native acceptance plan: `../../../lab/reports/item-toolbar-candidate-v1/session_toolbar.rs` and `README.md`. Five offline tests pass, including exact full community packet oracles, rejection/ownership/target bounds, no consumption, one-click capture, unassign/reconnect and clone-discard rollback. It borrows that original potion control template with verified Stew name/icon/owned quantity, captures one actual native click per assigned slot if later integrated, then returns the authentic “can't use that ability right now” failure and clears the probe slot. It selects no health/size/duration/buff rule and changes no persistent state. This is an honest request-inspection milestone, not restored consumable effects.

Independent review **PASS**: [receipt](../../../lab/reports/original-item-toolbar-independent-review.json), SHA-256 `68c5dc6c9e30c91316eeea44a2e4fc25329f567196df9be08f2c0828501e613d`. The reviewer rehashed source/extracts, checked original PE/vtable, independently recompiled all four community vectors and reran all five tests. [Immutable offline candidate receipt](../../../lab/reports/item-toolbar-candidate-v1/offline-candidate-receipt.json), SHA-256 `4d993dc1d198b83e065daf7871f982801b4af71e12639975c8110017b85bd4c9`, binds the unmodified candidate. [Integration conditions](../../../lab/reports/item-toolbar-candidate-v1/integration-conditions.md) require a fresh default session and an **unmounted** disposable fixture because the borrowed `ForceDismount=true` may have a client control side effect. Real gateway context/bundle/manifest integration and actual native assignment/click/failure/restart acceptance remain pending. Reassignment can permit another inspection request; there is still no consumption or effect.
