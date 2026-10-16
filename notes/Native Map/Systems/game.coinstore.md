# System: game.coinstore

108 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (64)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b471e0` | `InitializeStoreDataManager` | ClientInGamePurchaseManager | One-time initialisation of the client's store (InGamePurchase) data: it wires the startup-data callback and loads the store data source before the merchant UI can be opened. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00c34b80` | `BundleIdSetMap1024Ctor` | SoeUtil::HashMap<int,SoeUtil::Set<SoeUt… | Constructs the 1024-bucket bundle-id set map used by store data. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34bd0` | `BundleGroupMap16Ctor` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Constructs the 16-bucket store bundle-group definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34c00` | `BundleCategoryGroupMap64Ctor` | SoeUtil::HashListMap<int,GameCommerce::… | Constructs the 64-bucket store category-group definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34c30` | `StoreShortcutMap64Ctor` | SoeUtil::HashListMap<int,GameCommerce::… | Constructs the 64-bucket store shortcut definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34c60` | `StorePortalCategoryMap64Ctor` | SoeUtil::HashListMap<int,GameCommerce::… | Constructs the 64-bucket portal-category definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34ca0` | `StoreBillboardMap64Ctor` | SoeUtil::HashListMap<int,GameCommerce::… | Constructs a 64-bucket store billboard definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34d30` | `BillboardPanelMap16Ctor` | SoeUtil::HashListMap<int,AppStoreBillbo… | Constructs the 16-bucket app-store billboard-panel map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c34ea0` | `BundleCategoryNodeMap128Ctor` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Constructs the 128-bucket map of store category-tree nodes. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c351e0` | `AppStoreBundleMap2048Ctor` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Constructs the 2048-bucket app-store bundle definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35710` | `GameCommerce::MarketingDataHandler::~MarketingDataHandler` | GameCommerce::MarketingDataHandler | Begins destruction of the marketing-data handler base object. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35720` | `GameCommerce::MarketingDataHandler::ScalarDeletingDtor` | GameCommerce::MarketingDataHandler | Provides the scalar deleting destructor for the marketing-data handler. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35d60` | `StoreBundleSetMapAllocate` | SoeUtil::HashMap<int,SoeUtil::Set<SoeUt… | Allocates a node for the 1024-bucket bundle-id set map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35da0` | `StoreBundleSetMapFree` | SoeUtil::HashMap<int,SoeUtil::Set<SoeUt… | Frees a node allocation for this store bundle-id map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35e20` | `StoreDefinitionMapAllocate` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Allocates storage for a store-definition map entry. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35e60` | `StoreDefinitionMapFree` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Frees storage allocated for a store-definition map entry. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35e80` | `AppStoreBundleMapAllocate` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Allocates a bundle definition map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35ec0` | `AppStoreBundleMapFree` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Frees storage allocated for a bundle definition map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35f00` | `BundleGroupMapAllocate` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Allocates a store bundle-group map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35f40` | `BundleGroupMapFree` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Frees storage allocated for a bundle-group map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35fb0` | `BundleCategoryNodeMapAllocate` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Allocates a node for the store bundle category tree map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35ff0` | `BundleCategoryNodeMapFree` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Frees a store bundle category tree map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36010` | `CategoryGroupMapAllocate` | SoeUtil::HashListMap<int,GameCommerce::… | Allocates a category-group map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36050` | `CategoryGroupMapFree` | SoeUtil::HashListMap<int,GameCommerce::… | Frees a category-group map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c360c0` | `ShortcutMapAllocate` | SoeUtil::HashListMap<int,GameCommerce::… | Allocates a store shortcut map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36100` | `ShortcutMapFree` | SoeUtil::HashListMap<int,GameCommerce::… | Frees a store shortcut map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36120` | `PortalCategoryMapAllocate` | SoeUtil::HashListMap<int,GameCommerce::… | Allocates a portal-category map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36160` | `PortalCategoryMapFree` | SoeUtil::HashListMap<int,GameCommerce::… | Frees a portal-category map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c361d0` | `BillboardDefinitionMapAllocate` | SoeUtil::HashListMap<int,GameCommerce::… | Allocates a store billboard definition map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36210` | `BillboardDefinitionMapFree` | SoeUtil::HashListMap<int,GameCommerce::… | Frees a store billboard definition map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36410` | `BillboardPanelMapAllocate` | SoeUtil::HashListMap<int,AppStoreBillbo… | Allocates a billboard-panel map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c36450` | `BillboardPanelMapFree` | SoeUtil::HashListMap<int,AppStoreBillbo… | Frees a billboard-panel map node. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c37430` | `StoreBillboardDefinitionDtor` | GameCommerce::StoreBillboardDefinition<… | Destroys a store billboard definition and its panel array. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c37490` | `StoreBillboardDefinition::IsValid` | GameCommerce::StoreBillboardDefinition<… | Checks billboard definition fields and every contained panel. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c375d0` | `StoreBillboardDefinition::ScalarDeletingDtor` | GameCommerce::StoreBillboardDefinition<… | Provides the scalar deleting destructor for a store billboard definition. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c37e20` | `StoreDefinition::RemoveBundleEntry` | GameCommerce::StoreDefinition<AppStoreB… | Clears bundle entries owned by a store definition. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c384e0` | `StoreBillboardDefinitionCtor` | GameCommerce::StoreBillboardDefinition<… | Constructs an empty store billboard definition. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38b70` | `PortalCategoryMapDtor` | SoeUtil::HashListMap<int,GameCommerce::… | Clears the portal-category map during destruction. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38cb0` | `StoreBillboardFormatDebug` | GameCommerce::StoreBillboardDefinition<… | Produces a diagnostic representation of a store billboard definition. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38f00` | `AppStoreBundleMapScalarDeletingDtor` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Provides the scalar deleting destructor for the app-store bundle map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38f40` | `PortalCategoryMapScalarDeletingDtor` | SoeUtil::HashListMap<int,GameCommerce::… | Provides the scalar deleting destructor for the portal-category map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c39080` | `AppStoreBundleMapCtor` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Constructs the pooled app-store bundle definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c3e8e0` | `StoreBundleCategoryDefinition::ReadFields` | StoreBundleCategoryDefinition | Parses GameCommerce category definition fields from the marketing data source. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3ea10` | `StoreBillboardPanelDefinition::ReadFields` | StoreBillboardPanelDefinition | Parses GameCommerce billboard panel definition fields from the marketing data source. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3ec30` | `StoreBundleCategoryMapDtor` | SoeUtil::HashListMap | Destroys the category tree hash-list map and all owned nodes. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3ed10` | `StoreBillboardPanelMapCtor` | SoeUtil::HashListMap | Constructs the GameCommerce billboard panel hash-list map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3f760` | `StoreBillboardPanelMapDtor` | SoeUtil::HashListMap | Destroys the GameCommerce billboard panel hash-list map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3f9e0` | `StoreDefinition::AssignFrom` | StoreDefinition | Virtual slot 4 of the store-definition template. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fa40` | `StoreDefinition::RebuildBundleMap` | StoreDefinition | Virtual slot 6 of the store-definition template. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fa70` | `StoreBundleCategoryTreeCtor` | StoreBundleCategoryTree | Initializes the GameCommerce store bundle category tree. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fb40` | `StoreBundleCategoryTree::AppendDebugText` | StoreBundleCategoryTree | Builds a readable debug listing for the category tree. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fc80` | `StoreBundleCategoryTreeDtor` | StoreBundleCategoryTree | Destroys the category tree and its owned map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3ff00` | `StoreBundleIdSetHashMapDtorBase` | SoeUtil::HashMap | Destroys the typed store-bundle ID hash map base. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c400c0` | `LoadStoreBillboardPanels` |  | Loads store billboard-panel definitions from marketing data. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c40410` | `LoadStoreBillboards` |  | Loads billboard definitions and groups their panels. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c406d0` | `StoreBundleIdSetHashMapDtor` | SoeUtil::HashMap | Destroys the typed set-valued store bundle hash map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41170` | `StoreBundleIdHashMapCtor` | SoeUtil::HashMap | Initializes the 1024-bucket store-bundle ID hash map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41270` | `ClientInGamePurchaseStoreManager::OnDataUpdate` | ClientInGamePurchaseStoreManager | Reloads the manager-owned in-game purchase definitions. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41350` | `StoreBundleIdSetHashCtor` | SoeUtil::Hash | Constructs the hash wrapper around the bundle-ID set map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41410` | `StoreDefinitionMapDtor` | SoeUtil::HashListMap | Destroys entries in the store-ID definition map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41480` | `StoreDefinitionMapCtor` | SoeUtil::HashListMap | Initializes the fixed-capacity store-ID definition map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41ed0` | `StoreDefinitionMapDtor` | SoeUtil::HashListMap | Destroys the fixed-capacity store-ID definition map. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41fc0` | `ClientInGamePurchaseStoreManagerCtor` | ClientInGamePurchaseStoreManager | Initializes all store data containers owned by the manager. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c42100` | `ClientInGamePurchaseStoreManagerDtor` | ClientInGamePurchaseStoreManager | Releases manager-owned store containers and day-offer objects. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |

