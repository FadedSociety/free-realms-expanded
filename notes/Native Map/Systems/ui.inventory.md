# System: ui.inventory

77 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (26)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6d830` | `InventoryCategoriesDataSource::CategoriesTreeNode::KeyNameForFlag` | InventoryCategoriesDataSource::Categori… | Key-name label returned by the category tree node (vtable slot 4). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6d960` | `BaseInventoryDataSource::ColumnLabel_PriceNoSale` | BaseInventoryDataSource | Column header label for the price and no-sale columns of inventory data sources. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6db10` | `InventoryDataSource::ColumnLabel_Kinds1dTo1c` | InventoryDataSource | Column label text for the rental and price/no-sale columns of inventory data sources. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6de90` | `InventoryCategoriesDataSource::ScalarDeletingDtor` | InventoryCategoriesDataSource | Deleting-destructor entry (vtable slot 0) of InventoryCategoriesDataSource. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6deb0` | `InventoryDataSource::InventoryDataSource` | InventoryDataSource | Constructs an inventory data source that owns a text member at +0x2ec. Research note 082 identifies the instance built for BaseClient.SearchInventoryItemsData as this constructor's output, and 00a6e910 reads that member… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6df70` | `InventoryDataSource::ReleaseTextMember2ec → InventoryDataSource::~InventoryData…` | InventoryDataSource | Member teardown of the text string (+0x2ec) owned by InventoryDataSource. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e000` | `StorageDataSource::StorageDataSource` | StorageDataSource | Constructor of the storage data source, which derives from InventoryDataSource. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e0f0` | `InventoryDataSource::ScalarDeletingDtor` | InventoryDataSource | Deleting-destructor entry (vtable slot 0) of InventoryDataSource. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e170` | `InventoryCategoriesDataSource::CategoriesTreeNode::CategoriesTreeNode` | InventoryCategoriesDataSource::Categori… | Builds one category tree node whose display name is looked up in the string table by id. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e260` | `InventoryCategoriesDataSource::CategoriesTreeNode::ReleaseNameString → Inventor…` | InventoryCategoriesDataSource::Categori… | Member teardown for the category node's name string. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e2e0` | `InventoryCategoriesDataSource::CategoriesTreeNode::ScalarDeletingDtor` | InventoryCategoriesDataSource::Categori… | Deleting-destructor entry (vtable slot 0) of the category tree node. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e9d0` | `InventoryCategoriesDataSource::InventoryCategoriesDataSource` | InventoryCategoriesDataSource | Constructs the category data source named BaseClient.InventoryCategoriesData and builds its tree. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6ea40` | `InventoryScreen_SelectCategoryEvent` |  | Sends the inventory screen's category-selection event to the UI script. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6ebf0` | `InventoryScreen_SelectItemEvent` |  | Sends the inventory screen's item-selection event to the UI script. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a7a530` | `InventoryNavigationDataSource::ItemGroupNode::Slot1_ReturnThis` | InventoryNavigationDataSource::ItemGrou… | Identity accessor for the ItemGroupNode interface at virtual slot 1. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7a5e0` | `InventoryNavigationDataSource::Slot1_ColumnName` | InventoryNavigationDataSource | Supplies the column name for each column of the inventory navigation data source. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7b380` | `InventoryNavigationDataSource::CategoryNode::Slot0_ReturnThis` | InventoryNavigationDataSource::Category… | Identity accessor at virtual slot 0 of the CategoryNode vtable. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7ba90` | `ItemGroupNode::MatchesFilter` | InventoryNavigationDataSource::ItemGrou… | Slot-3 predicate on InventoryNavigationDataSource::ItemGroupNode used during the datasource's filtered index rebuild; it checks record flags and a slot-kind selector, not the passed text pointer. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7bb70` | `InventoryNavigationDataSource::GetEntryCount` | InventoryNavigationDataSource | Answers the UI list's count query, rebuilding the visible-row index first when it is marked stale. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7bb90` | `InventoryNavigationDataSource::GetEntryValue` | InventoryNavigationDataSource | Fetches one cell of one visible row for the inventory list UI by delegating to the row node's column getter. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7bfe0` | `InventoryNavigationDataSource::RefreshNodeValues` | InventoryNavigationDataSource | Keeps the inventory rows' displayed values current after the underlying item state changes, pushing only changed values into the row widgets. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c220` | `CategoryNode::GetColumnValue` | InventoryNavigationDataSource::Category… | Feeds the category rows of the inventory list with either the definition's data or the node's cached display fields, refreshing the node's widget first when the corresponding field changed. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c6d0` | `InventoryNavigationDataSource::DeletingDtorThunk254` | InventoryNavigationDataSource | Vtable slot that lets the +0x254 subobject be destroyed through its own vtable group. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c6e0` | `InventoryNavigationDataSource::InventoryNavigationDataSource` | InventoryNavigationDataSource | Creates the client's inventory navigation data source that the inventory screen binds to, named Inventory.Navigation and advertised as BaseClient.Inventory. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c7e0` | `InventoryNavigationDataSource::ScalarDeletingDtor` | InventoryNavigationDataSource | Standard MSVC deleting destructor for the inventory navigation data source. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00c09060` | `RegisterInventoryScreenModule` |  | Registers the client 'InventoryScreen' Lua module (category selection, item groups, show/hide item, filters, SearchInventoryItems, RequestItemRequirementStatus). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (24)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6d870` | `BaseInventoryDataSource::BaseInventoryDataSource ⚠downgrade` | BaseInventoryDataSource | Base constructor of inventory data sources; the owner pointer at +0x2dc is read by the row predicate and the storage overrides. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6db90` | `StorageDataSource::RowPassesFilter` | StorageDataSource | Storage-list override of the row predicate in vtable slot 15, which differs from the base predicate 00a6d9b0. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e430` | `InventoryCategoriesDataSource::CategoriesTreeNode::FormatValueField` | InventoryCategoriesDataSource::Categori… | Formats one of the two stored values of a category tree node for display. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e480` | `InventoryCategoriesDataSource::RebuildCategoryTree` | InventoryCategoriesDataSource | Rebuilds the category tree from its keyed source table. It is called by the constructor 00a6e9d0 and by the reference-data step 00a6e5d0. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e5d0` | `CategoryReferenceData_RebuildAndNotify` |  | Item-category reference-data step: rebuilds the category tree, then notifies the object through slot 4. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e5f0` | `BaseInventoryDataSource::GetRowValueKind1bOr1c ⚠downgrade` | BaseInventoryDataSource | Row value getter for the Price and NoSale columns, deferring first to the base getter 00cb1160. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e6b0` | `InventoryDataSource::GetRowValueKind1dOr1e` | InventoryDataSource | Overrides base slot 4 for kinds 0x1d and 0x1e (a flag text and an elapsed-time value) and forwards every other kind to the base getter. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e7b0` | `RebuildIdListByVSlot15` |  | Recomputes the visible-row id array with the vtable slot-15 row predicate. It is driven by the search refresh 00a6e910. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a7af90` | `EntryIndex_AppendRecursive` |  | Appends a visible inventory entry to the entry index, indexing its parent chain first. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7b4d0` | `InventoryNavigationDataSource::BuildChildNodes` |  | Builds the child rows (category nodes, then item-group nodes) under one inventory category during the first-time build of the navigation tree. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7b930` | `InventoryNavigationDataSource::RebuildEntryIndex` | InventoryNavigationDataSource | Rebuilds the inventory navigation row index, either including all nodes or retaining nodes accepted by their slot-3 predicate. The meanings of the filter byte and selector are unresolved; the code does not establish tex… | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c1a0` | `WriteSharedColumnValue` |  | Shared fallback that writes three object dwords to the output cell; CategoryNode and ItemGroupNode call it for their default columns. The field semantics are unknown. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c310` | `ItemGroupNode::GetColumnValue ⚠downgrade` | InventoryNavigationDataSource::ItemGrou… | Supplies the item group rows of the inventory list with packed item fields (icons, counts, type/flag bytes, the item id at [0x12] resolved through FUN_00bb51a0) so the UI can draw each entry. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00bdc810` | `ResolveNameThenUseItem` |  | Name-based wrapper around the item-use path; its caller or script binding is not established in this function map. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00c09080` | `Inventory_FilterItemGroupByPage` |  | Handler registered as 'FilterItemGroupByPage' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c090c0` | `Inventory_SelectItemGroup` |  | Handler registered as 'SelectItemGroup' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c091c0` | `Inventory_GetGroupIdFromItemId` |  | Handler registered as 'GetGroupIdFromItemId' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09280` | `Inventory_CreateItemGroup` |  | Handler registered as 'CreateItemGroup' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c092c0` | `Inventory_SelectInventoryCategory` |  | Handler registered as 'SelectInventoryCategory' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09300` | `Inventory_SelectAltInventoryCategory` |  | Handler registered as 'SelectAltInventoryCategory' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09340` | `Inventory_SelectInventoryGenderFilter` |  | Handler registered as 'SelectInventoryGenderFilter' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09380` | `Inventory_HideItem` |  | Handler registered as 'HideItem' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09d30` | `Inventory_RequestItemRequirementStatus` |  | Handler registered as 'RequestItemRequirementStatus' in the InventoryScreen module (0x0182a4a8): asks the server whether the character meets an item's requirements. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cfd9a0` | `MerchantBuyInventoryDataSource::IsRowEligible ⚠downgrade` | MerchantBuyInventoryDataSource | Filters merchant buy-inventory rows for display or selection. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |

