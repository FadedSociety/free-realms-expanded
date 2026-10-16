# Class: PetActionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca690` | `PetActionDataSource::GetColumnName` | ✅ | virtual | Returns ID, Target, or Behavior for indices zero, one, or two; other indices return <error>. |
| `00ccb2c0` | `PetActionDataSource::RefreshAllRows ?` | 🟡 | function | Refreshes backing source, walks every row, invokes update callback with each row id, and marks the source dirty. |
| `00ccb980` | `— ⚠reject` | ⚪ | virtual | If source dirty flag is clear, refreshes all action rows; returns row count at source record +0xc. |
| `00ccc940` | `— ⚠reject` | ⚪ | method | Resolves the requested row and current pet, returns selected pet id or localized action text, and reports missing row, pet, or string values. |
| `00ccd3d0` | `PetActionDataSource::Dtor` | ✅ | dtor | Restores the PetActionDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd800` | `PetActionDataSource::Ctor` | ✅ | ctor | Initializes the embedded GUI data table, installs three PetActionDataSource vtables, stores two constructor arguments, and clears its state byte. |
| `00ccd890` | `PetActionDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the pet action data source destructor, then frees the allocation when the deleting flag is set. |
