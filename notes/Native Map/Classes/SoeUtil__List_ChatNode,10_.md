# Class: SoeUtil::List<ChatNode,10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b10b00` | `ChatNodeList10Ctor` | ✅ | ctor | Installs the unbounded-list vtable, zeros the list metadata, switches to the inline-capacity-10 vtable, and initializes its in-object free nodes. |
| `00b11530` | `ChatNodeList10PopAdjustorThunk` | ✅ | virtual | Adds 0x10 to ECX and jumps to 00b11450; this vtable entry is an adjustor thunk and does not execute the target body inline. |
| `00b11540` | `ChatNodeList10PushAdjustorThunk` | ✅ | virtual | Adds 0x10 to ECX and jumps to 00b114c0; this vtable entry is an adjustor thunk and does not execute the target body inline. |
| `00b12840` | `ChatNodeList10ScalarDeletingDtor` | ✅ | dtor | Calls the ChatNode list destructor, tests flag bit 0, conditionally frees param_1, and returns the original pointer when it is not freed. |
