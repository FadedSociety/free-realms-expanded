# Class: PetEquippedUtilityDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccabc0` | `PetEquippedUtilityDataSourceCtor` | ✅ | ctor | Initializes shared base, installs class vtables at main and embedded offsets, stores one value, and clears the other. |
| `00ccac60` | `PetEquippedUtilityDataSourceDtor` | ✅ | dtor | Restores class vtables at main and embedded offsets and invokes FUN_00cb1b10 to release base data-source state. |
| `00ccb1d0` | `PetEquippedUtilityDataSource::DtorAdjustorThunk` | ✅ | dtor | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb490; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb490` | `PetEquippedUtilityDataSourceScalarDeletingDtor` | ✅ | dtor | Calls equipped-utility data-source destructor, conditionally frees object, and returns it; bytes confirm call wrapper. |
| `00ccb4b0` | `PetEquippedUtilityDataSource::MatchesSlot` | ✅ | virtual | Requires category four, finds active pet, scans eight item slots from record +0x98, and returns one if a slot id equals this+0x10. |
| `00ccb9d0` | `PetEquippedUtilityDataSource::RefreshRows ? ⚠rename` | 🟡 | virtual | Refreshes backing source, scans inventory rows, selects entries whose type is one or two, invokes row callback for each, and marks source dirty. |
| `00ccc100` | `— ⚠reject` | ⚪ | method | Rejects indices above six, resolves the current pet record by key, reads the indexed utility id, resolves its item, and delegates the requested field lookup. |
