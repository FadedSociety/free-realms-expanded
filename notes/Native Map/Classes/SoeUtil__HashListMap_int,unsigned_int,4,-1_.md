# Class: SoeUtil::HashListMap<int,unsigned_int,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008e0ed0` | `HashListMapIntUInt_Dtor` | ✅ | dtor | Installs the map vtable and destroys its linked nodes until the map head becomes null. |
| `008e22c0` | `HashListMapIntUInt_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, destroys linked nodes, and optionally frees the map object when flag bit 0 is set. |
| `008ebd30` | `IntUnsignedHashListMapDtor` | ✅ | dtor | Installs the exact int-to-unsigned map vtable, then while the head at +8 is nonnull calls 008c00f0 with the map in ECX and the current head pushed as its argument. |
| `008f4e10` | `HashListMapIntUInt_Ctor` | ✅ | ctor | Installs the int-to-unsigned map vtable, zeros its map state and inline-storage fields, then leaves the HashList base vtable installed. |
