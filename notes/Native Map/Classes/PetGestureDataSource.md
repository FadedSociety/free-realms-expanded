# Class: PetGestureDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccadf0` | `PetGestureDataSource::GetColumnName` | ✅ | virtual | Maps indices zero through three to Trick, Order, X, and Y; other indices return <error>. |
| `00ccb580` | `— ⚠reject` | ⚪ | virtual | Refreshes backing table, finds selected gesture record by key, calls row callback for each entry, and marks data source dirty. |
| `00ccba50` | `PetGestureDataSource::GetRowCount` | ✅ | virtual | Refreshes gesture data when needed, finds selected gesture key in hash table, and returns its point count or zero if absent. |
| `00ccd220` | `— ⚠reject` | ⚪ | method | Resolves the row and gesture bucket keyed by this+0x230, verifies a matching gesture entry, then returns the requested gesture fields. |
| `00ccd5b0` | `PetGestureDataSource::Dtor` | ✅ | dtor | Restores PetGestureDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccdac0` | `PetGestureDataSource::Ctor` | ✅ | ctor | Initializes the GUI table, installs three PetGestureDataSource vtables, stores constructor arguments, clears its state byte, and zeros another field. |
| `00ccdb50` | `PetGestureDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the pet gesture data source and frees the allocation when the deleting flag is set. |
