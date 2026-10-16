# Class: SoeUtil::HashListMap<int,NotificationImage,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00958b40` | `NotificationImageMapCtor ?` | 🟡 | ctor | Installs HashListMap<int,NotificationImage,32,-1> vtable, clears counters and links, zeroes 0x80 bytes of bucket storage, and returns this. |
| `0095b410` | `HashListMap::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b450` | `HashListMapFreeNode` | ✅ | virtual | Passes the supplied node pointer to FUN_00401c00 with alignment 8. |
| `009642a0` | `NotificationImageHashListMapDestructor` | ✅ | dtor | Installs the NotificationImage hash-map vtable and repeatedly removes the node at param_1[2] through FUN_0095e740 until the map is empty. |
| `00964420` | `NotificationImageHashListMapScalarDeletingDestructor` | ✅ | dtor | Installs the NotificationImage map vtable, repeatedly removes remaining nodes through FUN_0095e740, and frees param_1 when flag bit 0 is set. |
