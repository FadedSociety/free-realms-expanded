# Class: SoeUtil::HashListMap<int,DataSourceRef<ItemGroupItemsDataSource>,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3b5f0` | `HashListMap_ItemGroupItemsDataSource8_AllocNode` | 🟢 | virtual | Vtable slot-1 node allocator of the item-group data-source map: calls FUN_006fb9c0 and FUN_006fb860, uses the returned memory manager's virtual slot 1 when available, otherwise the global manager DAT_01bdf280, otherwise… |
| `00a3b630` | `HashListMap_ItemGroupItemsDataSource8_FreeNode` | ✅ | virtual | One-argument node deallocator: calls FUN_00401c00(ptr,4) and returns, paired with the slot-1 allocator of the same map. |
| `00a3c090` | `HashListMap_ItemGroupItemsDataSource8_NodeScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor of one 0x14-byte map node: calls FUN_00a3bb00 (release of the data-source reference value) and then calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
| `00a3c130` | `HashListMap_ItemGroupItemsDataSource8_Erase` | 🟢 | method | Erase node (this=map, arg=node): selects the bucket by (node key at +0xc) & 7 within the 8 bucket heads at map+0x14, walks the +0x10 chain and splices the node out, then splices it out of the ordered list through the +4… |
| `00a3c5c0` | `— ⚠reject` | ⚪ | method | Insert into the 8-bucket item-group data-source map: allocates a node through the map's vtable slot +4, initialises it with 00a3b7b0 (source dword copied to node+0xc), links it at the tail (prev node+4, successor writte… |
| `00a3c660` | `HashListMap_ItemGroupItemsDataSource8_RemoveAll` | 🟢 | method | Drains the map by repeatedly erasing the node at the list head (map+8) with 00a3c130 until the head is null. |
| `00a3d210` | `HashListMap_ItemGroupItemsDataSource8_Destruct` | ✅ | dtor | Destructor body of the item-group data-source map: installs the map vtable at +0 and drains every node from the head at +8 through the erase routine 00a3c130. |
| `00a3d240` | `HashListMap_ItemGroupItemsDataSource8_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: installs the map vtable, drains all nodes through 00a3c130, and calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
| `00a3d280` | `HashListMap_ItemGroupItemsDataSource8_Destruct_B` | ✅ | dtor | Second emission of the map teardown body: under an SEH frame it installs the map vtable and drains the nodes at +8 through 00a3c130 without freeing the object. |
| `00a3d640` | `HashListMap_ItemGroupItemsDataSource8_Construct` | ✅ | ctor | Constructor of the 8-bucket item-group data-source map: stores the map vtable at +0, zeroes the fields +4..+0x30 (count, head, tail and the eight bucket heads), and finally overwrites +0 with SoeUtil::HashList<DataSourc… |
