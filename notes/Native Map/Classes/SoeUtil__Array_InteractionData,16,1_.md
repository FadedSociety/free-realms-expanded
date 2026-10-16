# Class: SoeUtil::Array<InteractionData,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9a9c0` | `SoeUtil_ArrayInteractionData16_FreeBuffer` | 🟢 | method | Frees the pointer argument through 00401c00 with alignment 4 unless it equals the object's inline storage address this+0x13 rounded down to 4 (this+0x10), in which case it does nothing. |
| `00a9af50` | `SoeUtil_ArrayInteractionData16_Dtor` | ✅ | dtor | Stores the Array<InteractionData,16,1> vtable, zeroes the count, releases the buffer at +4 through 00401c00 unless it is the inline storage at this+0x13 rounded to 4, then stores the Array<InteractionData,0,1> vtable, z… |
| `00a9b010` | `SoeUtil_ArrayInteractionData16_AllocBuffer` | 🟢 | method | Buffer-policy method for an array with 16 inline 0x20-byte slots: when the requested count is above 0x10 it delegates to 00a9a890 and returns its buffer; otherwise it reports capacity 0x10 through the out-pointer and re… |
| `00a9ba90` | `SoeUtil_ArrayInteractionData16_Construct` | ✅ | ctor | Stores the Array<InteractionData,0,1> vtable, zeroes the words at +4, +8 and +0xc, then stores the Array<InteractionData,16,1> vtable. |
| `00a9baf0` | `SoeUtil_ArrayInteractionData16_ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9af50 (the Array<InteractionData,16,1> destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
| `00a9c8a0` | `SoeUtil_ArrayInteractionData16_Swap` | 🟢 | method | Thiscall with a second array of the same class. Builds a scratch Array<InteractionData,0,1> in the stack frame (vftable 018185d8), appends the argument array's 0x20-byte elements to it through 00a9b210 and zeroes the ar… |
