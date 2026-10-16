# Class: PetEquippedWearablesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccaab0` | `PetEquippedWearablesDataSourceCtor` | ✅ | ctor | Initializes shared data source, installs its vtables at four object offsets, stores passed value, and clears its second state field. |
| `00ccab50` | `PetEquippedWearablesDataSourceDtor` | ✅ | dtor | Restores class vtables at main and embedded offsets and releases base data-source state through FUN_00cb1b10. |
| `00ccb1f0` | `PetEquippedWearablesDataSource::DtorAdjustorThunk` | ✅ | dtor | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb470; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb470` | `PetEquippedWearablesDataSourceScalarDeletingDtor` | ✅ | dtor | Calls equipped-wearables data-source destructor, conditionally frees object, and returns it; bytes confirm call wrapper. |
| `00ccc560` | `PetEquippedWearablesDataSource::FindByWearable ⚠downgrade` | 🟢 | method | Resolves the current pet, iterates its equipped utility item ids, resolves each item, matches its category to the requested value, and returns the selected result. |
