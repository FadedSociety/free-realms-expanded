# System: ui.housing

84 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (57)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0b410` | `RegisterHouseModule` |  | Registers the large client 'House' Lua module (instance/entry, fixtures place/move/rotate/pickup, grid spacing, names, ratings, neighbour visits, toggles) that ScriptsBase.lua's Housing object calls. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b490` | `Housing_SetUseFreeFlyCamera` |  | Handler registered as 'SetUseFreeFlyCamera' in the House module (0x0182add0): the housing free-fly camera toggle persisted through the user options. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b510` | `Housing_SetUseAutoOrientation` |  | Handler registered as 'SetUseAutoOrientation' in the House module (0x0182add0): the auto-orientation toggle used when placing fixtures. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c0a0` | `House_RotateFixtureCounterClockwise` |  | Handler registered as 'RotateFixtureCounterClockwise' in the House module (0x0182add0); note 027 recovers it as the radial-menu action that rotates the pending fixture counter-clockwise (action id 6). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c0f0` | `House_RotateFixtureClockwise` |  | Handler registered as 'RotateFixtureClockwise' in the House module (0x0182add0); note 027 recovers it as the clockwise radial action (action id 7, which the shared transform negates to -15 degrees). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c210` | `Housing_GetUseFreeFlyCamera` |  | Handler registered as 'GetUseFreeFlyCamera' in the House module (0x0182add0): the getter half of the housing free-fly camera toggle. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c310` | `Housing_GetUseAutoOrientation` |  | Handler registered as 'GetUseAutoOrientation' in the House module (0x0182add0): the getter half of the auto-orientation toggle (ScriptsBase.lua House.GetUseAutoOrientation). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cbb160` | `FixtureItemDataSource::ColumnHeaderText` | FixtureItemDataSource | Column header text for the fixture item table: FixtureGuid and Location are the two fixed headers, lower indices come from 00c92c10. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb1b0` | `HouseNamesDataSource::ColumnHeaderText` | HouseNamesDataSource | Column header text for the house names table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb1e0` | `SelectedFixtureDataSource::ColumnHeaderText` | SelectedFixtureDataSource | Column header text for the selected fixture table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb210` | `HousingInstanceDataSource::ColumnHeaderText` | HousingInstanceDataSource | Column header text for the housing instance table, shared by the Housing, OwnerHousing, MyHousing and PlayerHousing instance data sources. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb310` | `HousingDataSource::ColumnHeaderText` | HousingDataSource | Column header text for the housing data source table (the owner/house record view). | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb420` | `FixtureDataSource::ColumnHeaderText` | FixtureDataSource | Column header text for the fixture data source table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb480` | `HouseCustomNameDataSource::ColumnHeaderText` | HouseCustomNameDataSource | Column header text for the house custom name table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb4e0` | `HouseRatingKeywordsDataSource::ColumnHeaderText` | HouseRatingKeywordsDataSource | Column header text for the house rating keywords table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb520` | `HouseSearchDataSource::ColumnHeaderText` | HouseSearchDataSource | Column header text for the house search result table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbbac0` | `FixtureItemDataSource::Ctor` | FixtureItemDataSource | Constructor of FixtureItemDataSource, the fixture item table data source. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbc260` | `FixtureItemDataSource::DeletingDtor` | FixtureItemDataSource | Scalar deleting destructor of FixtureItemDataSource. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbd390` | `FixtureItemDataSource::GetValue` | FixtureItemDataSource | Provides fixture-item row values to the housing data source UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbd530` | `HouseNamesDataSource::GetValue` | HouseNamesDataSource | Provides available house-name row values to the housing UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbd700` | `SelectedFixtureDataSource::GetValue` | SelectedFixtureDataSource | Exposes the selected fixture's identifier as a data-source value. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbd760` | `HousingInstanceDataSource::GetValue` | HousingInstanceDataSource | Supplies housing-instance fields to the owner, own-house, and player-house data-source variants sharing this implementation. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbdc90` | `HousingDataSource::GetValue` | HousingDataSource | Provides the loaded house's properties to the housing UI data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbe150` | `FixtureDataSource::GetValue` | FixtureDataSource | Provides fixture-row identifier values to the housing UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbe200` | `HouseCustomNameDataSource::GetValue` | HouseCustomNameDataSource | Provides custom house-name fields and the resolved name to the housing UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbe430` | `HouseRatingKeywordsDataSource::GetValue` | HouseRatingKeywordsDataSource | Supplies house-rating keyword rows to the housing UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbe570` | `HouseSearchDataSource::GetValue` | HouseSearchDataSource | Provides house-search result values to the housing UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbedd0` | `HouseCustomNameDataSource::Destroy` | HouseCustomNameDataSource | Destroys the custom house-name data source and its base subobjects. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbee50` | `HouseRatingKeywordsDataSource::Destroy` | HouseRatingKeywordsDataSource | Destroys the rating-keyword data source and its base subobjects. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbeed0` | `HouseSearchDataSource::Destroy` | HouseSearchDataSource | Destroys the house-search data source and its base subobjects. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbef50` | `HouseNamesDataSource::HouseNamesDataSource` | HouseNamesDataSource | Constructs the data source that exposes available house names. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf030` | `HouseNamesDataSource::ScalarDeletingDtor` | HouseNamesDataSource | Implements scalar deleting destruction for HouseNamesDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf090` | `SelectedFixtureDataSource::SelectedFixtureDataSource` | SelectedFixtureDataSource | Constructs the housing view of the currently selected fixture. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf170` | `SelectedFixtureDataSource::ScalarDeletingDtor` | SelectedFixtureDataSource | Implements scalar deleting destruction for SelectedFixtureDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf1d0` | `HousingInstanceDataSource::HousingInstanceDataSource` | HousingInstanceDataSource | Constructs the common base for the owner, own-house and player-house instance views. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf280` | `HousingInstanceDataSource::ScalarDeletingDtor` | HousingInstanceDataSource | Implements scalar deleting destruction for HousingInstanceDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf3f0` | `FixtureDataSource::ScalarDeletingDtor` | FixtureDataSource | Implements scalar deleting destruction for FixtureDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf410` | `HouseCustomNameDataSource::HouseCustomNameDataSource` | HouseCustomNameDataSource | Constructs the custom house-name list data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf4a0` | `HouseCustomNameDataSource::ScalarDeletingDtor` | HouseCustomNameDataSource | Implements scalar deleting destruction for HouseCustomNameDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf4c0` | `HouseRatingKeywordsDataSource::HouseRatingKeywordsDataSource` | HouseRatingKeywordsDataSource | Constructs the house-rating-keyword list data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf550` | `HouseRatingKeywordsDataSource::ScalarDeletingDtor` | HouseRatingKeywordsDataSource | Implements scalar deleting destruction for HouseRatingKeywordsDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf570` | `HouseSearchDataSource::HouseSearchDataSource` | HouseSearchDataSource | Constructs the house-search results data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf600` | `HouseSearchDataSource::ScalarDeletingDtor` | HouseSearchDataSource | Implements scalar deleting destruction for HouseSearchDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf620` | `OwnerHousingInstanceDataSource::OwnerHousingInstanceDataSource` | OwnerHousingInstanceDataSource | Creates the owner-instance-list view over the common housing-instance data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf6f0` | `OwnerHousingInstanceDataSource::ScalarDeletingDtor` | OwnerHousingInstanceDataSource | Implements scalar deleting destruction for the owner-instance data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf750` | `MyHousingInstanceDataSource::MyHousingInstanceDataSource` | MyHousingInstanceDataSource | Creates the player's own-house list view. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf820` | `MyHousingInstanceDataSource::ScalarDeletingDtor` | MyHousingInstanceDataSource | Implements scalar deleting destruction for the own-house list data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf880` | `PlayerHousingInstanceDataSource::PlayerHousingInstanceDataSource` | PlayerHousingInstanceDataSource | Creates the player-house list view. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf950` | `PlayerHousingInstanceDataSource::ScalarDeletingDtor` | PlayerHousingInstanceDataSource | Implements scalar deleting destruction for the player-house list data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbf9b0` | `HousingDataSource::HousingDataSource` | HousingDataSource | Constructs the main housing-instance data source with its fixture subviews. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbfbf0` | `HousingDataSource::ScalarDeletingDtor` | HousingDataSource | Implements scalar deleting destruction for HousingDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cf9d40` | `HouseRatingDataSource::GetColumnName` | HouseRatingDataSource | Provides column labels for the house-rating UI datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf9dc0` | `FeaturedDataSource::GetColumnName` | FeaturedDataSource | Provides column labels for the featured-house datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf9f70` | `HouseRatingDataSource::GetCell` | HouseRatingDataSource | Supplies house-rating cell values to the housing UI. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa100` | `FeaturedDataSource::GetCell` | FeaturedDataSource | Supplies featured listing cell values to the housing UI. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa3b0` | `HouseRatingDataSource::HouseRatingDataSource` | HouseRatingDataSource | Constructs the house-rating datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa460` | `FeaturedDataSource::FeaturedDataSource` | FeaturedDataSource | Constructs the featured-rating datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0b5b0` | `Housing_SetHouseName` |  | Handler registered as 'SetHouseName' in the House module (0x0182add0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b600` | `Housing_EnterPreview` |  | Handler registered as 'EnterPreview' in the House module (0x0182add0): enters a house preview from the UI. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b6d0` | `Housing_PreviewTintAlias` |  | Handler registered as 'PreviewTintAlias' in the House module (0x0182add0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b860` | `Housing_GetFixtureGroupByType` |  | Handler registered as 'GetFixtureGroupByType' in the House module (0x0182add0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b990` | `Housing_RemoveFixtureCustomizationFromGroup` |  | Handler registered as 'RemoveFixtureCustomizationFromGroup' in the House module (0x0182add0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c530` | `Housing_IsActionPermitted` |  | Handler registered as 'IsActionPermitted' in the House module (0x0182add0): the permission query the housing UI runs before an action. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cbf2a0` | `FixtureDataSource::FixtureDataSource ⚠downgrade` | FixtureDataSource | Constructs the fixture-list data source for a housing instance. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

