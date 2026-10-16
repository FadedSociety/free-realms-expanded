# Class: SoeUtil::HashList<SoeUtil::RefPointer<LoadingScreenData>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cab3b0` | `HashList_LoadingScreenData::Ctor` | ✅ | ctor | Constructs the LoadingScreenData id table: zeroes the bucket and count words, clears 0x40 bytes of buckets with _memset, stores the HashListMap vtable and then the HashList vtable, so the final dynamic type is HashList. |
| `00cab430` | `HashList_LoadingScreenData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the HashList table (vtable slot 0): calls the destructor body 00caac50, then frees this through _free when bit 0 of the flag argument is set. |
