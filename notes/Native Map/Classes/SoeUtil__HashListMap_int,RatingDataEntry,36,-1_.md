# Class: SoeUtil::HashListMap<int,RatingDataEntry,36,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c81930` | `RatingDataHashListCtor` | ✅ | ctor | Installs the HashListMap vtable, zeros its head/count state, and clears 0x90 bytes of bucket storage. |
| `00c85110` | `HashListMapDtor` | ✅ | dtor | Installs the HashListMap vtable, then repeatedly destroys the node at object offset 8 until the list is empty. |
| `00c85470` | `HashListMapScalarDeletingDtor` | ✅ | virtual | Destroys all linked nodes, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00c855b0` | `HashListMapDtor` | ✅ | dtor | Installs the map vtable and repeatedly removes and destroys the linked node at object offset 8. |
| `00c85900` | `HashListMapCtor` | ✅ | ctor | Initializes the hash-list map fields and bucket storage to zero, then installs the HashList vtable. |