## 🟡 speculative (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6d990` | `BaseInventoryDataSource::QueryUnlessKind11 ?` | BaseInventoryDataSource | Virtual slot 2 (17 bytes, fastcall with ECX=this): when the dword at this+0x2b0 equals 0x11 it returns 0; otherwise it tail-jumps to 00cb0c60 with the same this. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6daa0` | `SetField2e0_NotifyPending ?` |  | If the dword at this+0x2e0 differs from param_2 (35 bytes: mov eax,[esp+4]; cmp [ecx+0x2e0],eax; je; mov [ecx+0x2e0],eax; mov byte [ecx+0x2d0],1; call through vtable slot 4 at +0x10). Stores the value, sets the byte fla… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6dad0` | `SetField2e4_NotifyPending ?` |  | Same as 00a6daa0 for the dword at this+0x2e4 (param_1[0xb9]): stores param_2 when it differs, sets the byte flag at +0x2d0 to 1 and calls virtual slot 4 of this with no arguments (35 bytes). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6db60` | `InventoryDataSource::ForwardToSlot14 ?` | InventoryDataSource | Virtual slot 17 (+0x44) thunk, 7 bytes: loads the vtable pointer from ECX, reads the entry at +0x38 (virtual slot 14) and jumps to it with the caller's arguments unchanged (indirect jmp, not a call). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6dcf0` | `BaseInventoryDataSource::SecondaryAdjustorToDtor ?` | BaseInventoryDataSource | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6de20 (BaseInventoryDataSource scalar deleting destructor). It is slot 1 of the secondary BaseInventoryDataSource vtable at 018157b4. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6dd70` | `HashTree_ContainsRecursive ?` |  | Recursive membership test over two chained hash tables. Returns 1 when param_2 is 0 or equals param_3. Otherwise it looks param_2 up with 00a6dc40, walks the 100-bucket table at *(this+0x38)+0x14 for the matching key, r… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6df60` | `InventoryDataSource::SecondaryAdjustorToDtor ?` | InventoryDataSource | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6e0f0 (InventoryDataSource scalar deleting destructor). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e090` | `StorageDataSource::SecondaryAdjustorToDtor ?` | StorageDataSource | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6e110 (StorageDataSource scalar deleting destructor). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e0a0` | `Wrapper_ReleaseTextMember2ec ?` |  | Builds an SEH frame (push -1, handler, fs:[0] link, cookie XOR) that saves ECX, then calls 00a6df70 with that same this. The cookie-XOR argument shown in the pseudocode is frame-setup noise, not an object pointer. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e230` | `InventoryCategoriesDataSource::CategoriesTreeNode::NameFieldAddress ?` | InventoryCategoriesDataSource::Categori… | Returns this+0x24, the address of the node's name IString member (lea eax,[ecx+0x24]; ret; 4 bytes). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e240` | `InventoryCategoriesDataSource::CategoriesTreeNode::GetValueAt1c ?` | InventoryCategoriesDataSource::Categori… | Returns the dword at this+0x1c (mov eax,[ecx+0x1c]; ret; 4 bytes). That field is the fourth constructor argument stored by 00a6e170. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e300` | `KeyedNodeTable_BuildChildren ?` |  | Recursive child builder. Looks up the node for key param_2+0x18 in the 20-bucket table at +0x1b4 of *(this+0x38), then walks the 100-bucket table (key at node+4) for matching entries. For each match it allocates 0x34 by… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00cb09d0` | `InventoryColumnName ? ⚠rename` |  | Provides item column labels to datasource implementations. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

