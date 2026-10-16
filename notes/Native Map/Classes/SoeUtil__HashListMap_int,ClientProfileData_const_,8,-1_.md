# Class: SoeUtil::HashListMap<int,ClientProfileData_const*,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2e330` | `HashListAllocator` | ✅ | method | Initializes allocator state, tries the current thread manager and global manager, then requests 0x14 bytes aligned to four. |
| `00c2e370` | `HashListFreeBuffer` | ✅ | method | Passes the supplied buffer to FUN_00401c00 at four-byte alignment. |
| `00c2f820` | `ClientProfileHashListClear` | ✅ | dtor | Restores the map vtable and repeatedly unlinks the current head through FUN_00c2e680 until empty. |
| `00c2f850` | `ClientProfileHashListScalarDeletingDtor` | ✅ | dtor | Restores the map vtable, unlinks all entries, and calls _free(this) when flag bit 0 is set. |
| `00c2fa00` | `ClientProfileHashListClear` | ✅ | dtor | Restores the map vtable and removes entries through FUN_00c2e680 until the head is null. |
