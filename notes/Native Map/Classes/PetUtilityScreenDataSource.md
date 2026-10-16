# Class: PetUtilityScreenDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca970` | `PetUtilityScreenDataSourceCtor` | ✅ | ctor | Initializes shared base, installs class vtables at main and embedded offsets, stores constructor values, and clears an additional field. |
| `00ccaa10` | `PetUtilityScreenDataSourceDtor` | ✅ | dtor | Restores class vtables at embedded offsets and invokes FUN_00cb1b10 to release base data-source state. |
| `00ccaa70` | `PetUtilityScreenDataSource::MatchesUtilityId ⚠downgrade` | 🟢 | virtual | Requires lookup category four, then compares another lookup value with this+0x2e0 and returns one only on equality. |
| `00ccb1e0` | `PetUtilityScreenDataSource::DtorAdjustorThunk` | ✅ | dtor | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb450; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb450` | `PetUtilityScreenDataSourceScalarDeletingDtor` | ✅ | dtor | Calls utility-screen data-source destructor, conditionally frees object, and returns it; bytes confirm standard scalar deleting wrapper. |