## 🟢 likely (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b47270` | `ClearStoreCachesAndRequestStartupData` | ClientInGamePurchaseManager | Full invalidation of the client's store caches (stores, credit cards, country/state lists and both SKU lists) followed by a startup-data re-request; the offsets match exactly the members the response handlers rebuild. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b485a0` | `BeginStoreStreaming` | ClientInGamePurchaseManager | Wrapper that starts the store item download and records that it happened in the manager's state byte next to the ready/inited bytes 0x3e828/0x3e829 used by 00b471e0 and 00b47a50. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00c350e0` | `StorePortalCategoryDefinitionCtorSetId` | GameCommerce::StorePortalCategoryDefini… | Constructs a portal-category definition and copies the supplied integer into its id field. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35140` | `AppStoreBillboardPanelDefinitionCtorSetId` | AppStoreBillboardPanelDefinition | Constructs an app-store billboard panel and copies the supplied integer into its id field. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c388b0` | `FindStoreBundleDefinition` |  | Looks up an app-store bundle definition using store id and bundle id. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38910` | `FindStoreBundleDefinitionAlias` |  | Looks up an app-store bundle definition using store id and bundle id. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c389b0` | `AppStoreBundleMapClear` | SoeUtil::HashListMap<SoeUtil::StrongTyp… | Clears the app-store bundle definition map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38f80` | `StoreBillboardDefinitionSetId` | GameCommerce::StoreBillboardDefinition<… | Initializes the billboard definition's identifier field. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c3e890` | `InsertBundleDefinitions` |  | Method role relies on the callee's accepted label for store bundle insertion; this body supplies the iteration and field offsets. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3eb30` | `StoreDefinition::DeletingDtor` | StoreDefinition | Name follows the direct destructor callee; this wrapper itself does not expose its concrete template type. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3f930` | `StoreDefinition::AppendBundleEntry` | StoreDefinition | Template family is supported by the entry constructor callee; this method itself has no vtable or string anchor. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fb20` | `StoreBundleCategoryTree::CanRefresh ⚠downgrade` | StoreBundleCategoryTree | Virtual slot 0 in the category-tree vtable; the body is a predicate, so this slot is not assumed to be a deleting destructor. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fe20` | `StoreDefinitionMapFindOrCreate` | SoeUtil::HashListMap | Store-definition context is established by the caller that handles the StoreBundleBase packet; this body shows key lookup and creation. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c40360` | `UpdateStoreEntryRegistration → ReplaceOrRegisterStoreEntry` |  | Callers include item-of-day, bundle-of-day and StoreBundleBase handlers; this routine manages their definition registration. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41220` | `StoreBundleIdHashMapCleanup` | SoeUtil::HashMap | The identity follows its only callee; EH callers confirm cleanup role. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c74840` | `SetPruneNotInTable1024` |  | Store filter/sort result pruning helper. Neighbour callers RequestStoreFilter (00c77a60) and RequestStoreSort (00c77ca0) are the store request builders. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c74bc0` | `StoreFilterInsertMatches` |  | Store-item search helper that filters item ids through a predicate and collects matches. Neighbour caller 00c75bd0 is the store item-list handler. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c75bd0` | `StoreRebuildFilteredList` |  | Rebuilds a filtered store item list. Neighbour caller 00c00b50 is outside this shard; the store page code is the context. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |

