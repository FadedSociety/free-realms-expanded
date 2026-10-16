# Class: SoeUtil::HashListMap<int,PetTrickInfo,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcbc0` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x34 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bcc00` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `008e0990` | `HashListMapPetTrickInfo_Dtor` | ✅ | dtor | Installs the map vtable and repeatedly destroys linked nodes until field +8 is null. |
| `008e20e0` | `HashListMapPetTrickInfo_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, destroys linked nodes, and optionally frees the map object based on flag bit 0. |
| `008eaee0` | `PetTrickInfoHashListMapDtor` | ✅ | dtor | Installs the exact PetTrickInfo map vtable, then while the head at +8 is nonnull calls 008bf410 with the map in ECX and the current head pushed as its argument. |
| `008f2130` | `HashListMapPetTrickInfo_Ctor` | ✅ | ctor | Installs the PetTrickInfo map vtable, clears the map header, zeroes 0x40 bytes of inline storage, then installs the HashList base vtable. |
