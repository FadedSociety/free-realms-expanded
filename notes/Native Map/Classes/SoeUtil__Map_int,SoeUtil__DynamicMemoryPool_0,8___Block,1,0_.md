# Class: SoeUtil::Map<int,SoeUtil::DynamicMemoryPool<0,8>::Block,1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01081410` | `MapCtorSentinelLayout` | 🟢 | ctor | Constructor body: stores the Map<int,DynamicMemoryPool<0,8>::Block,-1,0> vftable then the Map<int,DynamicMemoryPool<0,8>::Block,1,0> vftable at this+0x0, zeroes +0x4, +0x8, +0xc, +0x10 and the byte at +0x1c, sets the po… |
| `010814a0` | `MapSlot2ThunkAdj0xc` | 🟢 | virtual | Virtual slot 2 this-adjusting thunk (bytes: add ecx,0xc; jmp 0101a110). Adjusts the receiver by +0xc and jumps to 0101a110, which is outside this shard. |
| `010814b0` | `MapSlot3ThunkAdj0xc ?` | 🟡 | virtual | Sub-vtable slot 0 (vftable[3][0]) this-adjusting thunk (bytes: add ecx,0xc; jmp 0101a180). Adjusts the receiver by +0xc and jumps to 0101a180, which is outside this shard. |
| `01081540` | `MapDerivedDtorBody1x0` | 🟢 | dtor | Destructor body of the 1,0 variant: stores its vftable, destroys the layout-B tree at this+0x4 through 01080c40 and 010806a0, calls the container virtual slot 3, zeroes +0x4 and +0x8, then destroys a second tree at this… |
| `01081650` | `MapScalarDeletingDtor1x0` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 01081540 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
