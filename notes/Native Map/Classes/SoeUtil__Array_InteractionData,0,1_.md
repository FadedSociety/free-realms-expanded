# Class: SoeUtil::Array<InteractionData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9a890` | `SoeUtil_ArrayInteractionData_AllocBuffer` | 🟢 | method | Same buffer policy as 00a9a710 for 0x20-byte elements: grow to requested*5/4 when the requested count exceeds the capacity at this+0xc, return the current buffer at this+4 when the capacity is within requested..requeste… |
| `00a9a930` | `SoeUtil_ArrayInteractionData_FreeBuffer` | 🟢 | method | Forwards the pointer argument to 00401c00 (FreeMemory) with alignment 4 and returns. |
| `00a9a950` | `SoeUtil_ArrayInteractionData_Swap` | 🟢 | method | Thiscall with a second array and a flag. When the flag is nonzero it tail-delegates to the argument array's vtable slot [3] (register arguments not recovered). Otherwise it exchanges the buffer pointer at this+4, the el… |
| `00a9af20` | `SoeUtil_ArrayInteractionData_Dtor` | ✅ | dtor | Stores the Array<InteractionData,0,1> vtable, zeroes the count at +8, releases the buffer at +4 through 00401c00 with alignment 4, and zeroes the buffer pointer. |
| `00a9b150` | `SoeUtil_ArrayInteractionData_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<InteractionData,0,1> vtable, zeroes the count at +8, releases the buffer at +4 through 00401c00, zeroes the buffer pointer, then if the low bit of the flag byte is set calls _free on the object and retu… |
