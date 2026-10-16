# Class: TCGInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad1170` | `TCGInterface::Dtor` | ✅ | dtor | Restores the TCGInterface vtable at the start of the object and returns. |
| `00ad1180` | `TCGInterface::ScalarDeletingDtor` | ✅ | dtor | Restores the TCGInterface vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00ad1790` | `TCGInterface::Ctor` | ✅ | ctor | Installs the TCGInterface vtable and returns. |
