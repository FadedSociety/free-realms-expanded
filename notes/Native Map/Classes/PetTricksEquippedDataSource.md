# Class: PetTricksEquippedDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b5b340` | `PetTricksEquippedDataSource::PetTricksEquippedDataSource` | ✅ | ctor | Calls the BasePetTrickDataSource constructor, then installs PetTricksEquippedDataSource vtables at its three data-source subobjects. |
| `00b5b410` | `PetTricksEquippedDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the GuiUserDataTable base and frees the object only when param_2 bit 0 is set. |
| `00ccd1a0` | `PetTricksEquippedDataSource::GetField` | ✅ | method | Resolves the current pet, retrieves one of five equipped trick slots, and delegates field extraction when the slot resolves to a trick. |
