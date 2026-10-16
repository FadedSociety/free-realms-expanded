# Class: SoeUtil::HashList<RecipeData,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8cad0` | `HashListRemoveFirstAdjustor` | ✅ | virtual | Adds 0x114 to ECX and jumps to 0x00c8c9f0; this is an adjustor thunk, not the removal body. |
| `00c8cae0` | `HashListAddFirstAdjustor` | ✅ | virtual | Adds 0x114 to ECX and jumps to 0x00c8ca60; this is an adjustor thunk, not the insertion body. |
| `00c8ce50` | `RecipeHashListCtor` | ✅ | ctor | Initializes its embedded fixed hash map and installs the HashList vtable. |
| `00c8ceb0` | `RecipeHashListScalarDeletingDtor` | ✅ | dtor | Destroys the embedded map and frees this object when deleting flag bit 0 is set. |
