# Class: BasePetTrickDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b5b120` | `— ⚠reject` | ⚪ | ctor | Calls the GuiUserDataTable constructor, installs BasePetTrickDataSource vtables at three subobject offsets and stores param_2 at field 0x254. |
| `00b5b1b0` | `BasePetTrickDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the GuiUserDataTable destructor and frees the object only when param_2 bit 0 is set. |
| `00ccacc0` | `BasePetTrickDataSource::GetColumnName` | ✅ | virtual | Maps indices zero through twenty to item guid, icon, name, description, quantity, equip state, model, abilities, power rating and effect labels; other indices return <error>. |
