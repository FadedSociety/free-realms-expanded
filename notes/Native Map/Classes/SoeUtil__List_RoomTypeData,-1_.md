# Class: SoeUtil::List<RoomTypeData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f7220` | `ListSlot2_AllocNode ?` | 🟡 | virtual | Virtual slot [2] of SoeUtil::List<RoomTypeData,-1>::vftable (no arguments): calls 006fb9c0, then 006fb860 and the slot [1] call on the thread manager, else on DAT_01bdf280, else 006fb950(0x10,4). Result left in eax. |
| `009f7260` | `ListSlot3_FreeAlign4 ?` | 🟡 | virtual | Virtual slot [3] of SoeUtil::List<RoomTypeData,-1>::vftable (one stack argument, ret 4; bytes 8b4424046a0450e8...): pushes 4 and the argument, calls 00401c00 (FreeMemory with alignment 4), pops 8 bytes and returns. |
| `009f8020` | `ListRoomTypeData_DtorBody` | ✅ | dtor | Destructor body as an 11-byte thunk (c7 01 vtable; e9 rel32, bytes checked): stores SoeUtil::List<RoomTypeData,-1>::vftable into [this], then jumps to 009f7b70, which drains and frees the list nodes. |
| `009f80d0` | `ListRoomTypeData_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot [0] of SoeUtil::List<RoomTypeData,-1>::vftable (thiscall, ret 4): stores the list vtable into [this], calls 009f7b70 with ECX = this to drain and free the nodes, then calls _free(this) t… |
