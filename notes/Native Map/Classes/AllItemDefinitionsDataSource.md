# Class: AllItemDefinitionsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c974c0` | `AllItemDefinitionsDataSource::Ctor` | ✅ | ctor | Constructs the base item data source with BaseClient.AllItemDefinitions, installs derived vtables, and initializes the remaining state. |
| `00c97540` | `AllItemDefinitionsDataSource::Dtor` | ✅ | dtor | Restores derived vtable pointers, then invokes the shared base item data-source destructor. |
| `00c975a0` | `AllItemDefinitionsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the all-items data-source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the pointer. |
