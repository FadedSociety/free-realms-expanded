# System: ui.equip_screen

34 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a737b0` | `EquipmentSlotsDataSource::ColumnName` | EquipmentSlotsDataSource | Column header names for the equipment slot data source. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a739a0` | `EquipmentSlotsDataSource::DtorBody` | EquipmentSlotsDataSource | Destructor body of EquipmentSlotsDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a73a10` | `EquipmentSlotsDataSource::Ctor` | EquipmentSlotsDataSource | Constructor of EquipmentSlotsDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a73a80` | `EquipmentSlotsDataSource::ScalarDeletingDtor` | EquipmentSlotsDataSource | Scalar deleting destructor of EquipmentSlotsDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00c10aa0` | `RegisterEquipScreenLuaModule` |  | Publishes the equipment-screen command table to the client's Lua UI; with this table every ui.equip_screen function in this shard gets its script name. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00990600` | `SetVehicleEquipScreenState` |  | Updates vehicle equip-screen state across the data-source list. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00a10750` | `Slots_CommitChangedApplyThunk` |  | Thunk to Slots_CommitChangedApply; the EquipScreen preview-card caller supplies the UI context. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00c10bb0` | `EquipScreenEquipItemGroup` |  | Equips a whole item group (set) in the preview. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10bf0` | `EquipScreenEquipItem` |  | Equips one item given by its string/record reference plus an integer (slot or count). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10c50` | `EquipScreenGetItemPreviewType` |  | Returns how (if at all) the given item can be previewed on the character, for the equipment UI. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10f10` | `EquipScreenGetMaxItemsPurchasable` |  | Returns how many units of the item the player may still buy, for the store/equipment UI. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11020` | `EquipScreenPreviewStyleCard` |  | Previews a style card on the character. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11110` | `EquipScreenSelectVehicle` |  | Selects a vehicle in the equipment/vehicle UI with two numeric identifiers. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c111f0` | `EquipScreenSetVehicleTypeFilter` |  | Sets which vehicle types the equipment/vehicle list shows. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11390` | `EquipScreenGetMaxBundlesPurchasable` |  | Returns how many units of the given bundle the player may still buy. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11410` | `EquipScreenGetBundlePreviewType` |  | Returns how the given bundle can be previewed on the character. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11500` | `EquipScreenGetBundleEquippableState` |  | Tells the equipment UI whether the given bundle's items can currently be equipped. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11580` | `EquipScreenCanPreviewBundle` |  | Answers whether the given bundle may be previewed. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11b10` | `EquipScreenUseStyleCard` |  | Lua command that applies a style card to the character (inventory channel 42 sub-opcode 10). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11cb0` | `EquipScreenCheckImmediateActivationItem` |  | Lua command that asks the server whether an immediate-activation item may be used right now. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c10b20` | `EquipScreenSelectJob ? ⚠rename` |  | Selects the job whose equipment set the screen shows. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10b60` | `EquipScreenEquipLightsaber ? ⚠downgrade` |  | Equips a two-handed/lightsaber-class item in the equipment preview using the two numeric arguments (item id and colour/variant). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10fd0` | `EquipScreenUseStyleCardByItemId ? ⚠rename` |  | Applies a style card identified by item id (the send path for the style-card packet lives in the sibling command 00c11b10). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11060` | `EquipScreenClearPreviewStyleCard ? ⚠downgrade` |  | Clears the style-card preview (the -1 argument means 'all'). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11160` | `EquipScreenEquipVehicleItem ? ⚠downgrade` |  | Equips a vehicle-related item referenced by name/guid string. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c111b0` | `EquipScreenRemoveVehicleItem ? ⚠rename` |  | Removes a vehicle item from the character. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c11bc0` | `EquipScreenUseImmediateActivationItem ? ⚠downgrade` |  | Lua command that consumes an immediate-activation item (food/potion style) and returns true to the script when it queued the request. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0098fa40` | `— ⚠reject` |  | Applies the supplied value to two nested objects and calls VehicleListDataSource::SetActiveVehicleId; the caller path runs through inventory-category selection. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009905c0` | `— ⚠reject` |  | Refreshes row identifiers for equip-screen data sources. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00a73820` | `— ⚠reject` | EquipmentSlotsDataSource | Cell accessor for the equipment slot data source (key, column, out). | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00c10ac0` | `— ⚠reject` |  | Refreshes the equipment screen's item list for a job/category id taken from the script. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10d20` | `— ⚠reject` |  | Previews an item on the character by its numeric item id, using the client's ItemRecord helper object. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10e30` | `— ⚠reject` |  | Tells the equipment UI whether the player can currently equip the given item (default '4' when the argument is missing/invalid). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10ea0` | `— ⚠reject` |  | Answers whether the given item may be previewed, for the equipment UI. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c11b10` | `EquipScreenUseStyleCard` | send | InventoryPacketUseStyleCard |  | Hypothesis: the server consumed the style card and pushed the resulting cosmetic update; the client request shape is proven, the reply is not. |
