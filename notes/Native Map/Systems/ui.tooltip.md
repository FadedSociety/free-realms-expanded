# System: ui.tooltip

48 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a73e00` | `AbilityEffectsTooltipDataSource::ColumnTitle` | AbilityEffectsTooltipDataSource | Column header text for the ability-effects tooltip data source. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a74f60` | `AbilityEffectsTooltipDataSource::DtorBody` | AbilityEffectsTooltipDataSource | Destructor body of AbilityEffectsTooltipDataSource: releases the 10-slot entry array member and the holder. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75000` | `AbilityEffectsTooltipDataSource::AdjustorToScalarDtor` | AbilityEffectsTooltipDataSource | Adjustor thunk: forwards the secondary-base this pointer (minus 0x254) to the scalar deleting destructor. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75230` | `AbilityEffectsTooltipDataSource::Ctor` | AbilityEffectsTooltipDataSource | Constructor of AbilityEffectsTooltipDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a752e0` | `AbilityEffectsTooltipDataSource::ScalarDeletingDtor` | AbilityEffectsTooltipDataSource | Scalar deleting destructor of AbilityEffectsTooltipDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a756b0` | `TooltipDataSource::RowFieldOrError` | TooltipDataSource | Returns a row's text field, or Error! for an index past the row count. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a76360` | `TooltipDataSourceNodeArray_ScalarDeletingDtor` | SoeUtil::Array<TooltipDataSource::Node,… | Deleting destructor of the TooltipDataSource::Node array type, reached through virtual slot 0. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a76700` | `Tooltip_FireOnTooltipRequest` |  | Raises the Lua event TooltipMethods:OnTooltipRequest with two parameters for a tooltip request. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a76930` | `TooltipDataSource::DtorBody` | TooltipDataSource | Tears down a TooltipDataSource object: destroys its node array and then its base-class parts. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a771b0` | `StoreItemTooltip_BuildRows` |  | Fills the StoreItemTooltip row table for a store item (image set, category and tint, price, entry count, first item and item list). | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a77b40` | `ItemGroupTooltip_BuildRowsById` |  | Starts an item-group tooltip for a group id: resets the tooltip state, sets its kind to 0xf, writes the group rows, and optionally fires the OnTooltipRequest event. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a77d40` | `AbilityItemTooltip_BuildRows` |  | Fills the AbilityItemTooltip row table for an ability item (profile, experience, rank experience and maximum, activatable flag and text). | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a786d0` | `AbilityTooltip_BuildRows` |  | Fills the AbilityTooltip rows for an ability record and registers it with the active-ability tooltip source when that source exists. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a78ca0` | `ProxiedCharacter_BuildRows` |  | Fills a ProxiedCharacter tooltip with the proxied character's name, description and icon. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a79880` | `TooltipDataSource::Ctor` | TooltipDataSource | Constructs a TooltipDataSource with an empty node array and three default Column/Value rows. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a799b0` | `TooltipDataSource::ScalarDeletingDtor` | TooltipDataSource | Deleting destructor of TooltipDataSource objects, reached through the primary vtable slot 0. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |

