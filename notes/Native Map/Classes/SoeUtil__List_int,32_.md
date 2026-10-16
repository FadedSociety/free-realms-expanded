# Class: SoeUtil::List<int,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008f21b0` | `SoeUtilListInt32_Ctor` | ✅ | ctor | Installs the dynamic List<int> vtable, clears three header fields, switches to the inline-32 list vtable, and initializes its embedded sentinel pool. |
| `008f2bd0` | `SoeUtilListInt32_Dtor` | ✅ | dtor | Installs the inline list vtable, drains list nodes, recursively frees the tree-backed inline pool, clears pool fields, installs the dynamic base vtable, and drains that list. |
| `008f2c80` | `ListInt32_Slot2Adjustor` | ✅ | virtual | Adds 0x10 to ECX and jumps to 008f2b00; the eight-byte body is an adjustor thunk. |
| `008f2c90` | `ListInt32_Slot3Adjustor` | ✅ | virtual | Adds 0x10 to ECX and jumps to 008f2b70; the eight-byte body is an adjustor thunk. |
