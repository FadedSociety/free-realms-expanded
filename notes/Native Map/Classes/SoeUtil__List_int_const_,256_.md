# Class: SoeUtil::List<int_const*,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cdf420` | `List256Constructor` | ✅ | ctor | Installs the base list vtable, zeros head/tail/count, installs the 256-capacity specialization vtable, and initializes its backing storage. |
| `00cdf4a0` | `List256DestructorBody` | ✅ | dtor | Restores the capacity-256 vtable, clears nodes, frees both backing arrays with alignment 4, resets fields, restores the base vtable, and clears again. |
| `00cdf550` | `List256AdjustorPlus10Slot2` | ✅ | virtual | Adds 0x10 to ECX and tail-jumps to 007f7d20; this is an adjustor thunk, not the target method body. |
| `00cdf560` | `List256AdjustorPlus10Slot3` | ✅ | virtual | Adds 0x10 to ECX and tail-jumps to 007f7d90; this is an adjustor thunk, not the target method body. |
| `00cdf570` | `List256ScalarDeletingDestructor` | ✅ | dtor | Calls the capacity-256 list destructor body, frees this when flag bit 0 is set, and otherwise returns this. |
