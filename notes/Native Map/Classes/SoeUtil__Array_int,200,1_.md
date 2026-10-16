# Class: SoeUtil::Array<int,200,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8d0c0` | `IntegerArray200::FreeExternal` | ✅ | virtual | Calls the aligned free helper only when the supplied pointer differs from this object's inline-buffer address. |
| `00b8e9d0` | `IntegerArray200::Ctor` | ✅ | ctor | Initializes array pointer/size/capacity fields and installs the 200-inline integer-array specialization vtable. |
| `00b8ea30` | `IntegerArray200::Allocate` | ✅ | virtual | Returns aligned inline storage and reports capacity 200, or delegates requests above 200 to FUN_0073a540. |
| `00b8eb10` | `IntegerArray200::DtorBody` | ✅ | dtor | Restores the 200-inline specialization vtable, releases external storage if used, then resets base array state. |
