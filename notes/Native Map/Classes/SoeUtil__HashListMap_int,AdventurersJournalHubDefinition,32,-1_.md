# Class: SoeUtil::HashListMap<int,AdventurersJournalHubDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a42960` | `HashListMap_HubDefinition_Ctor` | ✅ | ctor | Constructs the journal hub-definition hash map: installs the instantiation vtable, zeroes count and list fields, memsets the 32 bucket pointers at +0x14. |
| `00a433b0` | `HashListMap_HubDefinition_AllocNode` | ✅ | virtual | Node allocator of the hub-definition map: same allocator chain as 00a43340 but with node size 0x38. |
| `00a433f0` | `HashListMap_HubDefinition_FreeNode` | ✅ | virtual | Node deallocator of the hub-definition map: one-argument wrapper calling FreeMemory(param_1, 4). |
| `00a436a0` | `HashListMap_HubDefinition_RemoveNode` | 🟢 | method | Node removal for the hub-definition map: bucket selected with (node->key at +0x30) & 0x1f, bucket chain walked through node+0x34, insertion-list unlink through the node's +0x28/+0x2c links, map count decrement at +0x10 … |
| `00a44910` | `HashListMap_HubDefinition_Clear` | 🟢 | method | Drains the hub map by repeatedly removing the head node through 00a436a0 until map+8 is null. |
| `00a44c50` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Destructor of the hub-definition map: installs the instantiation vtable and drains the list through 00a436a0. |
| `00a44d20` | `SoeUtil::HashListMap::ScalarDeletingDtor → SoeUtil::HashListMap<int,Adventurers…` | ✅ | dtor | Scalar deleting destructor of the hub-definition map: installs the vtable, drains through 00a436a0 and frees with _free when the delete flag is set. |
| `00a44f90` | `HashListMap_HubDefinition_Insert` | 🟢 | method | Inserts a hub node: allocates through vtable slot 1, zeroes words 0..9, stores the caller's key at node+0x30, links the node into the insertion list (prev at +0x2c, next at +0x28) and the bucket chain at +0x34 for bucke… |
| `00a45220` | `HashListMap_HubDefinition_DtorB` | ✅ | dtor | Second copy of the hub-definition map destructor: installs the vtable and drains through 00a436a0. |
