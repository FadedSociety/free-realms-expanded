# Class: SoeUtil::HashListMap<int,TextureAssetTrackingNode,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a8f550` | `TextureTrackingHashMapCleanup` | ✅ | dtor | Installs the specialized map vtable and removes every entry at head until the list is empty. |
| `00a8f6a0` | `TextureTrackingHashMapDeletingDestructor` | ✅ | dtor | Runs the map cleanup body, frees the object only when flag bit 0 is set, and otherwise returns the original pointer. |
| `00a8fa10` | `TextureTrackingHashMapCleanup` | ✅ | dtor | Installs the texture-map vtable and repeatedly removes entries from the list head until empty. |
