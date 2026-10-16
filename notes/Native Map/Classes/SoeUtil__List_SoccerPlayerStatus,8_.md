# Class: SoeUtil::List<SoccerPlayerStatus,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b7f100` | `SoccerPlayerStatusListInline8Ctor` | ✅ | ctor | Installs the generic list vtable, zeros list fields, switches to the inline-8 list vtable, initializes its node pool through 00b75c90, and returns this. |
| `00b7fb30` | `AdjustThisPlus16JumpToPoolPop` | ✅ | virtual | Decoded bytes add 0x10 to ECX then jump to 00b7fa50; this is an adjustor thunk, not the target function's body. |
| `00b7fb40` | `AdjustThisPlus16JumpToPoolPush` | ✅ | virtual | Decoded bytes add 0x10 to ECX then jump to 00b7fac0; this is an adjustor thunk, not the target function's body. |
| `00b7fc20` | `SoccerPlayerStatusListScalarDeletingDtor` | ✅ | virtual | Calls the list destructor body, tests flag bit 0, frees this when set, and otherwise returns this. |
