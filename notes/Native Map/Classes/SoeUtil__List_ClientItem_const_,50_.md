# Class: SoeUtil::List<ClientItem_const*,50>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e28e0` | `ClientItemListCtor` | ✅ | ctor | Clears list pointers/counts, installs fixed-capacity fifty-element vtable and initializes backing storage. |
| `009e3300` | `ClientItemListDtor` | ✅ | dtor | Destroys end nodes/backing block, clears list state and restores base-list vtable. |
| `009e33b0` | `ThisAdjustor_009e3230` | ✅ | virtual | Adds 0x10 to ECX and jumps to 009e3230; this-adjustor thunk, not target body. |
| `009e33c0` | `ThisAdjustor_009e32a0` | ✅ | virtual | Adds 0x10 to ECX and jumps to 009e32a0; this-adjustor thunk, not target body. |
| `009e3430` | `ClientItemListScalarDeletingDtor` | ✅ | dtor | Calls bounded-list destructor, then frees this when deleting flag set; otherwise returns this. |
