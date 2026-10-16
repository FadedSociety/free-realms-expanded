# Class: SoeUtil::Array<ItemRecord const*,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a13640` | `Array_ItemRecordPtr1_FreeInlineAware` | ✅ | virtual | Virtual slot 2 deallocate: identical to 00a13620 - frees through FUN_00401c00(ptr,4) only when the buffer is not the single inline element at this+0x10 derived as (this+0x13)&0xfffffffc. |
| `00a149f0` | `Array_ItemRecordPtr1_Dtor` | ✅ | dtor | Destructor of the 1-element small-buffer array: stores Array<...,1,1>::vftable, zeroes the size, frees the buffer with FUN_00401c00(ptr,4) unless it is the inline element, then chains to the base array destructor. |
| `00a14a80` | `Array_ItemRecordPtr1_BufferHook` | ✅ | virtual | Small-buffer hook: tail-jumps to the generic grow routine 00a134f0 when the requested count exceeds 1, otherwise returns the single inline element address (this+0x13)&0xfffffffc and reports capacity 1. |
| `00a15910` | `— ⚠reject` | ⚪ | ctor | Constructor: writes the base Array<...,0,1> vftable, zeroes the pointer, size and capacity, then installs the derived Array<...,1,1> vftable over the single inline element. |
| `00a15990` | `Array_ItemRecordPtr1_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): calls the 1-element array destructor 00a149f0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00a15b20` | `Array_ItemRecordPtr1_Swap` | ✅ | virtual | Virtual slot 3 (Swap) of the 1-inline-element array, byte-identical to 00a15a30: temporary copy of param_2, param_2 replaced by this array's elements with this size cleared, temporary appended back into this array and t… |