## ⚪ unknown (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6d9b0` | `— ⚠reject` | BaseInventoryDataSource | Per-row predicate that the visible-row rebuild (00a6e7b0) calls through vtable slot 15 to decide which rows to keep. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6dd00` | `— ⚠reject` | InventoryCategoriesDataSource | Destructor body of the category data source; releases the owned object at +0x30. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6de20` | `— ⚠reject` | BaseInventoryDataSource | Deleting-destructor entry (vtable slot 0) of BaseInventoryDataSource. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e110` | `— ⚠reject` | StorageDataSource | Deleting-destructor entry (vtable slot 0) of StorageDataSource. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6e910` | `— ⚠reject` | InventoryDataSource | Virtual refresh of the filtered (search) row list of an inventory data source. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a76a60` | `— ⚠reject` |  | Builds the InventoryItemTooltip row table for an item from its fields (icon, description, membership and trade flags, category, power, prices, equip requirements). | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7b2f0` | `— ⚠reject` | InventoryNavigationDataSource::Category… | Constructs an inventory category node with three ids and an empty int child list. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |
| `00a7bbf0` | `— ⚠reject` | InventoryNavigationDataSource | Builds the inventory UI's category list once from the item category definitions, sorted by the definition's order key, each category node pulling in its item groups. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c530` | `— ⚠reject` | InventoryNavigationDataSource | Tears down the inventory navigation data source; its node-array buffer has a count-guarded _free path, while the index buffer uses the SoeUtil memory-manager release path. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00c09220` | `— ⚠reject` |  | Handler registered as 'GetItemDefinitionIndex' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09720` | `— ⚠reject` |  | Handler registered as 'ShowItemGroup' in the InventoryScreen module (0x0182a4a8): previews a group of items as world objects. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c098b0` | `— ⚠reject` |  | Handler registered as 'ShowItemId' in the InventoryScreen module (0x0182a4a8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c09a80` | `— ⚠reject` |  | Handler registered as 'SearchInventoryItems' in the InventoryScreen module (0x0182a4a8); the project notes record that this text-search path crashed once, so it is the client's inventory text search entry. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cb0b30` | `— ⚠reject` | BaseInventoryDataSource | Marks datasource for refresh and invokes its callback. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

