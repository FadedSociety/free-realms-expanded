# Class: SoeUtil::Map

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b95390` | `Map_StringFixed16InvertedList_Destruct` | ✅ | dtor | Destructor body of the StringFixed<16>-keyed text-index map: reinstalls the map vtable, destroys the two root child chains with FUN_00b93800, runs FUN_00b90230, tears the tree nodes down through the map vtable slot 3 (c… |
| `00b954c0` | `Map_StringFixed16InvertedList_DeletingDestruct` | ✅ | dtor | MSVC scalar deleting destructor of the StringFixed<16>-keyed map: reinstalls the map vtable, destroys the two root child chains and the tree through FUN_00b93800/FUN_00b90230 and the vtable slot 3 call, zeroes param_1[1… |
| `00b95520` | `Map_StringFixed16InvertedList_Construct` | ✅ | ctor | Constructor body: stores the -1,0 map vtable, zeroes the tree head fields param_1[1] and param_1[2], then stores the 64,0 map vtable and calls FUN_00b8ca00; returns this. |
| `00b95f50` | `Map_Thunk_PopFreeNode2 → Map_StringFixed16InvertedList_Thunk_PopFreeNode` | ✅ | virtual | Adjustor thunk of 8 bytes: adds 0xc to this and jumps to 00b95e70 (free-node pop of the hash-list subobject at +0xc). |
| `00b95f60` | `MapHashList_Thunk_PushFreeNode2` | ✅ | virtual | Adjustor thunk of 8 bytes: adds 0xc to this and jumps to 00b95ee0 (free-node push of the hash-list subobject at +0xc). |
| `00b96290` | `— ⚠reject` | ⚪ | dtor | Compound destructor body: reinstalls the 64,0 map vtable, destroys the tree through FUN_00b93800/FUN_00b90230 and the map vtable slot 3 call, clears param_1[1..3] and frees the node-pool pointers param_1[5]/param_1[6] v… |
| `00b963a0` | `Map_StringFixed16InvertedList_DeletingDestruct2` | ✅ | dtor | Scalar deleting destructor: calls the compound map destructor 00b96290 and then _free(this) when bit 0 of param_2 is set. |
