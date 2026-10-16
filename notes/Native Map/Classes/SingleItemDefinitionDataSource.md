# Class: SingleItemDefinitionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c975c0` | `SingleItemDefinitionDataSource::Ctor` | ✅ | ctor | Constructs the base item source using BaseClient.ItemDefinitionSelect, installs the single-item data source vtables, and returns the object. |
| `00c97640` | `SingleItemDefinitionDataSource::Dtor` | ✅ | dtor | Restores the single-item derived vtable pointers and invokes the shared base data-source destructor. |
| `00c976a0` | `SingleItemDefinitionDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the single-item source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the pointer. |
