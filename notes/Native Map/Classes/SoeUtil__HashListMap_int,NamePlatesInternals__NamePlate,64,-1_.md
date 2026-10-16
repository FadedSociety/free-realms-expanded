# Class: SoeUtil::HashListMap<int,NamePlatesInternals::NamePlate,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d0d70` | `NamePlatesInternals::NamePlateHashMapCtor` | ✅ | ctor | Installs the HashListMap vtable, clears its counters and storage, and initializes its bucket array to zero. |
| `009d1a00` | `NamePlatesInternals::NamePlateHashMap::Dtor` | ✅ | dtor | Installs the hash-list-map vtable and repeatedly removes the head node at this+8 through FUN_009d16d0. |
| `009d1a50` | `NamePlatesInternals::NamePlateHashMap::ScalarDeletingDtor` | ✅ | dtor | Installs the hash-list-map vtable, removes each node through FUN_009d16d0, and frees the object when the deleting flag is set. |
| `009d1a90` | `NamePlatesInternals::NamePlateHashMap::DtorBody` | ✅ | dtor | Installs the hash-list-map vtable, removes each map entry through FUN_009d16d0, and restores the previous exception frame. |
| `009d1b40` | `NamePlatesInternals::NamePlateHashMapCtor` | ✅ | ctor | Installs map fields, clears 256 bytes of buckets, and finishes by installing the HashList base vtable. |
