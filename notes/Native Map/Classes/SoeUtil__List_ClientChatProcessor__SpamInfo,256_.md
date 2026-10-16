# Class: SoeUtil::List<ClientChatProcessor::SpamInfo,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b13500` | `SpamInfoList256Ctor` | ✅ | ctor | Installs the unbounded list vtable, zeros list metadata, switches to the inline-capacity-256 vtable, and initializes the in-object free nodes. |
| `00b13f30` | `SpamInfoList256PopAdjustorThunk` | ✅ | virtual | Adds 0x10 to ECX and jumps to 00b13e50; this vtable entry is an adjustor thunk and does not execute the target body inline. |
| `00b13f40` | `SpamInfoList256PushAdjustorThunk` | ✅ | virtual | Adds 0x10 to ECX and jumps to 00b13ec0; this vtable entry is an adjustor thunk and does not execute the target body inline. |
| `00b14020` | `SpamInfoList256ScalarDeletingDtor` | ✅ | dtor | Calls the shared SpamInfo list destructor, tests flag bit 0, conditionally frees param_1, and returns the original pointer when no free occurs. |
