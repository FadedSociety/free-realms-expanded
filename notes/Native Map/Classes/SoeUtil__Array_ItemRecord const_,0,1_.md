# Class: SoeUtil::Array<ItemRecord const*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a134f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the array: for a requested element count it returns the current buffer (this+4) when capacity (this+0xc) already satisfies the request, otherwise picks a new capacity - request*5/4 when growing, reques… |
| `00a13590` | `Array_ItemRecordPtr_AllocatorSlot2_Free` | ✅ | virtual | Eighteen-byte deallocate hook: loads the buffer argument, pushes alignment 4, calls FUN_00401c00(buffer,4) and returns with ret 8. |
| `00a135b0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 3: when the flag argument is non-zero it dispatches to virtual slot 3 of the second array handle and returns; otherwise it exchanges three dwords between the two arrays - data pointer (this+4), size (this+8… |
| `00a14880` | `Array_ItemRecordPtr_Dtor` | ✅ | dtor | Destructor body (bytes 568bf1 8b4604 6a04 50 c706vtable c74608 0 e8...): stores the class vftable, zeroes the size at +8, frees the buffer at +4 through FUN_00401c00(ptr,4) and zeroes the pointer. |
| `00a14b10` | `Array_ItemRecordPtr_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores the class vftable, zeroes the size, frees the buffer at +4 with FUN_00401c00(ptr,4), clears the pointer and calls _free on the object when bit 0 of the flag argument is… |
