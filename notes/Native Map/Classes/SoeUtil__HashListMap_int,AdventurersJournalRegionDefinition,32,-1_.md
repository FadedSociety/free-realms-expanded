# Class: SoeUtil::HashListMap<int,AdventurersJournalRegionDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a42910` | `HashListMap_RegionDefinition_Ctor` | ✅ | ctor | Constructs the journal region-definition hash map: installs the instantiation vtable, zeroes the count and list fields at +0x10/+8/+0xc, memsets the 32 bucket pointers (0x80 bytes) at +0x14 and clears +4. |
| `00a43340` | `HashListMap_RegionDefinition_AllocNode` | ✅ | virtual | Vtable slot 1 (node allocator) of the region-definition map: pushes the node size 0x30 and alignment 4, obtains the current memory manager through FUN_006fb9c0/FUN_006fb860 and calls its allocate slot 1, falling back to… |
| `00a43380` | `HashListMap_RegionDefinition_FreeNode` | ✅ | virtual | Vtable slot 2 (node deallocator) of the region-definition map: a one-argument wrapper that calls FreeMemory(param_1, 4) (18-byte body, mov eax,[esp+4]; push 4; push eax; call; ret 4). |
| `00a43630` | `HashListMap_RegionDefinition_RemoveNode` | 🟢 | method | Removes param_2 from the region-definition map: selects the bucket with (node->key at +0x28) & 0x1f, walks the bucket chain through node+0x2c to relink the predecessor, unlinks the node from the insertion list through i… |
| `00a448f0` | `HashListMap_RegionDefinition_Clear` | 🟢 | method | Drains the map: while the head pointer at map+8 is non-null it removes the head node through the region removal helper 00a43630, which updates the head in place. |
| `00a44c20` | `SoeUtil::HashListMap_RegionDefinition::~HashListMap` | ✅ | dtor | Installs the exact region-definition HashListMap vtable and repeatedly removes the insertion-list head through 00a43630 until map+8 is null. |
| `00a44ce0` | `SoeUtil::HashListMap::ScalarDeletingDtor → SoeUtil::HashListMap<int,Adventurers…` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the region-definition map: installs the vtable, drains the list through 00a43630 and frees the map object with _free when bit 0 of the delete flag is set. |
| `00a44ee0` | `HashListMap_RegionDefinition_Insert` | 🟢 | method | Inserts a region node: allocates through vtable slot 1, zeroes words 0..6 of the node, stores the caller's key at node+0x28, links the node at the insertion-list tail (prev at +0x24, next at +0x20, map head +8 / tail +0… |
| `00a451b0` | `HashListMap_RegionDefinition_DtorB` | ✅ | dtor | Second copy of the region-definition map destructor (with its own EH frame): installs the instantiation vtable and drains the insertion list through 00a43630. |