## 🟡 speculative (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0b3c0` | `Housing_ClearPlacementFields ?` |  | Housing manager helper: the registered House.PlaceFixture handler 00c0b710 clears the same four fields inline before placing a fixture, so this is the standalone clear used by the housing placement flow. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cbb460` | `FixtureDataSource::GetSub238Field7c ?` | FixtureDataSource | Virtual slot 2 of FixtureDataSource (thiscall, no arguments): when the pointer at receiver+0x238 is nonzero returns the dword at that pointer plus 0x7c, otherwise returns 0. Bytes 8b 81 38 02 00 00 85 c0 74 04 8b 40 7c … | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbbba0` | `FixtureItemDataSource::GetField52d0 ?` | FixtureItemDataSource | Virtual slot 2 of FixtureItemDataSource: bytes 8b 81 d0 52 00 00 c3 decode to mov eax,[ecx+0x52d0]; ret, returning the dword at receiver+0x52d0. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbbbe0` | `HouseNamesDataSource::GetViaSub228Dec0 ?` | HouseNamesDataSource | Virtual slot 2 of HouseNamesDataSource: bytes 8b 81 28 02 00 00 8b 80 c0 de 00 00 c3 decode to load the pointer at receiver+0x228, load the dword at that pointer plus 0xdec0 and return it. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbbbf0` | `HousingInstanceDataSource::GetField234 ?` | HousingInstanceDataSource | Virtual slot 2 of the housing instance data sources (shared implementation): bytes 8b 81 34 02 00 00 c3 decode to mov eax,[ecx+0x234]; ret, returning the dword at receiver+0x234. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbc480` | `HouseRatingKeywordsDataSource::GetValueAfterRebuild ?` | HouseRatingKeywordsDataSource | Virtual slot 2 of HouseRatingKeywordsDataSource (vtable 01837938; thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbbc50 with ECX = receiver-0x30 (the base sub-object); then returns the dword at… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbc4a0` | `HouseSearchDataSource::GetValueAfterRebuildSlot4 ?` | HouseSearchDataSource | Virtual slot 2 of HouseSearchDataSource (thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbbcb0 with ECX = receiver-0x30; then returns the dword at receiver+0x230. Bytes 56 8b f1 80 be 2c 02 00 … | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbc730` | `HouseCustomNameDataSource::GetValueAfterRebuildType123 ?` | HouseCustomNameDataSource | Virtual slot 2 of HouseCustomNameDataSource (vtable 018378c0; thiscall, no arguments): when the byte at receiver+0x22c is zero calls 00cbc400 with ECX = receiver-0x30; then returns the dword at receiver+0x230. Bytes 56 … | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cf9f10` | `HouseRatingDataSource::GetRowCount ?` | HouseRatingDataSource | Returns the number of house-rating rows for the UI. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf9f40` | `FeaturedDataSource::GetRowCount ?` | FeaturedDataSource | Returns the number of featured rows for the UI. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa2b0` | `HouseRatingDataSource::Cleanup ?` | HouseRatingDataSource | Cleans up the house-rating datasource base and table. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa330` | `FeaturedDataSource::Cleanup ?` | FeaturedDataSource | Cleans up the featured datasource base and table. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa440` | `HouseRatingDataSource::ScalarDeletingDtor ?` | HouseRatingDataSource | Scalar deleting destructor for HouseRatingDataSource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cfa4f0` | `FeaturedDataSource::ScalarDeletingDtor ?` | FeaturedDataSource | Scalar deleting destructor for FeaturedDataSource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0b690` | `— ⚠reject` |  | Handler registered as 'PreviewTextureAlias' in the House module (0x0182add0): previews a fixture texture alias. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b710` | `— ⚠reject` |  | Handler registered as 'PlaceFixture' in the House module (0x0182add0): places the currently pending fixture. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0b8d0` | `— ⚠reject` |  | Handler registered as 'ApplyFixtureCustomizationToGroup' in the House module (0x0182add0). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0ba80` | `— ⚠reject` |  | Handler registered as 'SetGridSpacingInCentimeters' in the House module (0x0182add0); note 027 pins it as the engine side of Housing:SetGrid() with the 25/5 cm grid values and the manager field +0x4188. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c410` | `— ⚠reject` |  | Handler registered as 'RequestPlayerHouses' in the House module (0x0182add0): asks the housing system for the target player's house list. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cbfb00` | `— ⚠reject` | HousingDataSource | Destroys the main housing data source and its owned subviews. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

