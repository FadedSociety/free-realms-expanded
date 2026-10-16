# Class: BaseFilterItemDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb1b10` | `BaseFilterItemDataSource::Dtor` | ✅ | dtor | Restores the class vtables at four subobjects, destroys the embedded ItemGuid array, then releases the embedded GUI user-data holder. |
| `00cb1ba0` | `— ⚠reject` | ⚪ | virtual | Jumps directly to 00cb1c80, the scalar deleting destructor body for BaseFilterItemDataSource. |
| `00cb1bb0` | `BaseFilterItemDataSource::Ctor` | ✅ | ctor | Initializes the named GUI user-data holder and shared members, stores the source pointer, initializes filter state and vtables, and binds the Inventory table name. |
| `00cb1c80` | `BaseFilterItemDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the class destructor, frees the object when the deleting flag's low bit is set, and otherwise returns the original pointer. |
