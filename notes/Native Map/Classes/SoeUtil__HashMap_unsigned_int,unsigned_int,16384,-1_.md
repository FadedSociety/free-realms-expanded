# Class: SoeUtil::HashMap<unsigned_int,unsigned_int,16384,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8b910` | `UnsignedHashMap::Ctor` | ✅ | ctor | Installs the typed HashMap vtable, zeros count and state fields, and clears 0x10000 bytes starting at the bucket array. |
| `00b8d0f0` | `UnsignedHashMap::DtorBody` | ✅ | dtor | Restores the hash-map vtable and clears all nodes through the shared map cleanup routine. |
| `00b8d220` | `UnsignedHashMap::ScalarDeletingDtor` | ✅ | dtor | Installs the hash-map vtable, clears entries, and frees the object when delete flag bit zero is set. |
