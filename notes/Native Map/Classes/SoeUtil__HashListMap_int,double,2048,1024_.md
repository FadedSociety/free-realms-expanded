# Class: SoeUtil::HashListMap<int,double,2048,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008f1610` | `HashListMapIntDouble2048_Dtor` | ✅ | dtor | Installs the map vtable, repeatedly unlinks head nodes, frees the auxiliary tree and chunk block, clears their fields, then installs the base HashList vtable and drains remaining nodes. |
| `008f20a0` | `HashListMapDouble2048_Slot1Adjustor` | ✅ | virtual | Adds 0x2014 to ECX and jumps to 008f1fc0; this is an adjustor thunk, not the target function body. |
| `008f20b0` | `HashListMapDouble2048_Slot2Adjustor` | ✅ | virtual | Adds 0x2014 to ECX and jumps to 008f2030; this is an adjustor thunk, not the target function body. |