## 🟡 speculative (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c380a0` | `StoreBillboardDefinitionScalarDeletingDtor ?` | GameCommerce::StoreBillboardDefinition<… | Provides the scalar deleting destructor for a store billboard definition. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c387f0` | `ValidateStoreShortcutTarget ? ⚠downgrade` |  | Checks whether a store shortcut points to an existing supported target. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c389e0` | `ClearBundleMapTree ? ⚠downgrade` |  | Destroys a bundle map tree and resets its wrapper state. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38a20` | `ClearStoreIdTree ? ⚠downgrade` |  | Destroys a store-id tree and resets its wrapper state. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38a60` | `ClearBundleGroupTree ? ⚠downgrade` |  | Destroys a bundle-group tree and resets its wrapper state. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38aa0` | `ClearShortcutTree ? ⚠rename` |  | Destroys a shortcut tree and resets its wrapper state. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c3f840` | `StoreBundleCategoryMap::ScalarDeletingDtor ?` | SoeUtil::HashListMap | Calls the category-map destructor and frees the map object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3f860` | `StoreBillboardPanelMap::ScalarDeletingDtor ?` | SoeUtil::HashListMap | Calls the billboard-panel map destructor and frees the map object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fcf0` | `StoreBundleCategoryTree::ScalarDeletingDtor ?` | StoreBundleCategoryTree | Calls the category-tree destructor and frees the object through the memory manager when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c40270` | `StoreBundleIdSetHashMap::ScalarDeletingDtor ?` | SoeUtil::HashMap | Calls the typed hash-map destructor and frees the object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41150` | `StoreBundleIdHashMap::ScalarDeletingDtor ?` | SoeUtil::HashMap | Runs the typed hash-map destructor, then frees the object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41200` | `StoreDefinitionMapClearEntries ?` | SoeUtil::HashListMap | Repeatedly removes the current head entry through the store-definition map erase routine until the map is empty. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c413b0` | `StoreBundleIdSetHash::ScalarDeletingDtor ?` | SoeUtil::Hash | Destroys the backing store-bundle ID hash map and frees the object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41440` | `StoreDefinitionMap::ScalarDeletingDtor ?` | SoeUtil::HashListMap | Destroys every store definition and frees the map object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c41fa0` | `StoreDefinitionMap::ScalarDeletingDtor ?` | SoeUtil::HashListMap | Calls the typed map destructor and frees the map object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c42240` | `ClientInGamePurchaseStoreManager::ScalarDeletingDtor ?` | ClientInGamePurchaseStoreManager | Calls the manager destructor and frees the object when flag bit 0 is set. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |

