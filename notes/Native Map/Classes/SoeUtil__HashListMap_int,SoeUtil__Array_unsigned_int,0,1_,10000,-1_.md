# Class: SoeUtil::HashListMap<int,SoeUtil::Array<unsigned_int,0,1>,10000,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa6af0` | `HashListMapCtor_Array10000` | ✅ | ctor | Stores SoeUtil::HashListMap<int,SoeUtil::Array<unsigned_int,0,1>,10000,-1>::vftable, zeroes head/tail/count and clears a 40000-byte bucket array with _memset. |
| `00ab8910` | `HashListMapArrayUint_ClearAll` | ✅ | function | Repeatedly takes the head node of the list at +8 while it is non-null: hashes the node key (node+0x18) modulo 10000 into the bucket array at this+0x14, unlinks it from the bucket chain and from the head/tail list (+8, +… |
| `00aba110` | `SoeUtil_HashListMap_ArrayUint_10000_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,Array<unsigned_int,0,1>,10000,-1> vftable 01818b38 at +0, then jumps to FUN_00ab8910 (ClearAll for that map) with the same this. The jump follows a store, so it is a tail call… |
| `00aba3c0` | `SoeUtil_HashListMap_ArrayUint_10000_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores vftable 01818b38, calls FUN_00ab8910 (ClearAll), then calls _free on this when bit 0 of the flag argument is set. |
| `00ababe0` | `SoeUtil_HashListMap_ArrayUint_10000_Dtor2` | ✅ | dtor | Destructor body: stamps the HashListMap<int,Array<unsigned_int,0,1>,10000,-1> vftable 01818b38 at +0 and calls FUN_00ab8910 (ClearAll) on this. |
| `00abc280` | `SoeUtil_HashListMap_ArrayUint_10000_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,Array<unsigned_int,0,1>,10000,-1> vftable 01818b38, zeroes the head, tail and count fields and the 10000 bucket heads (40000 bytes) through FUN_00d836a0, then stamps the HashList<… |
