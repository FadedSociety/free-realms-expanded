# Class: SoeUtil::List<AppStoreBundleDefinition_const*,512>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a1b50` | `ListAppStoreBundleCtor512` | ✅ | ctor | Initializes list pointers/counts to zero, installs the List<AppStoreBundleDefinition_const*,512> vtable, then calls 0099BD60 with ECX pointing to embedded list storage at this+0x10. |
| `009a2570` | `AppStoreBundleListDestructor` | ✅ | dtor | Installs the 512-entry list vtable, drains its linked entries, clears stored list fields, recursively frees the tree children, frees the root, and restores the unbounded base-list vtable. |
| `009a2620` | `AppStoreBundleListPopAdjustor` | ✅ | virtual | Adds 0x10 to this and jumps to 009a24a0, the free-list pop implementation. |
| `009a2630` | `AppStoreBundleListPushAdjustor` | ✅ | virtual | Adds 0x10 to this and jumps to 009a2510, the free-list push implementation. |
| `009a2df0` | `AppStoreBundleListScalarDeletingDtor` | ✅ | dtor | Calls the AppStoreBundleDefinition list destructor, tests deletion flag bit 0, frees this when set, and otherwise returns this. |