## ⚪ unknown (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c36f80` | `— ⚠reject` |  | Checks that every referenced bundle id resolves in the loaded store bundle map. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c375f0` | `— ⚠reject` |  | Calls the billboard-definition destructor for a stack-adjusted temporary object during exception cleanup. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c38ae0` | `— ⚠reject` |  | Applies a one-time relinking pass to the stored definition tree. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c3eb90` | `— ⚠reject` |  | Store context comes from its only caller; the implementation shows bucket and tree cleanup mechanics. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3f880` | `— ⚠reject` | SoeUtil::HashMap | Walks all 1024 buckets, destroys each set value, decrements entry count, and clears sentinel and tree storage. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fd50` | `— ⚠reject` | SoeUtil::HashListMap | The map specialization is identified by related constructors and manager members; this body has no direct vtable evidence. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3fe70` | `— ⚠reject` |  | The two callers are store-manager cleanup/update paths; behavior here describes index maintenance only. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c3ff10` | `— ⚠reject` |  | Loads store portal-category definitions from marketing data. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c402b0` | `— ⚠reject` |  | Feature is supported by the caller's store-definition manager role and the called store-index routines. | [[Native Map/Shards/S0391 00c3df10-00c423a0|S0391]] |
| `00c77230` | `— ⚠reject` |  | Purchase path for a store item: resolves the definition, builds the purchase record and delegates the send to SendBuyItemRequest (00c76a00), which sends a PacketTunneledClientPacket. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c77230` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
