# System: net.promotions

39 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a05bb0` | `ClientPromotionalManager::LogLevel4` | ClientPromotionalManager | Second log override of the promotions manager (slot 1 of its vtable). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05bd0` | `ClientPromotionalManager::LogLevel3` | ClientPromotionalManager | Third log override of the promotions manager (slot 2 of its vtable). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05c50` | `HashMap_PromotionalBundleData64_Construct` | SoeUtil::HashMap<int,PromotionalBundleD… | Initialises a 64-bucket hash map keyed by an int bundle id holding PromotionalBundleData nodes. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05cd0` | `HashListMap_PromotionalBundleDatasource256_Construct` | SoeUtil::HashListMap<int,PromotionalBun… | Initialises the 256-bucket map that holds the registered promotional datasource objects by name key. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06220` | `HashMap_PromotionalBundleData64_NewNode` | SoeUtil::HashMap<int,PromotionalBundleD… | Virtual slot 1 of the bundle map; nodes are 0x134 bytes, which matches the 0x130 bucket-link offset used when the map is drained. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06260` | `HashMap_PromotionalBundleData64_FreeNode` | SoeUtil::HashMap<int,PromotionalBundleD… | Virtual slot 2 of the bundle map, invoked by the drain loop 00a06450 and the single-node removal 00a06b50. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06330` | `HashListMap_PromotionalBundleDatasource256_NewNode` | SoeUtil::HashListMap<int,PromotionalBun… | Virtual slot 1 of the datasource map; 0x14 bytes matches the node whose fields are value/next/prev/hash/bucket-next. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06370` | `HashListMap_PromotionalBundleDatasource256_FreeNode` | SoeUtil::HashListMap<int,PromotionalBun… | Virtual slot 2 of the datasource map; the removal routine 00a06290 invokes it after unlinking. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06b40` | `HashMap_PromotionalBundleData64_Destruct` | SoeUtil::HashMap<int,PromotionalBundleD… | Destructor of the bundle map, reached from four exception-unwind funclets. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06c00` | `HashMap_PromotionalBundleData64_ScalarDeletingDtor` | SoeUtil::HashMap<int,PromotionalBundleD… | Slot 0 of the bundle map vtable. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06f10` | `HashMap_PromotionalBundleData64_Destruct_B` | SoeUtil::HashMap<int,PromotionalBundleD… | Destructor of the bundle map as embedded in a larger class, reached from exception-unwind funclets 015abe99/015ac079. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06f60` | `Hash_PromotionalBundleData64_Construct` | SoeUtil::Hash<PromotionalBundleData,64,… | Constructs the Hash-derived promotional bundle map; the deleting destructor restores the HashMap vtable before draining it. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06fe0` | `Hash_PromotionalBundleData64_ScalarDeletingDtor` | SoeUtil::Hash<PromotionalBundleData,64,… | Slot 0 of the Hash<PromotionalBundleData,64,-1> vtable; frees a heap-allocated bundle map. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a070b0` | `HashListMap_PromotionalBundleDatasource256_Destruct` | SoeUtil::HashListMap<int,PromotionalBun… | Destructor of the datasource map, reached from exception-unwind funclets. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a070e0` | `HashListMap_PromotionalBundleDatasource256_ScalarDeletingDtor` | SoeUtil::HashListMap<int,PromotionalBun… | Slot 0 of the datasource map vtable. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07490` | `HashListMap_PromotionalBundleDatasource256_Destruct_B` | SoeUtil::HashListMap<int,PromotionalBun… | Destructor used by the manager destructor 00a07500 and by 00a076e0, and reached from exception-unwind funclets. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07500` | `ClientPromotionalManager::~ClientPromotionalManager` | ClientPromotionalManager | Tears down the client promotions subsystem: datasources, cached bundles and the non-trivial base object. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07660` | `HashListMap_PromotionalBundleDatasource256_Construct_B` | SoeUtil::HashListMap<int,PromotionalBun… | Builds the datasource map at manager+0x308 from the manager constructor 00a07b80. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a076e0` | `HashList_PromotionalBundleDatasource_ScalarDeletingDtor` | SoeUtil::HashList<PromotionalBundleData… | Slot 0 of the HashList<PromotionalBundleDatasource*,256,-1> vtable. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07b80` | `ClientPromotionalManager::ClientPromotionalManager` | ClientPromotionalManager | Creates the client-side promotions subsystem: bundle cache, datasource registry and its Lua-facing hooks. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a088c0` | `RedeemCode_HandleSelectOptionResult` |  | Reaction to the server answer for selecting a redemption option; runs the shared result processing afterwards. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a05b80` | `SetFlagByteAt0x1F4` |  | Setter for the flag byte at 0x1F4 of the promotions manager object: the manager constructor 00a07b80 writes 1 there and the packet-dispatch case handler 00904560 clears it via this setter. The class binding is neighbour… | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05b90` | `ClientPromotionalManager::LogLevel5 ⚠downgrade` | ClientPromotionalManager | Highest-severity log override of the promotions manager (slot 0 of its vtable); overrides the Loggable interface so manager messages land in Promotions.log. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05bf0` | `ClientPromotionalManager::LogLevel2 ⚠downgrade` | ClientPromotionalManager | Lowest-severity log override of the promotions manager (slot 3 of its vtable); the deserialiser 00a08b10 calls it through vtable slot 3 with the packet-failure text. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05ed0` | `PairNode_ConstructKeyAndData` |  | Node-payload constructor used by the bundle map insert 00a07700, which passes the bundle id and the source data record. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a060b0` | `NodePayload_InitWithKey` |  | Initialises one hash-list-map node payload from a key pointer; the insert 00a07930 uses it before linking the node into its bucket. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a066b0` | `ScalarDelete_a066b0` |  | Deleting destructor for the class whose only teardown step is resetting the Loggable vtable (the PromotionalItemData-style payload). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a069d0` | `FindNodeByStringKey` |  | Find a registered datasource object by its name; the found node is used by the unregister path 00a06a50. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06a50` | `UnregisterAndFreeObjectByKey` |  | Remove/replace path used before re-registering a datasource under the same name; keeps the global id table and the reference-counted object consistent. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06be0` | `HashListMap256_Drain` |  | Clear routine for the promotions datasource map used by destructor fragments. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07700` | `HashMap_PromotionalBundleData64_Insert` |  | Cache insert used by the bundle deserialiser 00a08b10 after removing any previous entry with the same id. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07d10` | `HashListMap256_Insert` |  | Insert used by the datasource registration path 00a07ff0, which passes the name hash and the newly created object. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07ff0` | `RegisterNamedDatasource` |  | Creates the data objects the client later uses for member-upsell bundles; the client message dispatcher 009471a0 calls it for names such as BaseClient.MemberUpgrade.PetTrial and BaseClient.MemberUpgrade.MountTrial. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a05d80` | `— ⚠reject` |  | Constructor fragment for the embedded bundle-item holder at offset 4 of its owner; the owner class is not proven here, so the name stays mechanical. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06290` | `— ⚠reject` |  | Node removal for the promotions datasource map (map object at manager+0x308); called by the manager destructor, the drain helpers and the re-register path. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06450` | `— ⚠reject` | SoeUtil::HashMap<int,PromotionalBundleD… | Clear/destroy-contents routine of the bundle map, used by its destructors and by the manager destructor. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06ab0` | `— ⚠reject` |  | Membership test over the manager bundle map: true when any cached bundle payload lists the given id. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a06b50` | `— ⚠reject` |  | Replacement path of the bundle deserialiser 00a08b10, which removes a cached bundle with the same id before inserting the newly parsed one. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08520` | `— ⚠reject` |  | Turns a server redemption response into the OnCodeRedemption script event and the cached result rows the UI shows. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

