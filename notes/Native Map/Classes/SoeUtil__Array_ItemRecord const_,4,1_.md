# Class: SoeUtil::Array<ItemRecord const*,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a13620` | `Array_ItemRecordPtr4_FreeInlineAware` | ✅ | virtual | Virtual slot 2 deallocate: computes the inline storage address as (this+0x13)&0xfffffffc and calls FUN_00401c00(ptr,4) only when the buffer differs from that inline array (bytes: 8b442404 83c113 83e1fc 3bc1 740b 6a04 50… |
| `00a148b0` | `Array_ItemRecordPtr4_Dtor` | ✅ | dtor | Destructor of the 4-element small-buffer array: stores Array<...,4,1>::vftable, zeroes the size, frees the buffer through FUN_00401c00(ptr,4) only when it is not the inline storage ((this+0x13)&~3), then chains to the b… |
| `00a14970` | `Array_ItemRecordPtr4_BufferHook` | ✅ | virtual | Small-buffer buffer hook: when the requested element count exceeds 4 it tail-jumps to the generic grow routine 00a134f0; otherwise it writes 4 to the out parameter and returns the inline storage address (this+0x13)&0xff… |
| `00a158b0` | `Array_ItemRecordPtr4_Ctor` | ✅ | ctor | Constructor: writes the base Array<...,0,1> vftable, zeroes the data pointer, size and capacity at +4/+8/+0xc, then writes the derived Array<...,4,1> vftable, leaving the inline storage ready at +0x10. |
| `00a15970` | `Array_ItemRecordPtr4_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0, bytes 568bf1 e838efffff f644240801 7409 56 e8...): calls the 4-element array destructor 00a148b0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00a15a30` | `Array_ItemRecordPtr4_Swap` | ✅ | virtual | Virtual slot 3 (Swap) of the 4-inline-element array implemented as three copies under an EH frame: builds a temporary array and appends param_2's elements to it, replaces param_2's contents with this array's elements an… |
