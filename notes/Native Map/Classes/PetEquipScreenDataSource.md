# Class: PetEquipScreenDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca880` | `PetEquipScreenDataSourceCtor` | ✅ | ctor | Calls base setup with constructor arguments, installs class vtables at main and embedded offsets, and stores two supplied values. |
| `00cca910` | `PetEquipScreenDataSourceDtor` | ✅ | dtor | Restores class vtables at four embedded offsets and calls FUN_00cb1b10 to release base data-source state. |
| `00ccb200` | `PetEquipScreenDataSource::DtorAdjustorThunk` | ✅ | dtor | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb380; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb380` | `PetEquipScreenDataSourceScalarDeletingDtor` | ✅ | dtor | Calls PetEquipScreenDataSource destructor and frees object when deleting flag is set; bytes show a normal call wrapper. |
| `00ccb3a0` | `PetEquipScreenDataSource::CanEquipItem` | ✅ | virtual | Requires category three, checks optional id filter, resolves item definition, verifies pet/equipment context and item metadata, then returns final eligibility lookup result. |
| `00ccc480` | `PetEquipScreenDataSource::HandleEquipSelection` | ✅ | method | For selection value 12, searches the current pet's utility ids for the selected item id and chooses one of two UI values; otherwise delegates to the base handler. |
