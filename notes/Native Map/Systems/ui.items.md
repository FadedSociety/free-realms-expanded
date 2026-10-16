# System: ui.items

35 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7e410` | `ItemAbilitiesDataSource::GetColumnName` | ItemAbilitiesDataSource | Names the five columns the item-abilities data source exposes. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7ea90` | `ItemAbilitiesDataSource::GetColumnValue` | ItemAbilitiesDataSource | Fills the item-abilities panel columns for the current item, using the ability definitions attached to the item definition. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7ede0` | `ItemAbilitiesDataSource::~ItemAbilitiesDataSource` | ItemAbilitiesDataSource | Tears down the item-abilities data source. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7ee70` | `ItemAbilitiesDataSource::DeletingDtorThunk254` | ItemAbilitiesDataSource | Lets the +0x254 subobject be destroyed through its own vtable group. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7ee80` | `ItemAbilitiesDataSource::ItemAbilitiesDataSource` | ItemAbilitiesDataSource | Creates the data source behind the item abilities panel. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7ef20` | `ItemAbilitiesDataSource::ScalarDeletingDtor` | ItemAbilitiesDataSource | Standard MSVC deleting destructor for the item-abilities data source. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a84370` | `ItemRecordCtor` | ItemRecord | Creates the small item reference records used by the job-customization equipment lists and their UI notifications. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00bb2050` | `ItemSortDataSource::ScalarDeletingDtor` | ItemSortDataSource | Deleting destructor so the object can be destroyed through the data-source base pointer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb20c0` | `BaseItemFilterManagerDtor` | BaseItemFilterManager<ClientItemDefinit… | Virtual destructor body of the item-filter manager base class. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb20d0` | `BaseItemFilterManagerScalarDeletingDtor` | BaseItemFilterManager<ClientItemDefinit… | Deleting destructor of the item-filter manager base class. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb2420` | `ClientItemFilterManager::~ClientItemFilterManager` | ClientItemFilterManager | Tears down the filter manager and its data source. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb2620` | `ClientItemFilterManager::ScalarDeletingDtor` | ClientItemFilterManager | Deleting destructor of the client item filter manager. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00c92c10` | `BaseItemDefinitionDataSource::ColumnName` | BaseItemDefinitionDataSource | Expose item-definition and inventory fields through the UI data-source API. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c92f70` | `ItemDefinitionDsUpdateDispatcher::DtorBody` | ItemDefinitionDsUpdateDispatcher | Clean up dispatcher singleton state during exception unwinding. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c93090` | `ItemDefinitionDsUpdateDispatcher::ScalarDeletingDtor → Singleton<ItemDefinition…` | ItemDefinitionDsUpdateDispatcher | Implement scalar-deleting destruction for the update dispatcher singleton. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c933f0` | `BaseItemDefinitionDataSource::CleanupSlot3` | BaseItemDefinitionDataSource | Provide a base-subobject adjustor thunk for item data-source destruction. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c93410` | `ItemDefinitionDsUpdateDispatcher::Ctor` | ItemDefinitionDsUpdateDispatcher | Initialize the item-definition data-source update dispatcher. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c93470` | `ItemDefinitionDsUpdateDispatcher::Dtor` | ItemDefinitionDsUpdateDispatcher | Destroy item-definition update dispatcher state. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c93990` | `BaseItemDefinitionDataSource::RemoveCachedDefinition` | BaseItemDefinitionDataSource | Remove an item definition from data-source cache and notify consumers. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c93c20` | `ItemDefinitionDsUpdateDispatcher::ScalarDeletingDtor` | ItemDefinitionDsUpdateDispatcher | Implement scalar-deleting destruction for the item-definition update dispatcher. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7e600` | `ItemAbilitiesDataSource::GetRowCount` | ItemAbilitiesDataSource | Count slot (vtable slot 2) of the abilities list, i.e. how many ability rows the current item definition has. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7e660` | `ItemAbilitiesDataSource::RefreshAfterAbilityDefinitionsCached` | ItemAbilitiesDataSource | Waits until every ability entry has a cached definition before refreshing the ItemAbilities list; helper 00a34d10 requests definitions that are missing. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a84450` | `ItemRecordListAppend` |  | Adds an item row to a job-customization equipment list. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00bb1dd0` | `ItemSortDataSource::GetStaticTableByIndex ⚠downgrade` | ItemSortDataSource | Vtable slot 1 accessor of the item-sort data source; the three fixed tables are the sort-type tables the sort UI asks for by index. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1e50` | `ItemSortDataSource::GetItemCount` | ItemSortDataSource | Vtable slot 2 count accessor of the item-sort data source; sibling slot 4 (00bb1ec0) uses exactly this value as the exclusive upper bound for its index argument. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1e80` | `ItemSortDataSource::RefreshList` | ItemSortDataSource | Rebuild step invoked from the ItemSortDataSource constructor 00bb1fd0: it pushes every entry of the source list into the sort model so the list view can re-render. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb24c0` | `ClientItemFilterManager::FilterMatch ⚠downgrade` | ClientItemFilterManager | The item filter predicate the inventory UI calls for every candidate item: true means the item passes the filter identified by param_3/param_4. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00c20730` | `ItemDefinitionSelect` |  | Handles the Lua-facing item-definition selection callback. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c93c90` | `ItemDefinitionDsUpdateDispatcher::NotifyListeners` | ItemDefinitionDsUpdateDispatcher | Notify item-definition data-source listeners about an update. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb1ec0` | `— ⚠reject` | ItemSortDataSource | Per-index activation of the item-sort list: the data source special-cases rows 0 and 1 and rejects every other row. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1f50` | `— ⚠reject` | ItemSortDataSource | Base cleanup of the item-sort data source; the scalar deleting destructor 00bb2050 calls it before freeing. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1fd0` | `— ⚠reject` | ItemSortDataSource | Creates the item-sort data source for the inventory list control under the name BaseClient.ItemSortTypes. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb2350` | `— ⚠reject` | ClientItemFilterManager | Builds the client-side item filter manager: definition tables (ItemFilters.txt via 0107f4e0) plus the ItemFilterDataSource object the UI queries. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00c93380` | `— ⚠reject` | BaseItemDefinitionDataSource | Invalidate or refresh item-definition rows when their source data changes. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c942c0` | `— ⚠reject` | BaseItemDefinitionDataSource | Remove an item definition from the shared cache and notify its consumers. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a7e660` | `ItemAbilitiesDataSource::RefreshAfterAbilityDefinitionsCached` | send | PacketTunneledClientPacket |  | Hypothesis: when an ability definition is absent locally, the server must supply the requested definition for this refresh path to complete. This depends on 00a34d10 requesting it through 00a34410 and the separate ability-definition response path; the server behavior is not established by this function. |
