# Class: SoeUtil::HashListMap<int,SoeUtil::StrongType<__int64,SoeUtilSignatureGuildGuid,0>,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcb50` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x18 bytes aligned to eight through the thread manager, global manager, or aligned CRT fallback. |
| `008bcb90` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `008e0960` | `HashListMapGuildGuid_Dtor` | ✅ | dtor | Installs the map vtable and repeatedly destroys its linked node at field +8 until the map is empty. |
| `008e20a0` | `HashListMapGuildGuid_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, destroys linked nodes, and optionally frees the map object based on flag bit 0. |
| `008eaf50` | `HashListMapDtor ? ⚠rename` | 🟡 | dtor | Installs the exact guild-GUID map vtable, then while the head at +8 is nonnull calls 008bff80 with the map in ECX and the current head pushed as its argument. |
| `008f20c0` | `HashListMapGuildGuid_Ctor` | ✅ | ctor | Installs the GuildGuid map vtable, zeros its state and embedded storage fields, then leaves the corresponding HashList base vtable installed. |
