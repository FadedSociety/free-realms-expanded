# Class: SoeUtil::HashList<ItemSortDefinition,32,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01027c50` | `HashListPoolPop1C` | 🟢 | method | Pops the head of the 0x1c-slot free list at +8: grows through 01027ad0 when the head is null, sets the head to the popped slot's next dword, decrements the free count at +0xc, and when the flag byte at +0x10 is set and … |
| `01027cc0` | `HashListPoolPush1C` | 🟢 | method | Pushes slot param_2 onto the 0x1c-slot free list head at +8 and increments the free count at +0xc; when the flag byte at +0x10 is set it finds the owning chunk (010275a0) and increments its free count at +0x18, calling … |
| `01027d30` | `HashListAllocSlotThunk32x8` | ✅ | virtual | Adjustor thunk: bytes 81 c1 94 00 00 00 (add ecx,0x94) then e9 (jmp 01027c50), so the call reaches the 0x1c-slot pop on the header at this+0x94. |
| `01027d40` | `HashListFreeSlotThunk32x8` | ✅ | virtual | Adjustor thunk: bytes 81 c1 94 00 00 00 (add ecx,0x94) then e9 (jmp 01027cc0), so the call reaches the 0x1c-slot push on the header at this+0x94. |
| `010282a0` | `HashListCtor32x8` | ✅ | ctor | Constructor: calls the 32x8 HashListMap constructor 01027d70 on this, then stores the HashList<ItemSortDefinition,32,8> vtable at offset 0 and returns this. |
| `01028300` | `HashListScalarDeletingDtor32x8` | ✅ | dtor | Scalar deleting destructor: calls the 32x8 body 010272a0 on this, then calls _free on this when bit 0 of the flag argument is set; returns this. |
