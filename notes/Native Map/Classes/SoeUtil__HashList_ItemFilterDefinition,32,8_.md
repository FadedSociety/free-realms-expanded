# Class: SoeUtil::HashList<ItemFilterDefinition,32,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107ed70` | `HashListSlot1AllocNodeThunk` | 🟢 | virtual | Virtual slot 1 this-adjusting thunk (bytes: add ecx,0x94; jmp 0107ec90). Adjusts the receiver by +0x94 and jumps to the pool node allocator 0107ec90. |
| `0107ed80` | `HashListSlot2FreeNodeThunk` | 🟢 | virtual | Virtual slot 2 this-adjusting thunk (bytes: add ecx,0x94; jmp 0107ed00). Adjusts the receiver by +0x94 and jumps to the pool node free routine 0107ed00. |
| `0107f2a0` | `HashList32x8Ctor` | 🟢 | ctor | Constructor body: calls 0107eea0 (HashListMap 32,8 constructor body) on this, then stores the SoeUtil::HashList<ItemFilterDefinition,32,8> vftable at *this and returns this. |
| `0107f360` | `HashList32x8ScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 0107e2e0 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
