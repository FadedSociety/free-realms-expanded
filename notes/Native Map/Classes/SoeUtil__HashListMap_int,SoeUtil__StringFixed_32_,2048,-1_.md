# Class: SoeUtil::HashListMap<int,SoeUtil::StringFixed<32>,2048,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d05370` | `HashListMap2048Ctor` | ✅ | ctor | Stores the HashListMap<int,SoeUtil::StringFixed<32>,2048,-1> vtable at offset 0, zeroes +0x4, +0x8, +0xc and +0x10, and zeroes 0x2000 bytes of bucket storage at +0x14 with _memset. |
| `00d05aa0` | `HashListBufferRealloc44` | ✅ | virtual | Virtual slot [1] of the StringFixed<32> hash list: calls 006fb9c0, then the same +4 vtable steps on 006fb860 and DAT_01bdf280 as 00d05a20, then allocates 0x44 bytes with alignment 4 via 006fb950. |
| `00d05ae0` | `HashListFreeBlock4` | ✅ | virtual | Virtual slot [2] of the StringFixed<32> hash list (one stack argument, ret 4): frees the pointer argument with 00401c00(ptr, 4). |
| `00d08010` | `StringHashListMap_Clear` | ✅ | dtor | Installs the HashListMap vtable and removes the first stored node repeatedly until the list is empty. |
| `00d08040` | `StringHashListMap::ScalarDeletingDtor` | ✅ | virtual | Removes all nodes, frees the object when deleting flag is set, and otherwise returns the original pointer. |
