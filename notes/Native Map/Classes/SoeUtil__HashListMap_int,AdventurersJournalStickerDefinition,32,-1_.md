# Class: SoeUtil::HashListMap<int,AdventurersJournalStickerDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a42a20` | `HashListMap_StickerDefinition_Ctor` | ✅ | ctor | Constructs the journal sticker-definition hash map: installs the instantiation vtable, zeroes count and list fields, memsets the 32 bucket pointers at +0x14. |
| `00a43490` | `HashListMap_StickerDefinition_AllocNode` | ✅ | virtual | Node allocator of the sticker-definition map: same allocator chain with node size 0x34. |
| `00a434d0` | `HashListMap_StickerDefinition_FreeNode` | ✅ | virtual | Node deallocator of the sticker-definition map: one-argument wrapper calling FreeMemory(param_1, 4). |
| `00a43780` | `HashListMap_StickerDefinition_RemoveNode` | 🟢 | method | Node removal for the sticker-definition map: bucket selected with (node->key at +0x2c) & 0x1f, chain walked through node+0x30, insertion-list unlink through the +0x24/+0x28 links, count decrement and a virtual node-deal… |
| `00a44950` | `HashListMap_StickerDefinition_Clear` | 🟢 | method | Drains the sticker map by repeatedly removing the head node through 00a43780 until map+8 is null. |
| `00a44cb0` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Destructor of the sticker-definition map: installs the instantiation vtable and drains the list through 00a43780. |
| `00a44da0` | `SoeUtil::HashListMap::ScalarDeletingDtor → SoeUtil::HashListMap<int,Adventurers…` | ✅ | dtor | Scalar deleting destructor of the sticker-definition map: installs the vtable, drains through 00a43780 and frees with _free when the delete flag is set. |
| `00a450f0` | `HashListMap_StickerDefinition_Insert` | 🟢 | method | Inserts a sticker node: allocates through vtable slot 1, zeroes words 0..8, stores the caller's key at node+0x2c, links into the insertion list (prev at +0x28, next at +0x24) and the bucket chain at +0x30 for bucket key… |
| `00a45300` | `HashListMap_StickerDefinition_DtorB` | ✅ | dtor | Second copy of the sticker-definition map destructor: installs the vtable and drains through 00a43780. |
