# Class: SoeUtil::HashList<NotificationImageDefinition,20,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01026180` | `HashListPoolGrow ? ⚠rename` | 🟡 | method | Adds a run of 0x28-byte free slots: count = max(requested, clamp(tree total, 2, 0x1000)) where the total is the value returned by 01025bf0 (it already starts at 0x10); allocates count*0x28+0x1c bytes through the pool or… |
| `010262a0` | `HashListPoolReleaseChunk ? ⚠rename` | 🟡 | method | Releases one fully free chunk: unlinks every free-list slot inside [base, base+capacity*0x28) from the list at +8, subtracts the chunk capacity from the total at +0xc, erases the chunk header with 01025fd0, then frees t… |
| `01026300` | `HashListPoolPop ? ⚠rename` | 🟡 | method | Pops the head of the free-slot list at +8: grows through 01026180 when the head is null, sets the head to the popped slot's next dword, decrements the free count at +0xc, and when the flag byte at +0x10 is set and 01025… |
| `01026370` | `HashListPoolPush` | 🟢 | method | Pushes slot param_2 onto the free-list head at +8 and increments the free count at +0xc; when the flag byte at +0x10 is set it finds the owning chunk (01025c50) and increments its free count at +0x18, calling 010262a0 t… |
| `010263e0` | `HashListAllocSlotThunk` | ✅ | virtual | Adjustor thunk: bytes 83 c1 64 (add ecx,0x64) then e9 (jmp 01026300), so the call reaches the pop routine on the HashList header at this+0x64. |
| `010263f0` | `HashListFreeSlotThunk` | ✅ | virtual | Adjustor thunk: bytes 83 c1 64 (add ecx,0x64) then e9 (jmp 01026370), so the call reaches the push routine on the HashList header at this+0x64. |
| `01026810` | `HashListCtor` | ✅ | ctor | Constructor: calls the HashListMap constructor 01026420 on this, then stores the HashList vtable at offset 0 and returns this. |
| `01026870` | `HashListScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 01025980 on this, then calls _free on this when bit 0 of the flag argument is set; returns this. |
