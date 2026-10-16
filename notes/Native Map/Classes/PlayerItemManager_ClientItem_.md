# Class: PlayerItemManager<ClientItem>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098fb10` | `PlayerItemManagerCtor` | ✅ | ctor | Installs the manager vtable, zeroes two 100-bucket tables and associated fields, and sets the final initialization flag to 1. |
| `00991860` | `PlayerItemManagerDtor` | ✅ | dtor | Reinstalls the generic player item manager vtable, prepares exception cleanup state, calls base item-manager teardown, and restores the exception frame. |
| `009918b0` | `PlayerItemManagerScalarDeletingDtor` | ✅ | dtor | Calls the manager destructor and frees the object when the deleting flag is set; otherwise returns the original pointer. |
