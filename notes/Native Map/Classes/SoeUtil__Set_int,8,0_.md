# Class: SoeUtil::Set<int,8,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a0f00` | `SetIntCtorInline8` | ✅ | ctor | Initializes generic Set fields, installs the Set<int,8,0> vtable, then calls 0099BAB0 with ECX pointing to the embedded set storage at this+0x0C. |
| `009a1930` | `SetIntDtorInline8` | ✅ | dtor | Destroys inline and heap set structures, recursively frees indexed tree nodes, clears free-list fields, and restores the generic Set vtable. |
| `009a1a30` | `SetIntPopThunk` | ✅ | virtual | Adjusts ECX by 0xC and jumps directly to 009A1850; this is an adjustor thunk with no separate body. |
| `009a1a40` | `SetIntPushThunk` | 🟢 | virtual | Adjusts ECX by 0xC and jumps directly to 009A18C0; it is an adjustor thunk, not an independent implementation. |
| `009a2d90` | `IntSetInline8ScalarDeletingDtor` | ✅ | dtor | Calls the Set<int,8,0> destructor, tests deletion flag bit 0, frees this when that bit is set, and otherwise returns this. |
