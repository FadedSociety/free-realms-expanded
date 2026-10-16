# Class: PetActiveDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccb210` | `PetActiveDataSource::RefreshActivePetRows` | ✅ | virtual | Refreshes source, finds active pet by id, walks display rows, and invokes row update callback for rows with positive count and matching pet id. |
| `00ccd6d0` | `PetActiveDataSource::Ctor` | ✅ | ctor | Constructs the PetListDataSource base, replaces its vtables with PetActiveDataSource vtables, and returns the initialized object. |
| `00ccd7a0` | `PetActiveDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the PetListDataSource base state and frees the allocation when the deleting flag is set. |
