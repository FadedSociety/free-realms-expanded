# Class: PetTrickDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca6f0` | `PetTrickDataSource::GetColumnName` | ✅ | virtual | Maps indices zero through six to ID, icon, name, skill, busy, known, and tooltip labels; other indices return <error>. |
| `00ccc300` | `— ⚠reject` | ⚪ | method | Looks up the current pet by key, collects its trick records, applies the callback-based filter, updates bound rows, and marks the data source dirty. |
| `00ccc420` | `PetTrickDataSource::GetCurrentPetField` | ✅ | method | Refreshes when stale, resolves the current pet by its key, returns zero when absent, otherwise returns the field at pet offset 0x44. |
| `00ccca10` | `PetTrickDataSource::GetField` | ✅ | method | Resolves a trick row and current pet, looks up the matching trick record, then returns selected fields or localized string values for that trick. |
| `00ccd450` | `PetTrickDataSource::Dtor` | ✅ | dtor | Restores the PetTrickDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd8b0` | `PetTrickDataSource::Ctor` | ✅ | ctor | Initializes the embedded GUI table, installs three PetTrickDataSource vtables, stores two arguments, and clears the state byte. |
| `00ccd940` | `PetTrickDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the pet trick data source and frees the allocation when the deleting flag is set. |