## 🟢 likely (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00961730` | `ClearTooltipStateAndHide` |  | Selects the gated or ungated tooltip-hide helper from the remaining flag bit. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963dd0` | `UpdateTooltipVisibility` |  | Coordinates tooltip show/hide operations from the stored state and row counts. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a73ee0` | `AbilityEffectsTooltipDataSource::CastCheckSetFlagNotify` | AbilityEffectsTooltipDataSource | Refresh hook for the ability-effects tooltip: when the bound GUI source is an AbilitiesDataSource with the matching field, it marks the source changed and notifies the owner. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75010` | `AbilityEffectsTooltipDataSource::GetCellEntry` | AbilityEffectsTooltipDataSource | Cell accessor for the ability-effects tooltip entry list (row, column, out). | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a756e0` | `ClearFields280To294` |  | Resets the list count (+0x280) and related state of the tooltip data source; called from the hide-tooltip paths 00904bf0 and 00904e70. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75da0` | `CastToAbilityEffectsTooltipDataSource` |  | Checked cast of a GUI data source to the ability-effects tooltip data source, logging a type-mismatch diagnostic when a name was supplied. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a761e0` | `TooltipDataSourceNodeArray_DtorBody ⚠downgrade` | SoeUtil::Array<TooltipDataSource::Node,… | Releases the node elements and buffer of a TooltipDataSource::Node array owned by a TooltipDataSource. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a76210` | `LookupAbilityEffectsTooltipDataSource` |  | Named lookup of the ability-effects tooltip data source in a GUI data-source table; returns 0 when the name is absent or the object is of another type. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a76a00` | `TooltipDataSource::Thunk_Slot1_Adjust254` | TooltipDataSource | Makes slot 1 of the secondary TooltipDataSource vtable (installed at +0x254) reach 00a799b0, which is the scalar deleting destructor of TooltipDataSource. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a76a10` | `TooltipDataSource::Slot0_GetField248` | TooltipDataSource | Reports the element count field of the TooltipDataSource node array (the same field 00a76160 uses as its bound), as seen through the secondary base. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a77a80` | `Tooltip_BuildItemRowsById` |  | Starts an inventory item tooltip for an item id: resets the tooltip state, rebuilds its rows through 00a76a60 and notifies the owner. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a78a40` | `RecipeTooltip_BuildRows ⚠downgrade` |  | Fills the RecipeTooltip row table for a recipe: icon, name, description, membership flag, count and quantity, and lists components when the recipe component source is registered. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a78e80` | `TrophyTooltip_BuildRows ⚠downgrade` |  | Fills the TrophyTooltip row table for a trophy: name, description, icon, tint, membership flag and progress. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a79620` | `RecipeTooltip_BuildRowsKind13 ⚠downgrade` |  | Second recipe tooltip variant, used by a different caller (00c23fc0) that presents the same recipe rows under kind 13. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a799d0` | `Tooltip_ItemByIdOrRequest` |  | Shows an inventory item tooltip from the item cache when the definition is known; otherwise, when asked, requests the item definition from the server and waits for it. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00c1fdc0` | `TooltipData_SetRowCount` |  | Changes the number of TooltipData rows. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c1fe10` | `TooltipData_SetText` |  | Updates a TooltipData entry with a wide-string value. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20040` | `TooltipData_PopulateFactoryNpcTooltip` |  | Populates the FactoryNpcTooltip title, values, and effect rows. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20220` | `TooltipData_PopulatePetTooltip` |  | Builds pet tooltip rows from the matching pet record; the identifier's source is unresolved. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c202b0` | `TooltipData_PopulatePetTrickTooltip` |  | Uses the supplied trick identifier to populate PetTrickTooltip fields. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20420` | `TooltipData_PopulateAbilityTooltip` |  | Populates AbilityTooltip fields and any active ability-effect rows for the supplied ability value. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c20480` | `TooltipData_PopulateAbilityItemTooltip` |  | Populates the AbilityItemTooltip fields from the resolved ability-item record. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `01251db0` | `ActionCodeToTooltip` |  | Maps a numeric game-action code to the tooltip label shown for that action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a750e0` | `AbilityEffectsTooltipDataSource::GetAfterOptionalRefresh ? → AbilityEffectsTool…` | AbilityEffectsTooltipDataSource | Virtual slot [2]. When the byte at this+0x240 is nonzero it calls 00a74af0 with ECX = this-0x30. Then returns the dword at this+0x250. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75100` | `Setter274RefreshNotify ?` |  | Takes one stack argument, stores it at this+0x274, sets the byte at this+0x270 to 1, calls 00a74af0 with ECX = this, then calls vtable slot +0x10 of this (ECX = this); returns with ret 4. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75720` | `ClearFields280ThenSetMode10 ?` |  | Takes two stack arguments. When the global object's +4 sub-object is nonzero: zeroes the dwords at this+0x280, +0x290, +0x294 and the byte at +0x28c, calls 008356b0 with 0, increments the dword at +0x288, sets +0x280 to… | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a74af0` | `—` |  | Clears the list at this+0x280 through 00a740e0 with the count. Reads the global object through 00769710 and returns when its +4 sub-object's +0x174 field is zero. Otherwise calls 00a34e30 with this+0x274 and this+0x278,… | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a75790` | `— ⚠reject` |  | When the global object's +4 sub-object is nonzero: zeroes the same fields as 00a75720 (+0x280, +0x28c, +0x290, +0x294), calls 008356b0 with 0, increments the dword at +0x288, sets +0x280 to 11, and when the dword at sub… | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a76160` | `— ⚠reject` | TooltipDataSource | Hands one TooltipDataSource node's wide member (or a static default for an out-of-range index) to a consumer; the consumer is not identified. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a76520` | `— ⚠reject` |  | Pushes ability values held by this object into the active, passive and item-ability tooltip data sources when those sources are registered, and registers the BaseClient.Abilities source. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a790f0` | `— ⚠reject` | TooltipDataSource | Re-fills the row table of the current tooltip kind after its underlying data changes, then notifies the owner; the item and ability branches are the two cases visible here. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00c1ffe0` | `— ⚠reject` |  | Populates recipe name, icon, description, membership, quantity, and component fields in the recipe tooltip. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a799d0` | `Tooltip_ItemByIdOrRequest` | send | PlayerUpdatePacketItemDefinitionRequest |  | The server is expected to answer the item-definition request with the PlayerUpdatePacketItemDefinitions reply for the same id (hypothesis; the reply is handled elsewhere, not in this function), so the cache fill depends on that reply. |
