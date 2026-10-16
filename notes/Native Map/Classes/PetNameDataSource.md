# Class: PetNameDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca820` | `PetNameDataSource::GetColumnName` | ✅ | virtual | Maps indices zero through three to String, Id, Part, and Selected; other indices return <error>. |
| `00ccc0e0` | `PetNameDataSource::GetField` | ✅ | method | When the readiness byte at this+0x22c is clear, calls the refresh helper, then returns the value stored at this+0x230. |
| `00cccdb0` | `— ⚠reject` | ⚪ | method | Resolves the name row, returns a stored name or localized value, and formats a key from player name, active pet id, and pet-name id. |
| `00ccd530` | `PetNameDataSource::Dtor` | ✅ | dtor | Restores PetNameDataSource vtables at primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccda10` | `PetNameDataSource::Ctor` | ✅ | ctor | Initializes the embedded GUI table, installs three PetNameDataSource vtables, stores constructor arguments, and clears the state byte. |
| `00ccdaa0` | `PetNameDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the pet-name data source and frees the allocation when the deleting flag is set. |
