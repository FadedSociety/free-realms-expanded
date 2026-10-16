# Class: SoeUtil::HashMap<int,int,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00988340` | `HashMap8Int_ClearThunk` | ✅ | dtor | Writes the map vtable into [this], then tail-jumps to 00987b50 to release bucket entries. |
| `00988350` | `HashMap8Int_AllocateNode` | ✅ | virtual | Uses the configured/global allocator sequence and requests 0xc bytes aligned to 4 as fallback. |
| `00988390` | `HashMap8Int_FreeNode` | ✅ | virtual | Forwards its pointer and alignment 4 to 00401c00, then returns. |
| `009884f0` | `HashMap8Int_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, clears entries through 00987b50, optionally frees the object, and otherwise returns it. |
