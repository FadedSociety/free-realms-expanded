# Class: PetListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca5a0` | `— ⚠reject` | ⚪ | virtual | If dirty byte +0x22c is clear, calls owner virtual method at offset +0x38, then calls FUN_010c8f30. |
| `00cca5c0` | `PetListDataSource::GetColumnName` | ✅ | virtual | Maps indices zero through thirteen to pet name, sex, icon, needs, mood, id, membership, appearance, and tutorial labels; other indices return <error>. |
| `00ccc200` | `PetListDataSource::Refresh` | ✅ | method | Counts pet records with a positive field, collects their ids, sorts the temporary list, updates bound UI rows, and sets the dirty flag when needed. |
| `00ccc680` | `— ⚠reject` | ⚪ | method | Resolves a row id and pet record, then switches over field indices to update the matching pet fields or localized string data. |
| `00ccd350` | `PetListDataSource::Dtor` | ✅ | dtor | Restores PetListDataSource vtables at the primary and two embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd620` | `PetListDataSource::Ctor` | ✅ | ctor | Initializes the embedded GUI table, installs three PetListDataSource vtables, stores two constructor arguments, and clears the initialization flag. |
| `00ccd6b0` | `PetListDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the PetListDataSource destructor and frees the object when the deleting flag is set. |
| `00ccd750` | `— ⚠reject` | ⚪ | dtor | Creates an exception-cleanup frame, invokes PetListDataSource destruction on a stack temporary, and restores the prior exception list. |
