# Class: PetTricksDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b5b210` | `PetTricksDataSource::PetTricksDataSource` | ✅ | ctor | Calls the BasePetTrickDataSource constructor, then replaces its three vtable pointers with PetTricksDataSource vtables. |
| `00b5b2e0` | `PetTricksDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the GuiUserDataTable base and frees the object only when param_2 bit 0 is set. |
| `00ccb530` | `PetTricksDataSource::GetActivePetField44` | ✅ | virtual | Finds active pet record by id and returns 32-bit field at record +0x44, or zero if no match. |
| `00ccd120` | `PetTricksDataSource::GetField` | ✅ | method | Resolves the current pet, walks its trick list to the requested index, and delegates row-field extraction to the shared trick-row helper. |
