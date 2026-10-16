# Class: PetTrickFavoriteDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cca7b0` | `PetTrickFavoriteDataSource::GetColumnName` | ✅ | virtual | Returns ID, Slot, Icon, Name, or SkillLevel for indices zero through four; other indices return <error>. |
| `00ccb300` | `— ⚠reject` | ⚪ | function | Refreshes backing source, locates active pet record, calls row callback for each of four favorite entries, and marks source dirty. |
| `00ccb9b0` | `— ⚠reject` | ⚪ | virtual | Refreshes favorite data when dirty flag is clear, then returns fixed count four. |
| `00cccc10` | `— ⚠reject` | ⚪ | method | Resolves a table row and current-pet favorite-trick record, then returns the requested field or localized text for that favorite. |
| `00ccd4c0` | `PetTrickFavoriteDataSource::Dtor` | ✅ | dtor | Restores PetTrickFavoriteDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd960` | `PetTrickFavoriteDataSource::Ctor` | ✅ | ctor | Initializes the GUI table, installs three favorite-trick data source vtables, stores constructor arguments, and clears its state byte. |
| `00ccd9f0` | `PetTrickFavoriteDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the favorite-trick data source and frees its allocation when the deleting flag is set. |
