# Class: SoeUtil::Array<int,16,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a52df0` | `SoeUtil::Array_Int16_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 2) of Array<int,16,0>: frees the pointer through FreeMemory(ptr,4) unless it is the object's inline buffer ((this+0x13)&~3). |
| `00a55610` | `— ⚠reject` | ⚪ | dtor | Releases any non-inline buffer using alignment four, resets the static-16 array state, then installs dynamic-array vtable and frees the dynamic buffer. |
| `00a556a0` | `SoeUtil::ArrayInt16_Allocate` | ✅ | virtual | Sets the output capacity to 0x10 and returns the aligned inline buffer at this+0x13 rounded down to a four-byte boundary. |
| `00a56390` | `SoeUtil::ArrayInt16_ScalarDeletingDtor` | ✅ | dtor | Calls the inline-and-dynamic array destructor and conditionally frees the containing object when flag bit 0 is set. |
| `00a583e0` | `SoeUtil::ArrayInt16_Ctor` | ✅ | ctor | Initializes a dynamic int array's three storage fields to zero, switches to the inline-capacity-16 vtable, then calls FUN_0073c370 with the supplied value. |
| `00a59a10` | `ArrayInt16Swap` | ✅ | virtual | Copies the other array into a temporary, copies this array into the other array, copies the temporary back into this array, then frees the temporary backing buffer with alignment 4. |
