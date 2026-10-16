# Class: SoeUtil::Map<unsigned_int,FlashMovie*,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c7da0` | `SoeUtil_MapFlashMovie_Dtor` | ✅ | dtor | Stores the Map<unsigned int,FlashMovie*,-1,0> vtable at this+0. If the root node at this+4 is nonzero it calls FUN_008ac620 on the words at node+0xc and node+0x10 (post-order tree destroy) and then vslot 3 (+0xc) of thi… |
| `008c9430` | `SoeUtil_MapFlashMovie_ScalarDeletingDtor` | ✅ | dtor | Stores the Map<unsigned int,FlashMovie*,-1,0> vtable at this+0, runs FUN_008ac620 on the two child words and the node teardown when set, and frees this with _free when bit 0 of the flag byte is set. Returns this. |
