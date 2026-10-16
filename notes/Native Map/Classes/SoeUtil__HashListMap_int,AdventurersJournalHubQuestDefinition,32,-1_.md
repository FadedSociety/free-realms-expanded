# Class: SoeUtil::HashListMap<int,AdventurersJournalHubQuestDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a429b0` | `HashListMap_HubQuestDefinition_Ctor` | ✅ | ctor | Constructs the journal hub-quest-definition hash map: installs the instantiation vtable, zeroes count and list fields, memsets the 32 bucket pointers at +0x14. |
| `00a43420` | `HashListMap_HubQuestDefinition_AllocNode` | ✅ | virtual | Node allocator of the hub-quest-definition map: same allocator chain with node size 0x1c. |
| `00a43460` | `HashListMap_HubQuestDefinition_FreeNode` | ✅ | virtual | Node deallocator of the hub-quest-definition map: one-argument wrapper calling FreeMemory(param_1, 4). |
| `00a43710` | `HashListMap_HubQuestDefinition_RemoveNode` | 🟢 | method | Node removal for the hub-quest-definition map: bucket selected with (node->key at +0x14) & 0x1f, chain walked through node+0x18, insertion-list unlink through the +0xc/+0x10 links, count decrement and a virtual node-dea… |
| `00a44930` | `HashListMap_HubQuestDefinition_Clear` | 🟢 | method | Drains the hub-quest map by repeatedly removing the head node through 00a43710 until map+8 is null. |
| `00a44c80` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Destructor of the hub-quest-definition map: installs the instantiation vtable and drains the list through 00a43710. |
| `00a44d60` | `SoeUtil::HashListMap::ScalarDeletingDtor → SoeUtil::HashListMap<int,Adventurers…` | ✅ | dtor | Scalar deleting destructor of the hub-quest-definition map: installs the vtable, drains through 00a43710 and frees with _free when the delete flag is set. |
| `00a45050` | `HashListMap_HubQuestDefinition_Insert` | 🟢 | method | Inserts a hub-quest node: allocates through vtable slot 1, zeroes the first words, stores the caller's key at node+0x14, links into the insertion list (prev at +0x10, next at +0xc) and the bucket chain at +0x18 for buck… |
| `00a45290` | `HashListMap_HubQuestDefinition_DtorB` | ✅ | dtor | Second copy of the hub-quest-definition map destructor: installs the vtable and drains through 00a43710. |
